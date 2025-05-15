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
    file: fs::File,
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
        Ok(LockFile {
            path: path.as_ref().to_path_buf(),
            file: Self::create_file_handle(path)?,
        })
    }

    /// Actually create the file handle
    fn create_file_handle<P>(path: P) -> io::Result<fs::File>
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

        Ok(file)
    }

    /// Check that the lock is still in place, recreating it if needed.
    ///
    /// # Errors
    ///
    /// A failure here indicates that the lock is no longer in place,
    /// and cannot be replaced. This might be as a result of another
    /// user deleting the lock file and obtaining their own. You should
    /// handle this with caution.
    pub fn ensure_still_locked(&mut self) -> io::Result<()> {
        if fs::exists(&self.path)? {
            Ok(())
        } else {
            // Need to reobtain file lock
            self.file = Self::create_file_handle(&self.path)?;
            Ok(())
        }
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path) {
            tracing::warn!("Failed to delete lock file: {e}");
        }
    }
}
