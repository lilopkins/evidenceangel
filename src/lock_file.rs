use std::{
    fs,
    io::{self, Write},
    path, process,
};

/// A locking file
pub struct LockFile {
    /// The path of this lockfile
    path: path::PathBuf,
    /// The file handle.
    _file: fs::File,
}

impl LockFile {
    /// Create a new lock file that is released when this is dropped.
    ///
    /// # Errors
    ///
    /// If this returns an error of any kind, it should be assumed that
    /// a lock could not be obtained.
    pub fn new<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<path::Path>,
    {
        let mut file = fs::File::create_new(&path)?;
        file.write_all(process::id().to_string().as_bytes())?;
        file.flush()?;

        #[allow(unsafe_code)]
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use winapi::um::fileapi::SetFileAttributesW;
            use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;

            let path = path.as_ref();
            let wide_path: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let _res = unsafe { SetFileAttributesW(wide_path.as_ptr(), FILE_ATTRIBUTE_HIDDEN) };
        }

        Ok(LockFile {
            path: path.as_ref().to_path_buf(),
            _file: file,
        })
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path) {
            tracing::warn!("Failed to delete lock file: {e}");
        }
    }
}
