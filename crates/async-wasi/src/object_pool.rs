use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ObjectPool<T> {
    stores: Vec<Vec<ObjectNode<T>>>,
}

impl<T> ObjectPool<T> {
    const DEFAULT_CAPACITY: usize = 512;

    pub fn new() -> Self {
        let mut pool = ObjectPool {
            stores: Vec::with_capacity(10),
        };
        pool.extend_stores();
        pool
    }

    #[inline]
    fn raw_index(index: usize) -> (usize, usize) {
        (
            index / Self::DEFAULT_CAPACITY,
            index % Self::DEFAULT_CAPACITY,
        )
    }

    fn extend_stores(&mut self) -> &mut [ObjectNode<T>] {
        let mut new_vec = Vec::with_capacity(Self::DEFAULT_CAPACITY);
        new_vec.resize_with(Self::DEFAULT_CAPACITY, ObjectNode::default);
        new_vec[0].header.next_chunk_offset = Self::DEFAULT_CAPACITY;
        self.stores.push(new_vec);
        self.stores.last_mut().unwrap()
    }

    pub fn push(&mut self, value: T) -> (usize, Option<T>) {
        if let Some(ObjectIndex {
            store_index,
            target_index,
            chunk_index,
        }) = self.first_none()
        {
            let v = self.stores[store_index][target_index].obj.replace(value);

            //try merge chunk
            {
                let current_chunk = &mut self.stores[store_index][chunk_index].header;
                current_chunk.next_none_offset += 1;
                if current_chunk.next_none_offset == current_chunk.next_chunk_offset
                    && current_chunk.next_chunk_offset < Self::DEFAULT_CAPACITY
                {
                    let next_chunk_offset = current_chunk.next_chunk_offset;
                    let next_chunk = self.stores[store_index][next_chunk_offset].header;

                    let current_chunk = &mut self.stores[store_index][chunk_index].header;
                    *current_chunk = next_chunk;
                }
            }

            (store_index * Self::DEFAULT_CAPACITY + target_index, v)
        } else {
            let store = self.extend_stores();
            let node = &mut store[0];
            node.header.next_none_offset += 1;
            let v = node.obj.replace(value);
            ((self.stores.len() - 1) * Self::DEFAULT_CAPACITY, v)
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        let (
            ObjectIndex {
                store_index,
                target_index,
                chunk_index,
            },
            last_chunk_index,
        ) = self.value_and_chunk(index)?;

        let v = self.stores[store_index][target_index].obj.take()?;
        let current_chunk = &mut self.stores[store_index][chunk_index].header;

        // try update chunk header
        {
            if target_index == current_chunk.next_none_offset - 1 {
                current_chunk.next_none_offset = target_index;
                if chunk_index == current_chunk.next_none_offset {
                    let next_chunk = current_chunk.next_chunk_offset;
                    let last_chunk = &mut self.stores[store_index][last_chunk_index].header;
                    last_chunk.next_chunk_offset = next_chunk;
                }
            } else if target_index == chunk_index {
                if index == 0 {
                    let new_chunk = *current_chunk;
                    current_chunk.next_none_offset = 0;
                    current_chunk.next_chunk_offset = 1;
                    let next_chunk = &mut self.stores[store_index][1].header;
                    *next_chunk = new_chunk;
                } else {
                    let new_chunk = *current_chunk;
                    let next_chunk = &mut self.stores[store_index][target_index + 1].header;
                    *next_chunk = new_chunk;
                    let last_chunk = &mut self.stores[store_index][last_chunk_index].header;
                    last_chunk.next_chunk_offset += 1;
                }
            } else {
                let new_chunk = *current_chunk;
                current_chunk.next_chunk_offset = target_index + 1;
                current_chunk.next_none_offset = target_index;
                let next_chunk = &mut self.stores[store_index][target_index + 1].header;
                *next_chunk = new_chunk;
            }
        }

        Some(v)
    }

    fn empty_chunk(&self) -> Option<(usize, bool)> {
        let chunk_headers = self.stores.iter().map(|s| s[0].header).rev();

        if chunk_headers.len() <= 1 {
            return None;
        }

        let empty_chunk = ChunkHead {
            next_none_offset: 0,
            next_chunk_offset: Self::DEFAULT_CAPACITY,
        };

        let mut empty_num = 0;
        let mut res_chunk_is_full = false;

        for chunk in chunk_headers {
            if chunk == empty_chunk {
                empty_num += 1;
            } else {
                if chunk.next_chunk_offset == chunk.next_none_offset {
                    res_chunk_is_full = true;
                }
                break;
            }
        }
        if empty_num == 0 {
            None
        } else {
            Some((empty_num, res_chunk_is_full))
        }
    }

    pub fn cleanup_stores(&mut self) {
        if let Some((mut n, res_is_full)) = self.empty_chunk() {
            if res_is_full {
                n -= 1;
            }
            for _ in 0..n {
                self.stores.pop();
            }
        }
    }

    fn first_none(&self) -> Option<ObjectIndex> {
        for (store_index, store) in self.stores.iter().enumerate() {
            'next_chunk: loop {
                let node = &store[0];
                let header = node.header;
                if header.next_none_offset == Self::DEFAULT_CAPACITY {
                    break 'next_chunk;
                }
                return Some(ObjectIndex {
                    store_index,
                    target_index: header.next_none_offset,
                    chunk_index: 0,
                });
            }
        }
        None
    }

    fn value_and_chunk(&self, index: usize) -> Option<(ObjectIndex, usize)> {
        let (store_index, value_index) = Self::raw_index(index);
        let store = self.stores.get(store_index)?;
        let mut last_chunk_index = 0;
        let mut chunk_index = 0;
        loop {
            let node = &store[chunk_index];
            debug_assert!(node.header.next_chunk_offset > chunk_index);
            if value_index < node.header.next_chunk_offset {
                return Some((
                    ObjectIndex {
                        store_index,
                        target_index: value_index,
                        chunk_index,
                    },
                    last_chunk_index,
                ));
            }
            last_chunk_index = chunk_index;
            chunk_index = node.header.next_chunk_offset;
            debug_assert!(chunk_index < Self::DEFAULT_CAPACITY);
            if chunk_index >= Self::DEFAULT_CAPACITY {
                return None;
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let (store_index, target_index) = Self::raw_index(index);
        let store = self.stores.get(store_index)?;
        let obj_node = &store[target_index];
        obj_node.obj.as_ref()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let (store_index, target_index) = Self::raw_index(index);
        let store = self.stores.get_mut(store_index)?;
        let obj_node = &mut store[target_index];
        obj_node.obj.as_mut()
    }
}

impl<T: Clone> Clone for ObjectPool<T> {
    fn clone(&self) -> Self {
        ObjectPool {
            stores: self.stores.clone(),
        }
    }
}

impl<T> ObjectPool<T> {
    pub fn iter(&self) -> impl Iterator<Item = Option<&T>> {
        let skip_end = self.empty_chunk().map(|(n, _)| n).unwrap_or(0);
        let stores_len = self.stores.len();
        self.stores[0..(stores_len - skip_end)]
            .iter()
            .flat_map(|store| store.iter())
            .map(|node| node.obj.as_ref())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = Option<&mut T>> {
        let skip_end = self.empty_chunk().map(|(n, _)| n).unwrap_or(0);
        let stores_len = self.stores.len();
        self.stores[0..(stores_len - skip_end)]
            .iter_mut()
            .flat_map(|store| store.iter_mut())
            .map(|node| node.obj.as_mut())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerialObjectPool<T>(Vec<Vec<SerializeChunk<T>>>);

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SerializeChunk<T> {
    next_none: usize,
    next_chunk: usize,
    values: Vec<T>,
}

impl<T> SerialObjectPool<T> {
    pub fn into<U, F: FnMut(T) -> U>(self, mut f: F) -> ObjectPool<U> {
        let mut pool = ObjectPool {
            stores: Vec::with_capacity(self.0.len()),
        };

        for serial_store in self.0 {
            let stores = pool.extend_stores();
            let mut chunk_id = 0;

            for serial_chunk in serial_store {
                let chunk = &mut stores[chunk_id];
                chunk.header.next_chunk_offset = serial_chunk.next_chunk;
                chunk.header.next_none_offset = serial_chunk.next_none;
                for (i, v) in serial_chunk.values.into_iter().enumerate() {
                    stores[chunk_id + i].obj = Some(f(v));
                }

                chunk_id = serial_chunk.next_chunk;
            }
            if chunk_id == ObjectPool::<U>::DEFAULT_CAPACITY {
                continue;
            }
        }

        pool
    }

    pub fn from_ref<U, F: FnMut(&U) -> T>(pool: &ObjectPool<U>, mut f: F) -> SerialObjectPool<T> {
        let skip_end = pool.empty_chunk().map(|(n, _)| n).unwrap_or(0);
        let stores_len = pool.stores.len();
        let mut serial_stores = Vec::new();
        for store in pool.stores[0..(stores_len - skip_end)].iter() {
            let mut serial_chunks = Vec::new();
            let mut chunk_index = 0;
            loop {
                let node = &store[chunk_index];

                let mut serial_chunk = SerializeChunk {
                    next_none: node.header.next_none_offset,
                    next_chunk: node.header.next_chunk_offset,
                    values: vec![],
                };
                for i in chunk_index..node.header.next_none_offset {
                    serial_chunk.values.push(f(&store[i].obj.as_ref().unwrap()));
                }

                chunk_index = node.header.next_chunk_offset;
                serial_chunks.push(serial_chunk);
                if chunk_index == ObjectPool::<U>::DEFAULT_CAPACITY {
                    break;
                }
            }
            serial_stores.push(serial_chunks);
        }

        SerialObjectPool(serial_stores)
    }

    pub fn from<U, F: FnMut(U) -> T>(mut pool: ObjectPool<U>, mut f: F) -> SerialObjectPool<T> {
        let skip_end = pool.empty_chunk().map(|(n, _)| n).unwrap_or(0);
        let stores_len = pool.stores.len();
        let mut serial_stores = Vec::new();
        for store in pool.stores[0..(stores_len - skip_end)].iter_mut() {
            let mut serial_chunks = Vec::new();
            let mut chunk_index = 0;
            loop {
                let chunk_header = store[chunk_index].header;

                let mut serial_chunk = SerializeChunk {
                    next_none: chunk_header.next_none_offset,
                    next_chunk: chunk_header.next_chunk_offset,
                    values: vec![],
                };
                for i in chunk_index..chunk_header.next_none_offset {
                    let v = &mut store[i].obj;
                    serial_chunk.values.push(f(v.take().unwrap()));
                }
                serial_chunks.push(serial_chunk);
                chunk_index = chunk_header.next_chunk_offset;
                if chunk_index == ObjectPool::<U>::DEFAULT_CAPACITY {
                    break;
                }
            }
            serial_stores.push(serial_chunks);
        }

        SerialObjectPool(serial_stores)
    }
}

struct ObjectIndex {
    store_index: usize,
    target_index: usize,
    chunk_index: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ChunkHead {
    next_none_offset: usize,
    next_chunk_offset: usize,
}

#[derive(Debug)]
pub struct ObjectNode<T> {
    obj: Option<T>,
    header: ChunkHead,
}

impl<T> Default for ObjectNode<T> {
    fn default() -> Self {
        Self {
            obj: None,
            header: ChunkHead::default(),
        }
    }
}

impl<T: Clone> Clone for ObjectNode<T> {
    fn clone(&self) -> Self {
        ObjectNode {
            obj: self.obj.clone(),
            header: self.header.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let mut pool = ObjectPool::new();
        let _ = pool.push("hello");
        let (id2, _) = pool.push("world");
        pool.push("example");
        pool.remove(id2);
        pool.push("foo");

        let r = pool
            .iter()
            .take(4)
            .map(|s| s.map(|s| *s))
            .collect::<Vec<Option<&str>>>();

        assert_eq!(r, vec![Some("hello"), Some("foo"), Some("example"), None]);
    }

    #[test]
    fn test_push() {
        let mut pool = ObjectPool::new();
        assert_eq!(pool.push("0"), (0, None));
        assert_eq!(pool.push("1"), (1, None));
        assert_eq!(pool.push("2"), (2, None));
        assert_eq!(pool.push("3"), (3, None));
        assert_eq!(pool.push("4"), (4, None));
        assert_eq!(
            pool.stores[0][0].header,
            ChunkHead {
                next_none_offset: 5,
                next_chunk_offset: ObjectPool::<&str>::DEFAULT_CAPACITY
            }
        );
    }

    #[test]
    fn test_push_and_remove() {
        let mut pool = ObjectPool::new();

        assert_eq!(pool.push("0"), (0, None));
        assert_eq!(pool.push("1"), (1, None));
        assert_eq!(pool.push("2"), (2, None));
        assert_eq!(pool.push("3"), (3, None));
        assert_eq!(pool.push("4"), (4, None));
        // |----------*
        // |0|1|2|3|4|
        // |----------DEFAULT_CAPACITY--*
        assert_eq!(pool.remove(2), Some("2"));
        assert_eq!(
            pool.stores[0][0].header,
            ChunkHead {
                next_none_offset: 2,
                next_chunk_offset: 3
            }
        );
        assert_eq!(
            pool.stores[0][3].header,
            ChunkHead {
                next_none_offset: 5,
                next_chunk_offset: ObjectPool::<&str>::DEFAULT_CAPACITY
            }
        );
        // |----*|----*
        // |0|1|_|3|4|
        // |------|-----DEFAULT_CAPACITY--*

        assert_eq!(pool.remove(1), Some("1"));
        // |--*  |----*
        // |0|_|_|3|4|
        // |------|-----DEFAULT_CAPACITY--*
        assert_eq!(
            pool.stores[0][0].header,
            ChunkHead {
                next_none_offset: 1,
                next_chunk_offset: 3
            }
        );

        assert_eq!(pool.remove(3), Some("3"));
        // |--*    |--*
        // |0|_|_|_|4|
        // |--------|-----DEFAULT_CAPACITY--*
        assert_eq!(
            pool.stores[0][0].header,
            ChunkHead {
                next_none_offset: 1,
                next_chunk_offset: 4
            }
        );
        assert_eq!(
            pool.stores[0][4].header,
            ChunkHead {
                next_none_offset: 5,
                next_chunk_offset: ObjectPool::<&str>::DEFAULT_CAPACITY
            }
        );

        assert_eq!(pool.push("1"), (1, None));
        // |----*  |--*
        // |0|1|_|_|4|
        // |--------|-----DEFAULT_CAPACITY--*
        assert_eq!(
            pool.stores[0][0].header,
            ChunkHead {
                next_none_offset: 2,
                next_chunk_offset: 4
            }
        );

        assert_eq!(pool.remove(0), Some("0"));
        // |*|--*  |--*
        // |_|1|_|_|4|
        // |--|-----|-----DEFAULT_CAPACITY--*
        assert_eq!(
            pool.stores[0][0].header,
            ChunkHead {
                next_none_offset: 0,
                next_chunk_offset: 1
            }
        );
        assert_eq!(
            pool.stores[0][1].header,
            ChunkHead {
                next_none_offset: 2,
                next_chunk_offset: 4
            }
        );
        let v = pool
            .iter()
            .take(5)
            .map(|s| s.map(|s| *s))
            .collect::<Vec<Option<&str>>>();
        assert_eq!(v, vec![None, Some("1"), None, None, Some("4")]);

        assert_eq!(pool.remove(1), Some("1"));
        // |*      |--*
        // |_|_|_|_|4|
        // |--------|-----DEFAULT_CAPACITY--*
        assert_eq!(
            pool.stores[0][0].header,
            ChunkHead {
                next_none_offset: 0,
                next_chunk_offset: 4
            }
        );
        assert_eq!(
            pool.stores[0][4].header,
            ChunkHead {
                next_none_offset: 5,
                next_chunk_offset: ObjectPool::<&str>::DEFAULT_CAPACITY
            }
        );
        let v = pool
            .iter()
            .take(5)
            .map(|s| s.map(|s| *s))
            .collect::<Vec<Option<&str>>>();
        assert_eq!(v, vec![None, None, None, None, Some("4")]);

        assert_eq!(pool.push("0"), (0, None));
        assert_eq!(pool.push("1"), (1, None));
        assert_eq!(pool.push("2"), (2, None));
        // |------*|--*
        // |0|1|2|_|4|
        // |--------|-----DEFAULT_CAPACITY--*
        assert_eq!(
            pool.stores[0][0].header,
            ChunkHead {
                next_none_offset: 3,
                next_chunk_offset: 4
            }
        );
        assert_eq!(
            pool.stores[0][4].header,
            ChunkHead {
                next_none_offset: 5,
                next_chunk_offset: ObjectPool::<&str>::DEFAULT_CAPACITY
            }
        );

        assert_eq!(pool.push("3"), (3, None));
        // |----------*
        // |0|1|2|3|4|
        // |--------------DEFAULT_CAPACITY--*
        assert_eq!(
            pool.stores[0][0].header,
            ChunkHead {
                next_none_offset: 5,
                next_chunk_offset: ObjectPool::<&str>::DEFAULT_CAPACITY
            }
        );
    }

    #[test]
    fn test_store_extends() {
        let mut pool = ObjectPool::new();
        let cap = ObjectPool::<&str>::DEFAULT_CAPACITY;
        for i in 0..cap {
            assert_eq!(pool.push(format!("{i}")), (i, None));
        }

        assert_eq!(pool.push("example".to_string()), (cap, None));
        assert_eq!(pool.push("foo".to_string()), (cap + 1, None));
        assert_eq!(pool.push("bar".to_string()), (cap + 2, None));

        assert_eq!(
            pool.stores[1][0].header,
            ChunkHead {
                next_none_offset: 3,
                next_chunk_offset: ObjectPool::<&str>::DEFAULT_CAPACITY
            }
        );
    }

    #[test]
    fn test_cleanup() {
        let mut pool = ObjectPool::new();
        let cap = ObjectPool::<()>::DEFAULT_CAPACITY;
        for i in 0..cap {
            pool.push(i);
        }
        pool.extend_stores();
        pool.extend_stores();

        assert_eq!(pool.stores.len(), 3);

        assert_eq!(
            pool.stores[0][0].header,
            ChunkHead {
                next_none_offset: cap,
                next_chunk_offset: cap
            }
        );

        pool.cleanup_stores();
        assert_eq!(pool.stores.len(), 2);

        pool.remove(2);
        pool.cleanup_stores();
        assert_eq!(pool.stores.len(), 1);
    }

    #[test]
    fn test_serde() {
        let mut pool = ObjectPool::new();
        let cap = ObjectPool::<&str>::DEFAULT_CAPACITY;
        for i in 0..cap {
            assert_eq!(pool.push(format!("{i}")), (i, None));
        }
        assert_eq!(pool.push("example".to_string()), (cap, None));
        assert_eq!(pool.push("foo".to_string()), (cap + 1, None));
        assert_eq!(pool.push("bar".to_string()), (cap + 2, None));

        // fake data
        {
            let s = pool.extend_stores();
            s[0].header.next_none_offset += 1;
            s[0].obj = Some("fake".to_string());
        }

        for i in 64..128 {
            pool.remove(i);
        }

        for i in 64 + 128..512 {
            pool.remove(i);
        }

        let serde_pool = SerialObjectPool::from(pool, |f| f);
        let s = serde_json::to_string_pretty(&serde_pool).unwrap();
        let expect = {
            r#"
[
  [
    {
      "next_none": 64,
      "next_chunk": 128,
      "values": [
        "0",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        "10",
        "11",
        "12",
        "13",
        "14",
        "15",
        "16",
        "17",
        "18",
        "19",
        "20",
        "21",
        "22",
        "23",
        "24",
        "25",
        "26",
        "27",
        "28",
        "29",
        "30",
        "31",
        "32",
        "33",
        "34",
        "35",
        "36",
        "37",
        "38",
        "39",
        "40",
        "41",
        "42",
        "43",
        "44",
        "45",
        "46",
        "47",
        "48",
        "49",
        "50",
        "51",
        "52",
        "53",
        "54",
        "55",
        "56",
        "57",
        "58",
        "59",
        "60",
        "61",
        "62",
        "63"
      ]
    },
    {
      "next_none": 192,
      "next_chunk": 512,
      "values": [
        "128",
        "129",
        "130",
        "131",
        "132",
        "133",
        "134",
        "135",
        "136",
        "137",
        "138",
        "139",
        "140",
        "141",
        "142",
        "143",
        "144",
        "145",
        "146",
        "147",
        "148",
        "149",
        "150",
        "151",
        "152",
        "153",
        "154",
        "155",
        "156",
        "157",
        "158",
        "159",
        "160",
        "161",
        "162",
        "163",
        "164",
        "165",
        "166",
        "167",
        "168",
        "169",
        "170",
        "171",
        "172",
        "173",
        "174",
        "175",
        "176",
        "177",
        "178",
        "179",
        "180",
        "181",
        "182",
        "183",
        "184",
        "185",
        "186",
        "187",
        "188",
        "189",
        "190",
        "191"
      ]
    }
  ],
  [
    {
      "next_none": 3,
      "next_chunk": 512,
      "values": [
        "example",
        "foo",
        "bar"
      ]
    }
  ],
  [
    {
      "next_none": 1,
      "next_chunk": 512,
      "values": [
        "fake"
      ]
    }
  ]
]"#
        };
        assert_eq!(s, expect.trim());

        let new_pool = serde_pool.into(|s| s);
        assert_eq!(new_pool.stores.len(), 3);

        assert_eq!(
            new_pool.stores[0][0].header,
            ChunkHead {
                next_none_offset: 64,
                next_chunk_offset: 128
            }
        );
        assert_eq!(
            new_pool.stores[0][128].header,
            ChunkHead {
                next_none_offset: 192,
                next_chunk_offset: 512
            }
        );
        assert_eq!(
            new_pool.stores[1][0].header,
            ChunkHead {
                next_none_offset: 3,
                next_chunk_offset: 512
            }
        );
        assert_eq!(
            new_pool.stores[2][0].header,
            ChunkHead {
                next_none_offset: 1,
                next_chunk_offset: 512
            }
        );
    }
}
