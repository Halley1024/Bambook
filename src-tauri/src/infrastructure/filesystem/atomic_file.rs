use std::{
    ffi::OsString,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::error::{AppError, AppResult};

pub(crate) fn write_bytes_atomically(path: &Path, bytes: &[u8]) -> AppResult<()> {
    write_file_atomically(path, |temporary_path| {
        fs::write(temporary_path, bytes)?;
        Ok(())
    })
}

pub(crate) fn write_file_atomically(
    path: &Path,
    writer: impl FnOnce(&Path) -> AppResult<()>,
) -> AppResult<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let temporary_path = create_adjacent_temporary_path(path)?;
    let mut guard = TemporaryFileGuard::new(temporary_path.clone());
    writer(&temporary_path)?;
    OpenOptions::new()
        .write(true)
        .open(&temporary_path)?
        .sync_all()?;
    replace_file_atomically(&temporary_path, path)?;
    guard.disarm();
    Ok(())
}

fn create_adjacent_temporary_path(path: &Path) -> AppResult<PathBuf> {
    let file_name = path.file_name().ok_or(AppError::MissingFileName)?;
    for _ in 0..16 {
        let mut temporary_name = OsString::from(".");
        temporary_name.push(file_name);
        temporary_name.push(format!(
            ".{}-{}.tmp",
            std::process::id(),
            NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        let temporary_path = path.with_file_name(temporary_name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
        {
            Ok(file) => {
                drop(file);
                return Ok(temporary_path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not allocate an adjacent temporary file",
    )
    .into())
}

#[cfg(windows)]
fn replace_file_atomically(source: &Path, destination: &Path) -> AppResult<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let succeeded = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if succeeded == 0 {
        Err(std::io::Error::last_os_error().into())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_file_atomically(source: &Path, destination: &Path) -> AppResult<()> {
    fs::rename(source, destination)?;
    if let Some(parent) = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

struct TemporaryFileGuard {
    path: PathBuf,
    armed: bool,
}

impl TemporaryFileGuard {
    fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for TemporaryFileGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::error::AppError;

    use super::write_file_atomically;

    #[test]
    fn failed_generation_keeps_the_existing_destination() {
        let root = std::env::temp_dir().join(format!(
            "bambook-atomic-failure-{}-{}",
            std::process::id(),
            super::NEXT_TEMP_FILE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let destination = root.join("document.html");
        fs::write(&destination, b"existing").unwrap();

        let result = write_file_atomically(&destination, |temporary| {
            fs::write(temporary, b"partial")?;
            Err(AppError::InvalidExportOutput { format: "test" })
        });

        assert!(result.is_err());
        assert_eq!(fs::read(&destination).unwrap(), b"existing");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}
