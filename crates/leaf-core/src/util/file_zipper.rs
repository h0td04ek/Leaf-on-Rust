//! Перенос `net.minecraft.util.FileZipper`.
//! `FileSystems.newFileSystem(zipfs)` → `zip` crate (ZipWriter).

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// `FileZipper` — zip-архив в temp-файл, переименовываемый при close.
pub struct FileZipper {
    output_file: PathBuf,
    temp_file: PathBuf,
    writer: Option<zip::ZipWriter<File>>,
}

impl FileZipper {
    /// `new FileZipper(outputFile)`.
    pub fn new(output_file: &Path) -> io::Result<Self> {
        let file_name = output_file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let temp_file = output_file.with_file_name(format!("{}_tmp", file_name));
        let file = File::create(&temp_file)?;
        let writer = zip::ZipWriter::new(file);
        Ok(Self { output_file: output_file.to_path_buf(), temp_file, writer: Some(writer) })
    }

    /// `add(path, filename)` — текстовый файл с UTF-8 содержимым.
    pub fn add_text(&mut self, path: &Path, filename: &str) -> io::Result<()> {
        let name = zip_entry_name(path);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let writer = self.writer.as_mut().expect("FileZipper already closed");
        writer.start_file(name, options)?;
        writer.write_all(filename.as_bytes())
    }

    /// `add(path, File filename)` — файл с диска.
    pub fn add_file(&mut self, path: &Path, filename: &Path) -> io::Result<()> {
        self.add_source(path, filename)
    }

    /// `add(path)` — файл или рекурсивно директория.
    pub fn add(&mut self, path: &Path) -> io::Result<()> {
        if path.is_file() {
            // Java: entry = path.getParent().relativize(path) → только имя файла
            let entry = Path::new(path.file_name().unwrap_or_default());
            return self.add_source(entry, path);
        }
        // директория — рекурсивный обход
        let mut stack = vec![path.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    let rel = p.strip_prefix(path).unwrap_or(&p);
                    self.add_source(rel, &p)?;
                }
            }
        }
        Ok(())
    }

    fn add_source(&mut self, entry_path: &Path, source: &Path) -> io::Result<()> {
        let name = zip_entry_name(entry_path);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let writer = self.writer.as_mut().expect("FileZipper already closed");
        writer.start_file(name, options)?;
        let mut f = File::open(source)?;
        io::copy(&mut f, writer)?;
        Ok(())
    }

    /// Дополнительный метод: добавить поток байтов.
    pub fn add_bytes(&mut self, path: &Path, data: &[u8]) -> io::Result<()> {
        let name = zip_entry_name(path);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let writer = self.writer.as_mut().expect("FileZipper already closed");
        writer.start_file(name, options)?;
        writer.write_all(data)
    }

    /// `close()` — завершить zip и переименовать temp → output.
    pub fn close(mut self) -> io::Result<()> {
        self.finish_inner()
    }

    fn finish_inner(&mut self) -> io::Result<()> {
        if let Some(writer) = self.writer.take() {
            let file = writer.finish()?;
            drop(file);
        }
        fs::rename(&self.temp_file, &self.output_file)
    }
}

fn zip_entry_name(path: &Path) -> String {
    path.iter()
        .map(|c| c.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

impl Drop for FileZipper {
    fn drop(&mut self) {
        let _ = self.finish_inner();
    }
}
