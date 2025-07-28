use std::{
    fmt, fs,
    io::{BufReader, BufWriter},
    path,
};

use zip::{ZipArchive, ZipWriter};

use crate::lock_file::LockFile;

/// A convenient type which can read and write to a ZIP file and cleanly switch between the two modes.
///
/// Whilst writing, you can also read the previous file, as it writes to a new temporary file, until
/// [`ZipReaderWriter::conclude_write`] is called.
#[derive(Default)]
pub(crate) struct ZipReaderWriter {
    /// The path of this zip file, if set
    path: Option<path::PathBuf>,
    /// The locking file for this evidence package. If this is [`Some`],
    /// you can be assured that the lock is obtained.
    lock_file: Option<LockFile>,
    /// The reader, if in write mode, of this reader/writer
    reader: Option<ZipArchive<BufReader<fs::File>>>,
    /// The writer, if in write mode, of this reader/writer
    writer: Option<ZipWriter<BufWriter<fs::File>>>,
}

impl Clone for ZipReaderWriter {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            ..Default::default()
        }
    }
}

impl fmt::Debug for ZipReaderWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode = if self.reader.is_some() {
            "read"
        } else if self.writer.is_some() {
            "write"
        } else {
            "idle"
        };
        f.debug_struct("ZipReadWriter")
            .field("file", &self.path)
            .field("current_mode", &mode)
            .finish_non_exhaustive()
    }
}

impl ZipReaderWriter {
    /// Create a new [`ZipReaderWriter`] instance.
    pub fn new(path: path::PathBuf) -> crate::Result<Self> {
        let mut o = Self {
            path: Some(path),
            ..Default::default()
        };
        o.update_lock_file()?;
        Ok(o)
    }

    /// Validate that the currently held lock is still locking the
    /// package.
    fn validate_lock(&mut self) -> crate::Result<()> {
        if let Some(lock_file) = self.lock_file.as_mut() {
            lock_file.ensure_still_locked().map_err(|e| {
                tracing::error!("The lock was lost! {e}");
                crate::Error::LockNotObtained
            })
        } else {
            Err(crate::Error::LockNotObtained)
        }
    }

    /// Update the locking file for this [`ZipReaderWriter`]. This will
    /// either obtain it (if a path is set), drop it (if a path isn't
    /// set), or will return a [`crate::Error::LockNotObtained`] error.
    fn update_lock_file(&mut self) -> crate::Result<()> {
        if let Some(path) = &self.path {
            let mut lock_path = path.clone();
            // SAFETY: only a file can be specified here
            lock_path.set_file_name(format!(
                ".~lock.{}",
                lock_path.file_name().unwrap().to_str().unwrap()
            ));
            // SAFETY: only a file can be specified here
            lock_path.set_extension(format!(
                "{}#",
                lock_path.extension().unwrap().to_str().unwrap()
            ));
            self.lock_file = Some(LockFile::new(lock_path).map_err(|e| {
                tracing::error!("Locking error: {e}");
                crate::Error::LockNotObtained
            })?);
        } else {
            self.lock_file = None;
        }
        Ok(())
    }

    /// Get this [`ZipReaderWriter`] instance in read mode.
    pub fn as_reader(&mut self) -> crate::Result<&mut ZipArchive<BufReader<fs::File>>> {
        if self.reader.is_none() {
            // Close writer
            tracing::debug!("Closing writer");
            self.writer = None;

            // Open reader
            tracing::debug!("Opening reader");
            self.reader = Some(ZipArchive::new(BufReader::new(fs::File::open(
                self.path
                    .as_ref()
                    .expect("zipreadwriter must not be called upon until file is set."),
            )?))?);
        }
        Ok(self.reader.as_mut().unwrap())
    }

    /// Get this [`ZipReaderWriter`] instance in write mode.
    #[allow(clippy::type_complexity)]
    pub fn as_writer(
        &mut self,
    ) -> crate::Result<(
        Option<&mut ZipArchive<BufReader<fs::File>>>,
        &mut ZipWriter<BufWriter<fs::File>>,
    )> {
        self.validate_lock()?;
        if self.writer.is_none() {
            tracing::debug!("Opening writer");
            // Open writer
            self.writer = Some(ZipWriter::new(BufWriter::new(fs::File::create(
                self.path
                    .as_ref()
                    .map(|p| {
                        let mut p = p.clone();
                        p.set_file_name(format!(
                            "{}.tmp",
                            p.file_name().unwrap().to_string_lossy()
                        ));
                        p
                    })
                    .expect("zipreadwriter must not be called upon until file is set."),
            )?)));
        }
        Ok((self.reader.as_mut(), self.writer.as_mut().unwrap()))
    }

    /// Conclude writing to the ZIP file and reset for reading or writing again.
    pub fn conclude_write(&mut self) -> crate::Result<()> {
        self.validate_lock()?;
        if self.writer.is_some() {
            // Close write
            tracing::debug!("Closing writer");
            let writer = self.writer.take().unwrap();
            writer.finish()?;

            tracing::debug!("Closing reader");
            self.reader = None;

            // Move temp file
            tracing::debug!("Moving temp file to overwrite package");
            let tmp_path = self
                .path
                .as_ref()
                .map(|p| {
                    let mut p = p.clone();
                    p.set_file_name(format!("{}.tmp", p.file_name().unwrap().to_string_lossy()));
                    p
                })
                .expect("zipreadwriter must not be called upon until file is set.");
            fs::rename(
                tmp_path,
                self.path
                    .as_ref()
                    .expect("zipreadwriter must not be called upon until file is set."),
            )?;
        }
        Ok(())
    }

    /// Interrupt a write early, concluding the write and removing the temporary file.
    pub fn interrupt_write(&mut self) -> crate::Result<()> {
        self.validate_lock()?;
        if self.writer.is_some() {
            // Close write
            tracing::debug!("Closing writer");
            let writer = self.writer.take().unwrap();
            writer.finish()?;

            tracing::debug!("Closing reader");
            self.reader = None;

            // Delete temp file
            tracing::debug!("Moving temp file to overwrite package");
            let tmp_path = self
                .path
                .as_ref()
                .map(|p| {
                    let mut p = p.clone();
                    p.set_file_name(format!("{}.tmp", p.file_name().unwrap().to_string_lossy()));
                    p
                })
                .expect("zipreadwriter must not be called upon until file is set.");
            fs::remove_file(tmp_path)?;
        }
        Ok(())
    }
}
