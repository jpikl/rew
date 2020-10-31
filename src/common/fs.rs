use fs_extra::error::{Error, ErrorKind, Result};
use fs_extra::{dir, file};
use lazy_static::lazy_static;
use std::path::Path;

lazy_static! {
    static ref FILE_COPY_OPTIONS: file::CopyOptions = get_file_copy_options();
    static ref DIR_COPY_OPTIONS: dir::CopyOptions = get_dir_copy_options();
}

fn get_file_copy_options() -> file::CopyOptions {
    let mut options = file::CopyOptions::new();
    options.overwrite = true;
    options.skip_exist = false;
    options
}

fn get_dir_copy_options() -> dir::CopyOptions {
    let mut options = dir::CopyOptions::new();
    options.overwrite = true;
    options.skip_exist = false;
    options.copy_inside = true;
    options.content_only = true;
    options
}

#[derive(PartialEq, Debug)]
pub enum FileType {
    File,
    Dir,
    Unknown,
}

impl From<&Path> for FileType {
    fn from(path: &Path) -> Self {
        match path.metadata() {
            Ok(metadata) => {
                if metadata.is_dir() {
                    FileType::Dir
                } else {
                    FileType::File
                }
            }
            Err(_) => FileType::Unknown,
        }
    }
}

pub enum TransferMode {
    Copy,
    Move,
}

pub fn transfer_path(src_path: &Path, dst_path: &Path, mode: TransferMode) -> Result<()> {
    match (FileType::from(src_path), FileType::from(dst_path)) {
        (FileType::Unknown, _) => Err(Error::new(
            ErrorKind::NotFound,
            &format!(
                "Path '{}' not found or user lacks permission",
                src_path.to_string_lossy()
            ),
        )),

        (FileType::File, FileType::Dir) => Err(Error::new(
            ErrorKind::Other,
            &format!(
                "Cannot to overwrite directory '{}' with file '{}'",
                dst_path.to_string_lossy(),
                src_path.to_string_lossy()
            ),
        )),

        (FileType::Dir, FileType::File) => Err(Error::new(
            ErrorKind::Other,
            &format!(
                "Cannot to overwrite file '{}' with directory '{}'",
                dst_path.to_string_lossy(),
                src_path.to_string_lossy()
            ),
        )),

        (FileType::File, _) => {
            // TODO test
            if let Some(dst_parent) = dst_path.parent() {
                dir::create_all(dst_parent, false)?;
            }
            match mode {
                TransferMode::Copy => {
                    file::copy(src_path, dst_path, &FILE_COPY_OPTIONS)?;
                }
                TransferMode::Move => {
                    // TODO try rename first
                    file::move_file(src_path, dst_path, &FILE_COPY_OPTIONS)?;
                }
            }
            Ok(())
        }

        (FileType::Dir, _) => {
            // TODO test
            dir::create_all(dst_path, false)?;
            match mode {
                TransferMode::Copy => {
                    dir::copy(src_path, dst_path, &DIR_COPY_OPTIONS)?;
                }
                TransferMode::Move => {
                    // TODO try rename first
                    dir::move_dir(src_path, dst_path, &DIR_COPY_OPTIONS)?;
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::FileType;
    use crate::testing::{debug_fse_error_kind, unpack_fse_error};
    use assert_fs::prelude::*;
    use assert_fs::{NamedTempFile, TempDir};
    use fs_extra::error::ErrorKind;

    #[test]
    pub fn file_type_file() {
        let file = NamedTempFile::new("a").unwrap();
        file.touch().unwrap();

        assert_eq!(FileType::from(file.path()), FileType::File);
    }

    #[test]
    pub fn file_type_dir() {
        assert_eq!(
            FileType::from(TempDir::new().unwrap().path()),
            FileType::Dir
        );
    }

    #[test]
    pub fn file_type_unknown() {
        assert_eq!(
            FileType::from(NamedTempFile::new("a").unwrap().path()),
            FileType::Unknown
        );
    }

    #[test]
    fn path_not_found_error() {
        let src_file = NamedTempFile::new("a").unwrap();

        assert_eq!(
            transfer_path(src_file.path(), &Path::new("b"), TransferMode::Copy)
                .map_err(unpack_fse_error),
            Err((
                debug_fse_error_kind(ErrorKind::NotFound),
                format!(
                    "Path '{}' not found or user lacks permission",
                    src_file.path().to_string_lossy()
                )
            ))
        );

        src_file.assert(predicates::path::missing());
    }

    #[test]
    fn overwrite_dir_with_file_error() {
        let src_file = NamedTempFile::new("a").unwrap();
        src_file.touch().unwrap();
        let dst_dir = TempDir::new().unwrap();

        assert_eq!(
            transfer_path(src_file.path(), dst_dir.path(), TransferMode::Copy)
                .map_err(unpack_fse_error),
            Err((
                debug_fse_error_kind(ErrorKind::Other),
                format!(
                    "Cannot to overwrite directory '{}' with file '{}'",
                    dst_dir.path().to_string_lossy(),
                    src_file.path().to_string_lossy()
                ),
            ))
        );

        src_file.assert(predicates::path::is_file());
        dst_dir.assert(predicates::path::is_dir());
    }

    #[test]
    fn overwrite_file_with_dir_error() {
        let src_dir = TempDir::new().unwrap();
        let dst_file = NamedTempFile::new("a").unwrap();
        dst_file.touch().unwrap();

        assert_eq!(
            transfer_path(src_dir.path(), dst_file.path(), TransferMode::Copy)
                .map_err(unpack_fse_error),
            Err((
                debug_fse_error_kind(ErrorKind::Other),
                format!(
                    "Cannot to overwrite file '{}' with directory '{}'",
                    dst_file.path().to_string_lossy(),
                    src_dir.path().to_string_lossy()
                ),
            ))
        );

        src_dir.assert(predicates::path::is_dir());
        dst_file.assert(predicates::path::is_file());
    }
}
