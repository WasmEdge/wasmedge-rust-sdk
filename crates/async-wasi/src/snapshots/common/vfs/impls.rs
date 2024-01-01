use std::{
    collections::HashMap,
    fmt::Debug,
    io::{Read, Seek, Write},
};

use crate::snapshots::env::Errno;

use super::{
    virtual_sys::{WasiVirtualDir, WasiVirtualFile},
    WasiDir, WasiFile, WasiNode,
};

#[derive(Debug)]
struct DirEntry {
    ino: usize,
    is_dir: bool,
}

#[derive(Debug)]
pub struct MemoryDir {
    ino: usize,
    nlink: usize,
    paths: HashMap<String, DirEntry>,
    is_open: usize,
}

impl Drop for MemoryDir {
    fn drop(&mut self) {
        log::trace!("\r\n{self:#?}")
    }
}

impl Drop for MemoryFile {
    fn drop(&mut self) {
        log::trace!("\r\n{self:#?} \r\n {:#?}", self.context.get_ref());
    }
}

impl WasiNode for MemoryDir {
    fn fd_fdstat_get(&self) -> Result<super::FdStat, Errno> {
        Ok(super::FdStat {
            filetype: super::FileType::DIRECTORY,
            fs_rights_base: super::WASIRights::dir_all(),
            fs_rights_inheriting: super::WASIRights::fd_all(),
            flags: super::FdFlags::empty(),
        })
    }

    fn fd_filestat_get(&self) -> Result<super::Filestat, Errno> {
        Ok(super::Filestat {
            filetype: super::FileType::DIRECTORY,
            inode: self.ino as _,
            nlink: self.nlink as _,
            size: self.paths.len() as _,
            atim: None,
            mtim: None,
            ctim: None,
        })
    }

    fn fd_filestat_set_size(
        &mut self,
        size: crate::snapshots::env::wasi_types::__wasi_filesize_t,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_filestat_set_times(
        &mut self,
        atim: crate::snapshots::env::wasi_types::__wasi_timestamp_t,
        mtim: crate::snapshots::env::wasi_types::__wasi_timestamp_t,
        fst_flags: crate::snapshots::env::wasi_types::__wasi_fstflags_t::Type,
    ) -> Result<(), Errno> {
        Ok(())
    }
}

impl WasiDir for MemoryDir {
    fn get_readdir(&self, start: u64) -> Result<Vec<(String, u64, super::FileType)>, Errno> {
        let mut r = vec![];
        for (path, DirEntry { ino, is_dir }) in self.paths.iter().skip(start as usize) {
            r.push((
                path.clone(),
                *ino as _,
                if *is_dir {
                    super::FileType::DIRECTORY
                } else {
                    super::FileType::REGULAR_FILE
                },
            ));
        }
        Ok(r)
    }
}

impl WasiVirtualDir for MemoryDir {
    fn create(ino: usize) -> Self {
        let mut paths = HashMap::default();
        paths.insert(".".to_string(), DirEntry { ino, is_dir: true });
        if ino == 0 {
            paths.insert("..".to_string(), DirEntry { ino, is_dir: true });
        }

        Self {
            ino,
            paths,
            is_open: 0,
            nlink: 1,
        }
    }

    fn add_sub_dir<P: AsRef<std::path::Path>>(
        &mut self,
        path: &P,
        ino: usize,
    ) -> Result<(), Errno> {
        self.paths.insert(
            path.as_ref()
                .to_str()
                .ok_or(Errno::__WASI_ERRNO_ILSEQ)?
                .to_string(),
            DirEntry { ino, is_dir: true },
        );
        self.nlink += 1;
        Ok(())
    }

    fn remove_sub_dir<P: AsRef<std::path::Path>>(&mut self, path: &P) -> Result<(), Errno> {
        let path = path.as_ref().to_str().ok_or(Errno::__WASI_ERRNO_ILSEQ)?;
        if let Some(DirEntry { ino, is_dir }) = self.paths.remove(path) {
            if is_dir && self.nlink > 1 {
                self.nlink -= 1;
            }
            Ok(())
        } else {
            Err(Errno::__WASI_ERRNO_NOENT)
        }
    }

    fn link_inode<P: AsRef<std::path::Path>>(
        &mut self,
        path: &P,
        ino: usize,
    ) -> Result<(), crate::snapshots::env::Errno> {
        self.paths.insert(
            path.as_ref()
                .to_str()
                .ok_or(Errno::__WASI_ERRNO_ILSEQ)?
                .to_string(),
            DirEntry { ino, is_dir: false },
        );
        Ok(())
    }

    fn unlink_inode<P: AsRef<std::path::Path>>(
        &mut self,
        path: &P,
    ) -> Result<(), crate::snapshots::env::Errno> {
        let path = path.as_ref().to_str().ok_or(Errno::__WASI_ERRNO_ILSEQ)?;
        if self.paths.remove(path).is_some() {
            Ok(())
        } else {
            Err(Errno::__WASI_ERRNO_NOENT)
        }
    }

    fn find_inode<P: AsRef<std::path::Path>>(&self, path: &P) -> Option<usize> {
        let path = path.as_ref().to_str()?;
        if path.is_empty() {
            return Some(self.ino);
        }
        let entry = self.paths.get(path)?;
        Some(entry.ino)
    }

    fn is_empty(&self) -> bool {
        self.paths.len() <= 2
    }

    fn is_open(&self) -> bool {
        self.is_open > 0
    }

    fn open(&mut self) {
        self.is_open += 1;
    }

    fn close(&mut self) -> usize {
        if self.is_open > 0 {
            self.is_open -= 1;
        }
        self.nlink
    }

    fn mark_remove(&mut self) {
        self.nlink = 0
    }
}

pub struct MemoryFile {
    context: std::io::Cursor<Vec<u8>>,
    nlink: usize,
    ino: usize,
    is_open: bool,
}

impl WasiNode for MemoryFile {
    fn fd_fdstat_get(&self) -> Result<super::FdStat, Errno> {
        Ok(super::FdStat {
            filetype: super::FileType::REGULAR_FILE,
            fs_rights_base: super::WASIRights::dir_all(),
            fs_rights_inheriting: super::WASIRights::fd_all(),
            flags: super::FdFlags::empty(),
        })
    }

    fn fd_filestat_get(&self) -> Result<super::Filestat, Errno> {
        Ok(super::Filestat {
            filetype: super::FileType::REGULAR_FILE,
            inode: self.ino as _,
            nlink: self.nlink as _,
            size: self.context.get_ref().len() as _,
            atim: None,
            mtim: None,
            ctim: None,
        })
    }

    fn fd_filestat_set_size(
        &mut self,
        size: crate::snapshots::env::wasi_types::__wasi_filesize_t,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_NOTSUP)
    }

    fn fd_filestat_set_times(
        &mut self,
        atim: crate::snapshots::env::wasi_types::__wasi_timestamp_t,
        mtim: crate::snapshots::env::wasi_types::__wasi_timestamp_t,
        fst_flags: crate::snapshots::env::wasi_types::__wasi_fstflags_t::Type,
    ) -> Result<(), Errno> {
        Ok(())
    }
}

impl WasiFile for MemoryFile {
    fn fd_read(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> Result<usize, Errno> {
        Ok(self.context.read_vectored(bufs)?)
    }

    fn fd_pread(
        &mut self,
        bufs: &mut [std::io::IoSliceMut<'_>],
        offset: crate::snapshots::env::wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno> {
        let old_pos = self.context.position();
        self.context.set_position(offset);
        let n = self.context.read_vectored(bufs);
        self.context.set_position(old_pos);
        Ok(n?)
    }

    fn fd_write(&mut self, bufs: &[std::io::IoSlice<'_>]) -> Result<usize, Errno> {
        Ok(self.context.write_vectored(bufs)?)
    }

    fn fd_pwrite(
        &mut self,
        bufs: &[std::io::IoSlice<'_>],
        offset: crate::snapshots::env::wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno> {
        let old_pos = self.context.position();
        self.context.set_position(offset);
        let n = self.context.write_vectored(bufs);
        self.context.set_position(old_pos);
        Ok(n?)
    }

    fn fd_seek(
        &mut self,
        offset: crate::snapshots::env::wasi_types::__wasi_filedelta_t,
        whence: crate::snapshots::env::wasi_types::__wasi_whence_t::Type,
    ) -> Result<crate::snapshots::env::wasi_types::__wasi_filesize_t, Errno> {
        let pos = match whence {
            crate::snapshots::env::wasi_types::__wasi_whence_t::__WASI_WHENCE_CUR => {
                std::io::SeekFrom::Current(offset)
            }
            crate::snapshots::env::wasi_types::__wasi_whence_t::__WASI_WHENCE_END => {
                std::io::SeekFrom::End(offset)
            }
            crate::snapshots::env::wasi_types::__wasi_whence_t::__WASI_WHENCE_SET => {
                std::io::SeekFrom::Start(offset as u64)
            }
            _ => return Err(Errno::__WASI_ERRNO_INVAL),
        };
        Ok(self.context.seek(pos)?)
    }

    fn fd_tell(&mut self) -> Result<crate::snapshots::env::wasi_types::__wasi_filesize_t, Errno> {
        Ok(self.context.position() as _)
    }
}

impl From<Vec<u8>> for MemoryFile {
    fn from(value: Vec<u8>) -> Self {
        Self {
            context: std::io::Cursor::new(value),
            nlink: 0,
            ino: 0,
            is_open: false,
        }
    }
}

impl WasiVirtualFile for MemoryFile {
    fn create(ino: usize) -> Self {
        Self {
            context: std::io::Cursor::new(Vec::new()),
            nlink: 0,
            ino,
            is_open: false,
        }
    }

    fn set_ino(&mut self, ino: usize) {
        self.ino = ino;
    }

    fn inc_link(&mut self) -> Result<usize, Errno> {
        self.nlink += 1;
        Ok(self.nlink)
    }

    fn dec_link(&mut self) -> Result<usize, Errno> {
        if self.nlink > 0 {
            self.nlink -= 1;
        }
        Ok(self.nlink)
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn open(&mut self) {
        self.is_open = true;
    }

    fn close(&mut self) -> usize {
        self.is_open = false;
        self.nlink
    }
}

impl Debug for MemoryFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryFile")
            .field("context_len", &self.context.get_ref().len())
            .field("nlink", &self.nlink)
            .field("ino", &self.ino)
            .field("is_open", &self.is_open)
            .finish()
    }
}
