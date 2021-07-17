use std::fs;
use std::path::Path;

use fs_extra::error::{Error, ErrorKind, Result};
use fs_extra::{dir, file};
use lazy_static::lazy_static;
use same_file::is_same_file;

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

#[derive(Clone, Copy)]
pub enum TransferMode {
    Move,
    Copy,
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

        (FileType::File, dst_type) => {
            if let Some(dst_parent) = dst_path.parent() {
                dir::create_all(dst_parent, false)?;
            }
            match mode {
                TransferMode::Move => {
                    if fs::rename(src_path, dst_path).is_err() {
                        file::move_file(src_path, dst_path, &FILE_COPY_OPTIONS)?;
                    }
                }
                TransferMode::Copy => {
                    if dst_type == FileType::Unknown || !is_same_file(src_path, dst_path)? {
                        file::copy(src_path, dst_path, &FILE_COPY_OPTIONS)?;
                    }
                }
            }
            Ok(())
        }

        (FileType::Dir, dst_type) => {
            dir::create_all(dst_path, false)?;
            match mode {
                TransferMode::Move => {
                    if fs::rename(src_path, dst_path).is_err() {
                        dir::move_dir(src_path, dst_path, &DIR_COPY_OPTIONS)?;
                    }
                }
                TransferMode::Copy => {
                    if dst_type == FileType::Unknown || !is_same_file(src_path, dst_path)? {
                        dir::copy(src_path, dst_path, &DIR_COPY_OPTIONS)?;
                    }
                }
            }
            Ok(())
        }
    }
}

lazy_static! {
    pub static ref FILE_COPY_OPTIONS: file::CopyOptions = get_file_copy_options();
    pub static ref DIR_COPY_OPTIONS: dir::CopyOptions = get_dir_copy_options();
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

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::{NamedTempFile, TempDir};
    use fs_extra::error::ErrorKind;
    use ntest::*;
    use test_case::test_case;

    use super::*;
    use crate::transfer::testing::{debug_fse_error_kind, unpack_fse_error};

    #[test_case(temp_dir().path(),            FileType::Dir     ; "dir")]
    #[test_case(touch(temp_file("a")).path(), FileType::File    ; "file")]
    #[test_case(temp_file("b").path(),        FileType::Unknown ; "unknown")]
    fn file_type(path: &Path, file_type: FileType) {
        assert_eq!(FileType::from(path), file_type);
    }

    mod transfer_path {
        use super::*;

        #[test]
        fn path_not_found() {
            let src_file = temp_file("a");

            assert_eq!(
                transfer_path(src_file.path(), &Path::new("b"), TransferMode::Move) // Mode is irrelevant
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
        fn overwrite_dir_with_file() {
            let src_file = touch(temp_file("a"));
            let dst_dir = temp_dir();

            assert_eq!(
                transfer_path(src_file.path(), dst_dir.path(), TransferMode::Move) // Mode is irrelevant
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
        fn overwrite_file_with_dir() {
            let src_dir = temp_dir();
            let dst_file = touch(temp_file("a"));

            assert_eq!(
                transfer_path(src_dir.path(), dst_file.path(), TransferMode::Move) // Mode is irrelevant
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

        #[test]
        fn rename_file() {
            let src_file = write(temp_file("a"), "1");
            let dst_file = temp_file("b");

            assert_eq!(
                transfer_path(src_file.path(), dst_file.path(), TransferMode::Move)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_file.assert(predicates::path::missing());
            dst_file.assert("1");
        }

        #[test]
        fn rename_file_to_itself() {
            let src_file = write(temp_file("a"), "1");

            assert_eq!(
                transfer_path(src_file.path(), src_file.path(), TransferMode::Move)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_file.assert("1");
        }

        #[test]
        fn move_file_to_other() {
            let src_file = write(temp_file("a"), "1");
            let dst_file = write(temp_file("b"), "2");

            assert_eq!(
                transfer_path(src_file.path(), dst_file.path(), TransferMode::Move)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_file.assert(predicates::path::missing());
            dst_file.assert("1");
        }

        #[test]
        fn copy_file() {
            let src_file = write(temp_file("a"), "1");
            let dst_file = temp_file("b");

            assert_eq!(
                transfer_path(src_file.path(), dst_file.path(), TransferMode::Copy)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_file.assert("1");
            dst_file.assert("1");
        }

        #[test]
        #[timeout(5000)] // fs_extra::file::copy freezes for same src/dst path
        fn copy_file_to_itself() {
            let src_file = write(temp_file("a"), "1");

            assert_eq!(
                transfer_path(src_file.path(), src_file.path(), TransferMode::Copy)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_file.assert("1");
        }

        #[test]
        fn copy_file_to_other() {
            let src_file = write(temp_file("a"), "1");
            let dst_file = write(temp_file("b"), "2");

            assert_eq!(
                transfer_path(src_file.path(), dst_file.path(), TransferMode::Copy)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_file.assert("1");
            dst_file.assert("1");
        }

        #[test]
        fn rename_dir() {
            let root_dir = temp_dir();

            let src_dir = mkdir(root_dir.child("a"));
            let src_file = write(src_dir.child("c"), "1");

            let dst_dir = root_dir.child("b");
            let dst_file = dst_dir.child("c");

            assert_eq!(
                transfer_path(src_dir.path(), dst_dir.path(), TransferMode::Move)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_dir.assert(predicates::path::missing());
            src_file.assert(predicates::path::missing());

            dst_dir.assert(predicates::path::is_dir());
            dst_file.assert("1");
        }

        #[test]
        fn rename_dir_to_itself() {
            let src_dir = temp_dir();
            let src_file = write(src_dir.child("a"), "1");

            assert_eq!(
                transfer_path(src_dir.path(), src_dir.path(), TransferMode::Move)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_dir.assert(predicates::path::is_dir());
            src_file.assert("1");
        }

        #[test]
        fn move_dir_to_other() {
            let root_dir = temp_dir();

            let src_dir = mkdir(root_dir.child("a"));
            let src_file = write(src_dir.child("c"), "1");

            let dst_dir = mkdir(root_dir.child("b"));
            let dst_file = write(dst_dir.child("c"), "2");

            assert_eq!(
                transfer_path(src_dir.path(), dst_dir.path(), TransferMode::Move)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_dir.assert(predicates::path::missing());
            src_file.assert(predicates::path::missing());

            dst_dir.assert(predicates::path::is_dir());
            dst_file.assert("1");
        }

        #[test]
        fn copy_dir() {
            let root_dir = temp_dir();

            let src_dir = mkdir(root_dir.child("a"));
            let src_file = write(src_dir.child("c"), "1");

            let dst_dir = root_dir.child("b");
            let dst_file = dst_dir.child("c");

            assert_eq!(
                transfer_path(src_dir.path(), dst_dir.path(), TransferMode::Copy)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_dir.assert(predicates::path::is_dir());
            src_file.assert("1");

            dst_dir.assert(predicates::path::is_dir());
            dst_file.assert("1");
        }

        #[test]
        #[timeout(5000)] // fs_extra::dir::copy freezes for same src/dst path
        fn copy_dir_to_itself() {
            let src_dir = temp_dir();
            let src_file = write(src_dir.child("a"), "1");

            assert_eq!(
                transfer_path(src_dir.path(), src_dir.path(), TransferMode::Copy)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_dir.assert(predicates::path::is_dir());
            src_file.assert("1");
        }

        #[test]
        fn copy_dir_to_other() {
            let root_dir = temp_dir();

            let src_dir = mkdir(root_dir.child("a"));
            let src_file = write(src_dir.child("c"), "1");

            let dst_dir = mkdir(root_dir.child("b"));
            let dst_file = write(dst_dir.child("c"), "2");

            assert_eq!(
                transfer_path(src_dir.path(), dst_dir.path(), TransferMode::Copy)
                    .map_err(unpack_fse_error),
                Ok(())
            );

            src_dir.assert(predicates::path::is_dir());
            src_file.assert("1");

            dst_dir.assert(predicates::path::is_dir());
            dst_file.assert("1");
        }
    }

    #[test]
    fn same_dir_and_file_copy_options() {
        assert_eq!(DIR_COPY_OPTIONS.overwrite, FILE_COPY_OPTIONS.overwrite);
        assert_eq!(DIR_COPY_OPTIONS.skip_exist, FILE_COPY_OPTIONS.skip_exist);
        assert_eq!(DIR_COPY_OPTIONS.buffer_size, FILE_COPY_OPTIONS.buffer_size);
    }

    fn temp_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    fn temp_file(name: &str) -> NamedTempFile {
        NamedTempFile::new(name).unwrap()
    }

    fn mkdir<P: PathCreateDir>(path: P) -> P {
        path.create_dir_all().unwrap();
        path
    }

    fn touch<F: FileTouch>(file: F) -> F {
        file.touch().unwrap();
        file
    }

    fn write<F: FileWriteStr>(file: F, data: &str) -> F {
        file.write_str(data).unwrap();
        file
    }
}
