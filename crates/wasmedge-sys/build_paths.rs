use crate::debug;

pub struct Env<S: AsRef<str>>(pub S);

impl<S: AsRef<str>> Env<S> {
    pub fn read<T: From<std::ffi::OsString>>(&self) -> Option<T> {
        if self.0.as_ref() != "OUT_DIR" && !self.0.as_ref().starts_with("CARGO_") {
            println!("cargo:rerun-if-env-changed={}", self.0.as_ref());
        }
        std::env::var_os(self.0.as_ref()).map(T::from)
    }

    pub fn expect<T: TryFrom<std::ffi::OsString>>(&self, msg: &str) -> T
    where
        <T as TryFrom<std::ffi::OsString>>::Error: std::fmt::Debug,
    {
        self.read::<std::ffi::OsString>()
            .map(T::try_from)
            .expect(msg)
            .expect(msg)
    }

    pub fn lossy(&self) -> Option<String> {
        self.read::<std::ffi::OsString>()
            .map(|v| v.to_string_lossy().to_string())
    }

    pub fn expect_lossy(&self, msg: &str) -> String {
        self.lossy().expect(msg)
    }
}

pub trait AsPath {
    fn as_path(&self) -> Option<std::path::PathBuf>;
}

impl<S: AsRef<str>> AsPath for Env<S> {
    fn as_path(&self) -> Option<std::path::PathBuf> {
        self.read()
    }
}

impl AsPath for &str {
    fn as_path(&self) -> Option<std::path::PathBuf> {
        Some(std::path::PathBuf::from(self))
    }
}

impl AsPath for &std::path::PathBuf {
    fn as_path(&self) -> Option<std::path::PathBuf> {
        Some((*self).clone())
    }
}

#[derive(Debug, Clone)]
pub struct LibWasmEdgePaths {
    pub lib_dir: std::path::PathBuf,
    pub inc_dir: std::path::PathBuf,
}

impl LibWasmEdgePaths {
    pub fn header(&self) -> std::path::PathBuf {
        self.inc_dir.join("wasmedge").join("wasmedge.h")
    }

    pub fn is_wasmedge_dir(&self) -> bool {
        debug!("searching for libwasmedge at {self:?}");
        self.header().exists() && self.lib_dir.exists()
    }

    pub fn try_from(
        base_dir: impl AsPath,
        inc_dir: impl AsPath,
        lib_dir: impl AsPath,
    ) -> Option<Self> {
        let pwd = std::env::current_dir().unwrap_or_default();
        match (base_dir.as_path(), inc_dir.as_path(), lib_dir.as_path()) {
            (Some(base_dir), Some(inc_dir), Some(lib_dir)) => Some(LibWasmEdgePaths {
                inc_dir: pwd.join(&base_dir).join(inc_dir),
                lib_dir: pwd.join(&base_dir).join(lib_dir),
            }),
            _ => None,
        }
    }
}
