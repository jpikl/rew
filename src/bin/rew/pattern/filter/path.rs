use crate::pattern::eval::ErrorKind;
use crate::pattern::filter::error::Result;
use crate::utils::AnyString;
use normpath::PathExt;
use std::ffi::OsStr;
use std::path::{Component, Path, MAIN_SEPARATOR};

pub fn get_absolute(value: String, current_dir: &Path) -> Result {
    if value.is_empty() {
        to_string(current_dir)
    } else {
        let path = Path::new(&value);
        if path.is_absolute() {
            Ok(value)
        } else {
            to_string(&current_dir.join(path))
        }
    }
}

pub fn get_canonical(value: String, current_dir: &Path) -> Result {
    let absolute_value = get_absolute(value, current_dir)?;
    let absolute_path = Path::new(&absolute_value);
    match absolute_path.normalize() {
        Ok(path_buf) => to_string(&path_buf),
        Err(error) => Err(ErrorKind::CanonicalizationFailed(AnyString(
            error.to_string(),
        ))),
    }
}

pub fn get_normalized(value: String) -> Result {
    let mut normalized_components = Vec::new();

    for component in Path::new(&value).components() {
        match &component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized_components.push(component);
            }
            Component::ParentDir => match normalized_components.last() {
                None | Some(Component::ParentDir) => normalized_components.push(component), // Keep '..', path is relative
                Some(Component::Normal(_)) => {
                    normalized_components.pop(); // Drop previous directory name
                }
                _ => {} // Drop '..', path is absolute
            },
            Component::CurDir => {} // Drop redundant '.'
        }
    }

    let mut normalized_value = String::new();
    let mut name_added = false;

    for component in normalized_components {
        match &component {
            Component::Prefix(_) | Component::RootDir => {
                normalized_value.push_str(to_str(&component)?);
            }
            Component::Normal(_) | Component::ParentDir => {
                if name_added {
                    normalized_value.push(MAIN_SEPARATOR);
                } else {
                    name_added = true;
                }
                normalized_value.push_str(to_str(&component)?);
            }
            Component::CurDir => {
                panic!(
                    "'{}' component should have been filtered out during path normalization",
                    component.as_os_str().to_string_lossy()
                );
            }
        }
    }

    if normalized_value.is_empty() {
        normalized_value.push_str(to_str(&Component::CurDir)?); // Bring back '.'
    }

    Ok(normalized_value)
}

pub fn get_parent_path(value: String) -> Result {
    opt_to_string(Path::new(&value).parent())
}

pub fn get_file_name(value: String) -> Result {
    opt_to_string(Path::new(&value).file_name())
}

pub fn get_base_name(value: String) -> Result {
    opt_to_string(Path::new(&value).file_stem())
}

pub fn get_base_name_with_path(mut value: String) -> Result {
    if let Some(extension_len) = Path::new(&value).extension().map(OsStr::len) {
        value.replace_range((value.len() - extension_len - 1).., "");
    }
    Ok(value)
}

pub fn get_extension(value: String) -> Result {
    opt_to_string(Path::new(&value).extension())
}

pub fn get_extension_with_dot(value: String) -> Result {
    let mut result = get_extension(value)?;
    if !result.is_empty() {
        result.insert(0, '.');
    }
    Ok(result)
}

fn opt_to_string<S: AsRef<OsStr> + ?Sized>(value: Option<&S>) -> Result {
    if let Some(value) = value {
        to_string(value)
    } else {
        Ok(String::new())
    }
}

fn to_string<S: AsRef<OsStr> + ?Sized>(value: &S) -> Result {
    to_str(value).map(str::to_string)
}

fn to_str<S: AsRef<OsStr> + ?Sized>(value: &S) -> std::result::Result<&str, ErrorKind> {
    if let Some(str) = value.as_ref().to_str() {
        Ok(str)
    } else {
        Err(ErrorKind::InputNotUtf8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::make_non_utf8_os_str;
    use std::path::Path;

    #[test]
    fn absolute_from_relative() {
        assert_eq!(
            get_absolute(String::from("parent/file.ext"), &Path::new("root")),
            Ok(String::from("root/parent/file.ext"))
        );
    }

    #[test]
    fn absolute_empty() {
        assert_eq!(
            get_absolute(String::new(), &Path::new("root")),
            Ok(String::from("root"))
        );
    }

    #[test]
    fn canonical() {
        let current_dir = std::env::current_dir().unwrap();
        assert_eq!(
            get_canonical(String::from("Cargo.toml"), &current_dir),
            Ok(current_dir.join("Cargo.toml").to_str().unwrap().to_string())
        );
    }

    #[test]
    fn canonical_empty() {
        assert_eq!(
            get_canonical(String::new(), &Path::new("root")),
            Err(ErrorKind::CanonicalizationFailed(AnyString(String::from(
                "This string is not compared by assertion"
            ))))
        );
    }

    #[test]
    fn normalized_empty() {
        assert_normalized("", ".");
    }

    fn assert_normalized(value: &str, result: &str) {
        assert_eq!(get_normalized(value.to_string()), Ok(result.to_string()));
    }

    #[test]
    fn parent_path() {
        assert_eq!(
            get_parent_path(String::from("root/parent/file.ext")),
            Ok(String::from("root/parent"))
        );
    }

    #[test]
    fn parent_path_missing() {
        assert_eq!(get_parent_path(String::from("file.ext")), Ok(String::new()));
    }

    #[test]
    fn parent_path_empty() {
        assert_eq!(get_parent_path(String::new()), Ok(String::new()));
    }

    #[test]
    fn file_name() {
        assert_eq!(
            get_file_name(String::from("root/parent/file.ext")),
            Ok(String::from("file.ext"))
        );
    }

    #[test]
    fn file_name_empty() {
        assert_eq!(get_file_name(String::new()), Ok(String::new()));
    }

    #[test]
    fn base_name() {
        assert_eq!(
            get_base_name(String::from("root/parent/file.ext")),
            Ok(String::from("file"))
        );
    }

    #[test]
    fn base_name_extension_missing() {
        assert_eq!(
            get_base_name(String::from("root/parent/file")),
            Ok(String::from("file"))
        );
    }

    #[test]
    fn base_name_empty() {
        assert_eq!(get_base_name(String::new()), Ok(String::new()));
    }

    #[test]
    fn base_name_with_path() {
        assert_eq!(
            get_base_name_with_path(String::from("root/parent/file.ext")),
            Ok(String::from("root/parent/file"))
        );
    }

    #[test]
    fn base_name_with_path_extension_missing() {
        assert_eq!(
            get_base_name_with_path(String::from("root/parent/file")),
            Ok(String::from("root/parent/file"))
        );
    }

    #[test]
    fn base_name_with_path_empty() {
        assert_eq!(get_base_name_with_path(String::new()), Ok(String::new()));
    }

    #[test]
    fn extension() {
        assert_eq!(
            get_extension(String::from("root/parent/file.ext")),
            Ok(String::from("ext"))
        );
    }

    #[test]
    fn extension_missing() {
        assert_eq!(
            get_extension(String::from("root/parent/file")),
            Ok(String::new())
        );
    }

    #[test]
    fn extension_empty() {
        assert_eq!(get_extension(String::new()), Ok(String::new()));
    }

    #[test]
    fn extension_with_dot() {
        assert_eq!(
            get_extension_with_dot(String::from("root/parent/file.ext")),
            Ok(String::from(".ext"))
        );
    }

    #[test]
    fn extension_with_dot_missing() {
        assert_eq!(
            get_extension_with_dot(String::from("root/parent/file")),
            Ok(String::new())
        );
    }

    #[test]
    fn extension_with_dot_empty() {
        assert_eq!(get_extension_with_dot(String::new()), Ok(String::new()));
    }

    #[test]
    fn to_str_utf8_error() {
        assert_eq!(to_str(make_non_utf8_os_str()), Err(ErrorKind::InputNotUtf8))
    }

    #[cfg(unix)]
    mod unix {
        use super::*;

        #[test]
        fn absolute_from_absolute() {
            assert_eq!(
                get_absolute(String::from("/root/parent/file.ext"), &Path::new("ignored")),
                Ok(String::from("/root/parent/file.ext"))
            );
        }

        #[test]
        fn normalized_relative_separator() {
            assert_normalized("abc", "abc");
            assert_normalized("abc/", "abc");
            assert_normalized("abc/def", "abc/def");
            assert_normalized("abc/def/", "abc/def");
            assert_normalized("abc//", "abc");
            assert_normalized("abc//def", "abc/def");
            assert_normalized("abc//def//", "abc/def");
        }

        #[test]
        fn normalized_relative_dot() {
            assert_normalized(".", ".");
            assert_normalized("./", ".");
            assert_normalized("./.", ".");
            assert_normalized("././", ".");
            assert_normalized("./abc", "abc");
            assert_normalized("./abc/", "abc");
            assert_normalized("abc/.", "abc");
            assert_normalized("abc/./", "abc");
        }

        #[test]
        fn normalized_relative_double_dot() {
            assert_normalized("..", "..");
            assert_normalized("../", "..");
            assert_normalized("../..", "../..");
            assert_normalized("../../", "../..");
            assert_normalized("../abc", "../abc");
            assert_normalized("../abc/", "../abc");
            assert_normalized("abc/..", ".");
            assert_normalized("abc/../", ".");
            assert_normalized("abc/../def", "def");
            assert_normalized("abc/../def/", "def");
            assert_normalized("abc/../def/ghi", "def/ghi");
            assert_normalized("abc/../def/ghi/", "def/ghi");
            assert_normalized("abc/../../ghi", "../ghi");
            assert_normalized("abc/../../ghi/", "../ghi");
            assert_normalized("abc/def/../../ghi", "ghi");
            assert_normalized("abc/def/../../ghi/", "ghi");
        }

        #[test]
        fn normalized_absolute_separator() {
            assert_normalized("/abc", "/abc");
            assert_normalized("/abc/", "/abc");
            assert_normalized("/abc/def", "/abc/def");
            assert_normalized("/abc/def/", "/abc/def");
            assert_normalized("//abc", "/abc");
            assert_normalized("//abc//", "/abc");
            assert_normalized("//abc//def", "/abc/def");
            assert_normalized("//abc//def//", "/abc/def");
        }

        #[test]
        fn normalized_absolute_dot() {
            assert_normalized("/.", "/");
            assert_normalized("/./", "/");
            assert_normalized("/./.", "/");
            assert_normalized("/././", "/");
            assert_normalized("/./abc", "/abc");
            assert_normalized("/./abc/", "/abc");
            assert_normalized("/abc/.", "/abc");
            assert_normalized("/abc/./", "/abc");
        }

        #[test]
        fn normalized_absolute_double_dot() {
            assert_normalized("/..", "/");
            assert_normalized("/../", "/");
            assert_normalized("/../..", "/");
            assert_normalized("/../../", "/");
            assert_normalized("/../abc", "/abc");
            assert_normalized("/../abc/", "/abc");
            assert_normalized("/abc/..", "/");
            assert_normalized("/abc/../", "/");
            assert_normalized("/abc/../def", "/def");
            assert_normalized("/abc/../def/", "/def");
            assert_normalized("/abc/../def/ghi", "/def/ghi");
            assert_normalized("/abc/../def/ghi/", "/def/ghi");
            assert_normalized("/abc/../../ghi", "/ghi");
            assert_normalized("/abc/../../ghi/", "/ghi");
            assert_normalized("/abc/def/../../ghi", "/ghi");
            assert_normalized("/abc/def/../../ghi/", "/ghi");
        }
    }

    #[cfg(windows)]
    mod windows {
        use super::*;

        #[test]
        fn absolute_from_absolute() {
            assert_eq!(
                get_absolute(String::from("C:\\parent\\file.ext"), &Path::new("ignored")),
                Ok(String::from("C:\\parent\\file.ext"))
            );
        }

        #[test]
        fn normalized_relative_separator() {
            assert_normalized("abc", "abc");
            assert_normalized("abc\\", "abc");
            assert_normalized("abc\\def", "abc\\def");
            assert_normalized("abc\\def\\", "abc\\def");
            assert_normalized("abc\\\\", "abc");
            assert_normalized("abc\\\\def", "abc\\def");
            assert_normalized("abc\\\\def\\\\", "abc\\def");
        }

        #[test]
        fn normalized_relative_forward_slashes() {
            assert_normalized("abc", "abc");
            assert_normalized("abc/", "abc");
            assert_normalized("abc/def", "abc\\def");
            assert_normalized("abc/def/", "abc\\def");
        }

        #[test]
        fn normalized_relative_dot() {
            assert_normalized(".", ".");
            assert_normalized(".\\", ".");
            assert_normalized(".\\.", ".");
            assert_normalized(".\\.\\", ".");
            assert_normalized(".\\abc", "abc");
            assert_normalized(".\\abc\\", "abc");
            assert_normalized("abc\\.", "abc");
            assert_normalized("abc\\.\\", "abc");
        }

        #[test]
        fn normalized_relative_double_dot() {
            assert_normalized("..", "..");
            assert_normalized("..\\", "..");
            assert_normalized("..\\..", "..\\..");
            assert_normalized("..\\..\\", "..\\..");
            assert_normalized("..\\abc", "..\\abc");
            assert_normalized("..\\abc\\", "..\\abc");
            assert_normalized("abc\\..", ".");
            assert_normalized("abc\\..\\", ".");
            assert_normalized("abc\\..\\def", "def");
            assert_normalized("abc\\..\\def\\", "def");
            assert_normalized("abc\\..\\def\\ghi", "def\\ghi");
            assert_normalized("abc\\..\\def\\ghi\\", "def\\ghi");
            assert_normalized("abc\\..\\..\\ghi", "..\\ghi");
            assert_normalized("abc\\..\\..\\ghi\\", "..\\ghi");
            assert_normalized("abc\\def\\..\\..\\ghi", "ghi");
            assert_normalized("abc\\def\\..\\..\\ghi\\", "ghi");
        }

        #[test]
        fn normalized_absolute_separator() {
            assert_normalized("C:\\abc", "C:\\abc");
            assert_normalized("C:\\abc\\", "C:\\abc");
            assert_normalized("C:\\abc\\def", "C:\\abc\\def");
            assert_normalized("C:\\abc\\def\\", "C:\\abc\\def");
            assert_normalized("C:\\\\abc", "C:\\abc");
            assert_normalized("C:\\\\abc\\\\", "C:\\abc");
            assert_normalized("C:\\\\abc\\\\def", "C:\\abc\\def");
            assert_normalized("C:\\\\abc\\\\def\\\\", "C:\\abc\\def");
        }

        #[test]
        fn normalized_absolute_forward_slashes() {
            assert_normalized("C:/abc", "C:\\abc");
            assert_normalized("C:/abc/", "C:\\abc");
            assert_normalized("C:/abc/def", "C:\\abc\\def");
            assert_normalized("C:/abc/def/", "C:\\abc\\def");
        }

        #[test]
        fn normalized_absolute_dot() {
            assert_normalized("C:\\.", "C:\\");
            assert_normalized("C:\\.\\", "C:\\");
            assert_normalized("C:\\.\\.", "C:\\");
            assert_normalized("C:\\.\\.\\", "C:\\");
            assert_normalized("C:\\.\\abc", "C:\\abc");
            assert_normalized("C:\\.\\abc\\", "C:\\abc");
            assert_normalized("C:\\abc\\.", "C:\\abc");
            assert_normalized("C:\\abc\\.\\", "C:\\abc");
        }

        #[test]
        fn normalized_absolute_double_dot() {
            assert_normalized("C:\\..", "C:\\");
            assert_normalized("C:\\..\\", "C:\\");
            assert_normalized("C:\\..\\..", "C:\\");
            assert_normalized("C:\\..\\..\\", "C:\\");
            assert_normalized("C:\\..\\abc", "C:\\abc");
            assert_normalized("C:\\..\\abc\\", "C:\\abc");
            assert_normalized("C:\\abc\\..", "C:\\");
            assert_normalized("C:\\abc\\..\\", "C:\\");
            assert_normalized("C:\\abc\\..\\def", "C:\\def");
            assert_normalized("C:\\abc\\..\\def\\", "C:\\def");
            assert_normalized("C:\\abc\\..\\def\\ghi", "C:\\def\\ghi");
            assert_normalized("C:\\abc\\..\\def\\ghi\\", "C:\\def\\ghi");
            assert_normalized("C:\\abc\\..\\..\\ghi", "C:\\ghi");
            assert_normalized("C:\\abc\\..\\..\\ghi\\", "C:\\ghi");
            assert_normalized("C:\\abc\\def\\..\\..\\ghi", "C:\\ghi");
            assert_normalized("C:\\abc\\def\\..\\..\\ghi\\", "C:\\ghi");
        }
    }
}