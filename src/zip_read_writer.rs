use std::{
    fmt, fs,
    io::{BufReader, BufWriter},
    path,
};

use getset::Setters;
use zip::{ZipArchive, ZipWriter};

/// A convenient type which can read and write to a ZIP file and cleanly switch between the two modes.
///
/// Whilst writing, you can also read the previous file, as it writes to a new temporary file, until
/// [`ZipReaderWriter::conclude_write`] is called.
#[derive(Default, Setters)]
pub(crate) struct ZipReaderWriter {
    #[getset(set = "pub")]
    file: Option<path::PathBuf>,
    reader: Option<ZipArchive<BufReader<fs::File>>>,
    writer: Option<ZipWriter<BufWriter<fs::File>>>,
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
            .field("file", &self.file)
            .field("current_mode", &mode)
            .finish()
    }
}

impl ZipReaderWriter {
    pub fn new(path: path::PathBuf) -> Self {
        Self {
            file: Some(path),
            ..Default::default()
        }
    }

    pub fn as_reader(&mut self) -> crate::Result<&mut ZipArchive<BufReader<fs::File>>> {
        if self.reader.is_none() {
            // Close writer
            log::debug!("Closing writer");
            self.writer = None;

            // Open reader
            log::debug!("Opening reader");
            self.reader = Some(ZipArchive::new(BufReader::new(fs::File::open(
                self.file
                    .as_ref()
                    .expect("zipreadwriter must not be called upon until file is set."),
            )?))?);
        }
        Ok(self.reader.as_mut().unwrap())
    }

    pub fn as_writer(&mut self) -> crate::Result<&mut ZipWriter<BufWriter<fs::File>>> {
        if self.writer.is_none() {
            log::debug!("Opening writer");
            // Open writer
            self.writer = Some(ZipWriter::new(BufWriter::new(fs::File::create(
                self.file
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
        Ok(self.writer.as_mut().unwrap())
    }

    pub fn conclude_write(&mut self) -> crate::Result<()> {
        if self.writer.is_some() {
            // Close write
            log::debug!("Closing writer");
            self.writer = None;

            log::debug!("Closing reader");
            self.reader = None;

            // Move temp file
            log::debug!("Moving temp file to overwrite package");
            let tmp_path = self
                .file
                .as_ref()
                .map(|p| {
                    let mut p = p.clone();
                    p.set_file_name(format!("{}.tmp", p.file_name().unwrap().to_string_lossy()));
                    p
                })
                .expect("zipreadwriter must not be called upon until file is set.");
            fs::rename(
                tmp_path,
                self.file
                    .as_ref()
                    .expect("zipreadwriter must not be called upon until file is set."),
            )?;
        }
        Ok(())
    }
}
