use std::cell;
use std::collections;
use std::fmt;
use std::fs;
use std::io;
use std::path;
use std::rc;

/// # File system interface
///
/// This is the main OS interface that Qinhuai uses to interact with the file system.
///
/// See: <https://www.sqlite.org/c3ref/vfs.html>
pub trait FileSystem {
  /// The type of errors that can occur when interacting with this file system.
  type Error: fmt::Debug + fmt::Display;

  /// The type of paths that this file system uses.
  type Path: ?Sized;

  /// The type of files that this file system uses.
  type File: File<Error = Self::Error>;

  /// Opens a file at the given `path`, creating it if it does not exist.
  fn open(&mut self, path: &Self::Path) -> Result<Self::File, Self::Error>;

  /// Deletes the file at the given `path`.
  fn delete(&mut self, path: &Self::Path) -> Result<(), Self::Error>;
}

/// # File interface
///
/// This is the main OS interface that Qinhuai uses to interact with files.
///
/// See: <https://www.sqlite.org/c3ref/io_methods.html>
pub trait File {
  /// The type of errors that can occur when interacting with this file.
  type Error: fmt::Debug + fmt::Display;

  /// Returns the size of the file in bytes.
  fn size(&mut self) -> Result<u64, Self::Error>;

  /// Sets the size of the file in bytes.
  fn truncate(&mut self, size: u64) -> Result<(), Self::Error>;

  /// Reads `amount` bytes from the file at the given `offset`.
  fn read(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), Self::Error>;

  /// Writes `data` to the file at the given `offset`.
  fn write(&mut self, offset: u64, buf: &[u8]) -> Result<(), Self::Error>;

  /// Flushes any buffered data to the file.
  fn sync(&mut self) -> Result<(), Self::Error>;

  /// Tries locking the file exclusively.
  fn try_lock(&mut self) -> Result<(), Self::Error>;

  /// Locks the file exclusively.
  fn lock(&mut self) -> Result<(), Self::Error>;

  /// Unlocks the file.
  fn unlock(&mut self) -> Result<(), Self::Error>;
}

/// # The primary implementation for [`FileSystem`]
///
/// This is simply a wrapper around [`std::fs`].
#[derive(Debug)]
pub struct StandardFileSystem;

/// Public constructor for [`StandardFileSystem`].
impl Default for StandardFileSystem {
  fn default() -> Self {
    StandardFileSystem
  }
}

impl FileSystem for StandardFileSystem {
  type Error = io::Error;
  type Path = path::Path;
  type File = StandardFile;

  fn open(&mut self, path: &Self::Path) -> Result<Self::File, Self::Error> {
    let file = fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path)?;
    Ok(Self::File::from(file))
  }

  fn delete(&mut self, path: &Self::Path) -> Result<(), Self::Error> {
    fs::remove_file(path)
  }
}

/// # The primary implementation for [`File`]
///
/// This is simply a wrapper around [`std::fs::File`].
#[derive(Debug)]
pub struct StandardFile(fs::File);

/// Public constructor for [`StandardFile`].
impl From<fs::File> for StandardFile {
  fn from(file: fs::File) -> Self {
    Self(file)
  }
}

impl File for StandardFile {
  type Error = io::Error;

  fn size(&mut self) -> Result<u64, Self::Error> {
    let StandardFile(inner) = self;
    io::Seek::seek(inner, io::SeekFrom::End(0))
  }

  fn truncate(&mut self, size: u64) -> Result<(), Self::Error> {
    let StandardFile(inner) = self;
    fs::File::set_len(inner, size)
  }

  fn read(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
    let StandardFile(inner) = self;
    io::Seek::seek(inner, io::SeekFrom::Start(offset))?;
    io::Read::read_exact(inner, buf)
  }

  fn write(&mut self, offset: u64, buf: &[u8]) -> Result<(), Self::Error> {
    let StandardFile(inner) = self;
    io::Seek::seek(inner, io::SeekFrom::Start(offset))?;
    io::Write::write_all(inner, buf)
  }

  fn sync(&mut self) -> Result<(), Self::Error> {
    let StandardFile(inner) = self;
    fs::File::sync_all(inner)
  }

  fn try_lock(&mut self) -> Result<(), Self::Error> {
    let StandardFile(inner) = self;
    fs2::FileExt::try_lock_exclusive(inner)
  }

  fn lock(&mut self) -> Result<(), Self::Error> {
    let StandardFile(inner) = self;
    fs2::FileExt::lock_exclusive(inner)
  }

  fn unlock(&mut self) -> Result<(), Self::Error> {
    let StandardFile(inner) = self;
    fs2::FileExt::unlock(inner)
  }
}

#[derive(Debug, Default)]
struct MemoryFileData {
  data: Vec<u8>,
  locked: bool,
}

/// In-memory implementation for [`FileSystem`]
///
/// Each file is represented by a byte vector and a boolean indicating whether the file is locked.
#[derive(Debug)]
pub struct MemoryFileSystem {
  files: collections::HashMap<String, rc::Rc<cell::RefCell<MemoryFileData>>>,
}

/// Public constructor for [`MemoryFileSystem`].
impl Default for MemoryFileSystem {
  fn default() -> Self {
    MemoryFileSystem { files: collections::HashMap::new() }
  }
}

impl FileSystem for MemoryFileSystem {
  type Error = String;
  type Path = str;
  type File = MemoryFile;

  fn open(&mut self, path: &Self::Path) -> Result<Self::File, Self::Error> {
    let file = self.files.entry(path.to_string()).or_default();
    Ok(file.clone().into())
  }

  fn delete(&mut self, path: &Self::Path) -> Result<(), Self::Error> {
    let file = self.files.remove(path);
    file.map(|_| ()).ok_or(String::new())
  }
}

/// In-memory implementation for [`File`]
///
/// Each file is represented by a byte vector and a boolean indicating whether the file is locked.
#[derive(Debug)]
pub struct MemoryFile {
  file: rc::Rc<cell::RefCell<MemoryFileData>>,
}

/// Public constructor for [`MemoryFile`].
impl From<rc::Rc<cell::RefCell<MemoryFileData>>> for MemoryFile {
  fn from(file: rc::Rc<cell::RefCell<MemoryFileData>>) -> Self {
    MemoryFile { file }
  }
}

impl File for MemoryFile {
  type Error = String;

  fn size(&mut self) -> Result<u64, Self::Error> {
    u64::try_from(self.file.borrow().data.len()).map_err(|x| x.to_string())
  }

  fn truncate(&mut self, size: u64) -> Result<(), Self::Error> {
    let size = usize::try_from(size).map_err(|x| x.to_string())?;
    self.file.borrow_mut().data.resize(size, 0xCC);
    Ok(())
  }

  fn read(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
    let offset = usize::try_from(offset).map_err(|x| x.to_string())?;
    let file = self.file.borrow();
    if offset + buf.len() > file.data.len() {
      return Err(String::new());
    }
    buf.copy_from_slice(&file.data[offset..offset + buf.len()]);
    Ok(())
  }

  fn write(&mut self, offset: u64, buf: &[u8]) -> Result<(), Self::Error> {
    let offset = usize::try_from(offset).map_err(|x| x.to_string())?;
    let mut file = self.file.borrow_mut();
    if offset + buf.len() > file.data.len() {
      file.data.resize(offset + buf.len(), 0xCC);
    }
    file.data[offset..offset + buf.len()].copy_from_slice(buf);
    Ok(())
  }

  fn sync(&mut self) -> Result<(), Self::Error> {
    Ok(())
  }

  fn try_lock(&mut self) -> Result<(), Self::Error> {
    let mut file = self.file.borrow_mut();
    if file.locked {
      Err(String::new())
    } else {
      file.locked = true;
      Ok(())
    }
  }

  fn lock(&mut self) -> Result<(), Self::Error> {
    let mut file = self.file.borrow_mut();
    if file.locked {
      Err(String::new())
    } else {
      file.locked = true;
      Ok(())
    }
  }

  fn unlock(&mut self) -> Result<(), Self::Error> {
    let mut file = self.file.borrow_mut();
    if file.locked {
      file.locked = false;
      Ok(())
    } else {
      Err(String::new())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile;

  fn test_filesystem_open_create<F: FileSystem>(fs: &mut F, path: &F::Path) {
    let mut file = fs.open(path).unwrap();
    assert_eq!(file.size().unwrap(), 0);
  }

  fn test_filesystem_open_existing<F: FileSystem>(fs: &mut F, path: &F::Path) {
    let mut file = fs.open(path).unwrap();
    file.write(0, b"hello").unwrap();

    let mut file = fs.open(path).unwrap();
    assert_eq!(file.size().unwrap(), 5);

    let mut buf = vec![0; 5];
    file.read(0, &mut buf).unwrap();
    assert_eq!(&buf, b"hello");
  }

  fn test_filesystem_delete<F: FileSystem>(fs: &mut F, path: &F::Path) {
    fs.open(path).unwrap();
    fs.delete(path).unwrap();
  }

  fn test_filesystem_delete_nonexistent<F: FileSystem>(fs: &mut F, path: &F::Path) {
    fs.delete(path).unwrap_err();
  }

  fn test_file_size<F: File>(file: &mut F) {
    file.write(0, b"hello").unwrap();
    assert_eq!(file.size().unwrap(), 5);
  }

  fn test_file_truncate<F: File>(file: &mut F) {
    file.write(0, b"hello").unwrap();
    file.truncate(2).unwrap();
    assert_eq!(file.size().unwrap(), 2);

    let mut buf = vec![0; 2];
    file.read(0, &mut buf).unwrap();
    assert_eq!(&buf, b"he");
  }

  fn test_file_extend<F: File>(file: &mut F) {
    file.write(0, b"hello").unwrap();
    file.truncate(8).unwrap();
    assert_eq!(file.size().unwrap(), 8);

    let mut buf = vec![0; 5];
    file.read(0, &mut buf).unwrap();
    assert_eq!(&buf, b"hello");
  }

  fn test_file_read_write<F: File>(file: &mut F) {
    file.write(0, b"hello").unwrap();
    let mut buf = vec![0; 5];
    file.read(0, &mut buf).unwrap();
    assert_eq!(&buf, b"hello");
  }

  fn test_file_read_past_eof<F: File>(file: &mut F) {
    file.write(0, b"hello").unwrap();
    let mut buf = vec![0; 5];
    file.read(4, &mut buf).unwrap_err();
  }

  fn test_file_write_past_eof<F: File>(file: &mut F) {
    file.write(0, b"hello").unwrap();
    file.write(4, b"world").unwrap();
    assert_eq!(file.size().unwrap(), 9);

    let mut buf = vec![0; 9];
    file.read(0, &mut buf).unwrap();
    assert_eq!(&buf, b"hellworld");
  }

  fn test_file_sync<F: File>(file: &mut F) {
    file.write(0, b"hello").unwrap();
    file.sync().unwrap();
  }

  fn test_file_lock_unlock<F: File>(file1: &mut F, file2: &mut F) {
    // No other access is possible once an exclusive lock is created.
    file1.lock().unwrap();
    file2.try_lock().unwrap_err();

    // Once the exclusive lock is dropped, the second file is able to create a lock.
    file1.unlock().unwrap();
    file2.lock().unwrap();
  }

  #[test]
  fn test_standard_filesystem_open_create() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    test_filesystem_open_create(&mut fs, &path);
  }

  #[test]
  fn test_standard_filesystem_open_existing() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    test_filesystem_open_existing(&mut fs, &path);
  }

  #[test]
  fn test_standard_filesystem_delete() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    test_filesystem_delete(&mut fs, &path);
  }

  #[test]
  fn test_standard_filesystem_delete_nonexistent() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    test_filesystem_delete_nonexistent(&mut fs, &path);
  }

  #[test]
  fn test_standard_file_size() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    let mut file = fs.open(&path).unwrap();
    test_file_size(&mut file);
  }

  #[test]
  fn test_standard_file_truncate() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    let mut file = fs.open(&path).unwrap();
    test_file_truncate(&mut file);
  }

  #[test]
  fn test_standard_file_extend() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    let mut file = fs.open(&path).unwrap();
    test_file_extend(&mut file);
  }

  #[test]
  fn test_standard_file_read_write() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    let mut file = fs.open(&path).unwrap();
    test_file_read_write(&mut file);
  }

  #[test]
  fn test_standard_file_read_past_eof() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    let mut file = fs.open(&path).unwrap();
    test_file_read_past_eof(&mut file);
  }

  #[test]
  fn test_standard_file_write_past_eof() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    let mut file = fs.open(&path).unwrap();
    test_file_write_past_eof(&mut file);
  }

  #[test]
  fn test_standard_file_sync() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    let mut file = fs.open(&path).unwrap();
    test_file_sync(&mut file);
  }

  #[test]
  fn test_standard_file_lock_unlock() {
    let mut fs = StandardFileSystem;
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("file");
    let mut file1 = fs.open(&path).unwrap();
    let mut file2 = fs.open(&path).unwrap();
    test_file_lock_unlock(&mut file1, &mut file2);
  }

  #[test]
  fn test_memory_filesystem_open_create() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    test_filesystem_open_create(&mut fs, &path);
  }

  #[test]
  fn test_memory_filesystem_open_existing() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    test_filesystem_open_existing(&mut fs, &path);
  }

  #[test]
  fn test_memory_filesystem_delete() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    test_filesystem_delete(&mut fs, &path);
  }

  #[test]
  fn test_memory_filesystem_delete_nonexistent() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    test_filesystem_delete_nonexistent(&mut fs, &path);
  }

  #[test]
  fn test_memory_file_size() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    let mut file = fs.open(&path).unwrap();
    test_file_size(&mut file);
  }

  #[test]
  fn test_memory_file_truncate() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    let mut file = fs.open(&path).unwrap();
    test_file_truncate(&mut file);
  }

  #[test]
  fn test_memory_file_extend() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    let mut file = fs.open(&path).unwrap();
    test_file_extend(&mut file);
  }

  #[test]
  fn test_memory_file_read_write() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    let mut file = fs.open(&path).unwrap();
    test_file_read_write(&mut file);
  }

  #[test]
  fn test_memory_file_read_past_eof() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    let mut file = fs.open(&path).unwrap();
    test_file_read_past_eof(&mut file);
  }

  #[test]
  fn test_memory_file_write_past_eof() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    let mut file = fs.open(&path).unwrap();
    test_file_write_past_eof(&mut file);
  }

  #[test]
  fn test_memory_file_sync() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    let mut file = fs.open(&path).unwrap();
    test_file_sync(&mut file);
  }

  #[test]
  fn test_memory_file_lock_unlock() {
    let mut fs = MemoryFileSystem::default();
    let path = "file".to_owned();
    let mut file1 = fs.open(&path).unwrap();
    let mut file2 = fs.open(&path).unwrap();
    test_file_lock_unlock(&mut file1, &mut file2);
  }
}
