use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::{NamedTempFile, TempDir};

    #[test]
    fn file_type_file() {
        let file = NamedTempFile::new("a").unwrap();
        file.touch().unwrap();
        assert_eq!(FileType::from(file.path()), FileType::File);
    }

    #[test]
    fn file_type_dir() {
        assert_eq!(
            FileType::from(TempDir::new().unwrap().path()),
            FileType::Dir
        );
    }

    #[test]
    fn file_type_unknown() {
        assert_eq!(
            FileType::from(NamedTempFile::new("a").unwrap().path()),
            FileType::Unknown
        );
    }
}
