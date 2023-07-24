use super::{REMOTE_ARCHIVE_SLUGS, STANDALONE_DIR, WASMEDGE_RELEASE_VERSION};
use crate::{
    build_paths::{AsPath, Env},
    debug,
};

#[derive(Debug)]
enum Archive {
    Local(std::path::PathBuf),
    Remote(String),
}

impl Archive {
    fn hash(&self) -> String {
        match self {
            Archive::Local(path) => sha256::try_digest(path).expect("failed to read archive"),
            Archive::Remote(url) => sha256::digest(url),
        }
    }

    fn read(&self) -> std::boxed::Box<dyn std::io::Read> {
        match self {
            Archive::Local(path) => {
                Box::new(std::fs::File::open(path).expect("failed to open archive"))
            }
            Archive::Remote(url) => {
                debug!("downloading archive");
                Box::new(
                    reqwest::blocking::Client::new()
                        .get(url)
                        .timeout(std::time::Duration::from_secs(3600 * 24))
                        .send()
                        .expect("failed to download archive"),
                )
            }
        }
    }
}

pub fn get_standalone_libwasmedge() -> std::path::PathBuf {
    let archive = match Env("WASMEDGE_STANDALONE_ARCHIVE").as_path() {
        Some(path) => Archive::Local(path),
        None => Archive::Remote(get_download_url()),
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

        let readable = archive.read();
        let ungzipped = flate2::read::GzDecoder::new(readable);

        debug!("extracting archive");
        tar::Archive::new(ungzipped)
            .unpack(STANDALONE_DIR.as_path())
            .expect("failed to extract archive");

        std::fs::write(STANDALONE_DIR.join(".stamp"), hash).expect("failed to write archive stamp");
    }

    std::fs::read_dir(STANDALONE_DIR.as_path())
        .expect("failed to read archive directory")
        .next()
        .expect("failed to find WasmEdge in archive directory")
        .expect("failed to find WasmEdge in archive directory")
        .path()
}

fn get_download_url() -> String {
    let os = Env("CARGO_CFG_TARGET_OS").lossy("failed to read CARGO_CFG_TARGET_OS");
    let arch = Env("CARGO_CFG_TARGET_ARCH").lossy("failed to read CARGO_CFG_TARGET_ARCH");
    let target = if cfg!(feature = "static") {
        format!("{os}_{arch}_static")
    } else {
        format!("{os}_{arch}")
    };

    debug!("building archive url for target {target}");
    let slug = REMOTE_ARCHIVE_SLUGS
        .get(target.as_str())
        .expect("target not supported with features `standalone` and `static`");

    format!("https://github.com/WasmEdge/WasmEdge/releases/download/{WASMEDGE_RELEASE_VERSION}/WasmEdge-{WASMEDGE_RELEASE_VERSION}-{slug}.tar.gz")
}
