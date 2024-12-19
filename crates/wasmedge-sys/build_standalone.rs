use super::{REMOTE_ARCHIVES, STANDALONE_DIR, WASMEDGE_RELEASE_VERSION};
use crate::{
    build_paths::{AsPath, Env},
    debug,
};

#[derive(Debug)]
enum Archive {
    Local { path: std::path::PathBuf },
    Remote { url: String, checksum: String },
}

impl Archive {
    fn hash(&self) -> String {
        match self {
            Archive::Local { path } => sha256::try_digest(path).expect("failed to read archive"),
            Archive::Remote { checksum, .. } => checksum.clone(),
        }
    }

    fn get(&self) -> std::path::PathBuf {
        match self {
            Archive::Local { path } => path.clone(),
            Archive::Remote { url, checksum } => {
                debug!("downloading archive");
                let dst = STANDALONE_DIR.join("archive.tar.gz");
                let mut request = do_http_request(url);
                let mut file = std::fs::File::create(&dst).expect("failed to create archive");
                std::io::copy(&mut request, &mut file).expect("failed to download archive");
                let sha = sha256::try_digest(&dst).expect("failed to read archive");
                if &sha != checksum {
                    panic!("wrong archive checksum, expected {checksum}, received {sha}");
                }
                dst
            }
        }
    }

    fn cleanup(&self) {
        if let Archive::Remote { .. } = self {
            let _ = std::fs::remove_file(STANDALONE_DIR.join("archive.tar.gz"));
        }
    }
}

pub fn get_standalone_libwasmedge() -> std::path::PathBuf {
    let archive = match Env("WASMEDGE_STANDALONE_ARCHIVE").as_path() {
        Some(path) => Archive::Local { path },
        None => get_remote_archive(),
    };
    debug!("using archive {archive:?}");

    let hash = archive.hash();
    if hash == std::fs::read_to_string(STANDALONE_DIR.join(".stamp")).unwrap_or_default() {
        debug!("skipping extraction, archive is already extracted");
    } else {
        if STANDALONE_DIR.exists() {
            debug!("deleting previous extraction");
            std::fs::remove_dir_all(STANDALONE_DIR.as_path()).expect("failed to cleanup directory")
        }
        std::fs::create_dir_all(STANDALONE_DIR.as_path()).expect("failed to create archive dir");

        let file = archive.get();
        let readable = std::fs::File::open(file).expect("failed to open archive");
        let ungzipped = flate2::read::GzDecoder::new(readable);

        debug!("extracting archive");
        tar::Archive::new(ungzipped)
            .unpack(STANDALONE_DIR.as_path())
            .expect("failed to extract archive");

        archive.cleanup();

        std::fs::write(STANDALONE_DIR.join(".stamp"), hash).expect("failed to write archive stamp");
    }

    std::fs::read_dir(STANDALONE_DIR.as_path())
        .expect("failed to read archive directory")
        .filter_map(|entry| entry.ok())
        .find(|entry| match entry.file_type() {
            Ok(ty) => ty.is_dir(),
            _ => false,
        })
        .expect("failed to find WasmEdge in archive directory")
        .path()
}

fn get_remote_archive() -> Archive {
    let os = Env("CARGO_CFG_TARGET_OS").expect_lossy("failed to read CARGO_CFG_TARGET_OS");
    let arch = Env("CARGO_CFG_TARGET_ARCH").expect_lossy("failed to read CARGO_CFG_TARGET_ARCH");
    let libc = Env("CARGO_CFG_TARGET_ENV").expect_lossy("failed to read CARGO_CFG_TARGET_ENV");

    let mut target = format!("{os}/{arch}");
    if os == "linux" && !libc.is_empty() {
        target = target + "/" + &libc;
    }
    if cfg!(feature = "static") {
        target += "/static";
    }

    debug!("building archive url for target {target}");
    let (sha, slug) = REMOTE_ARCHIVES
        .get(target.as_str())
        .expect("target not supported with features `standalone` and `static`")
        .to_owned();

    let asset_name = if cfg!(feature = "static") {
        format!("WasmEdge-{WASMEDGE_RELEASE_VERSION}-fmt-patch-{slug}.tar.gz")
    } else {
        format!("WasmEdge-{WASMEDGE_RELEASE_VERSION}-{slug}.tar.gz")
    };
    let url = format!("https://github.com/WasmEdge/WasmEdge/releases/download/{WASMEDGE_RELEASE_VERSION}/{asset_name}");

    let checksum = sha.to_string();
    Archive::Remote { url, checksum }
}

fn do_http_request(url: &str) -> impl std::io::Read {
    let builder = reqwest::blocking::Client::builder();
    let builder = match Env("WASMEDGE_STANDALONE_PROXY").lossy() {
        Some(proxy) => {
            debug!("using proxy to download archive: {proxy}");
            let proxy = reqwest::Proxy::all(proxy).expect("failed to parse proxy");
            let user = Env("WASMEDGE_STANDALONE_PROXY_USER").lossy();
            let pass = Env("WASMEDGE_STANDALONE_PROXY_PASS").lossy();
            let proxy = match user.zip(pass) {
                Some((user, pass)) => proxy.basic_auth(&user, &pass),
                _ => proxy,
            };
            builder.proxy(proxy)
        }
        None => builder,
    };
    builder
        .build()
        .expect("failed to create http request")
        .get(url)
        .timeout(std::time::Duration::from_secs(3600 * 24))
        .send()
        .expect("failed to download archive")
}
