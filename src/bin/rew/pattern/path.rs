use crate::pattern::eval::ErrorKind;
use crate::utils::AnyString;
use normpath::PathExt;
use pathdiff::diff_paths;
use std::ffi::OsStr;
use std::path::{is_separator, Component, Path, PathBuf, MAIN_SEPARATOR};

type Result = std::result::Result<String, ErrorKind>;

pub fn to_absolute(value: String, working_dir: &Path) -> Result {
    if value.is_empty() {
        to_string(working_dir)
    } else {
        let path = Path::new(&value);
        if path.is_absolute() {
            Ok(value)
        } else {
            into_string(working_dir.join(path))
        }
    }
}

pub fn to_relative(value: String, working_dir: &Path) -> Result {
    let path = Path::new(&value);
    if path.is_relative() {
        Ok(value)
    } else {
        into_string(diff_paths(path, working_dir).unwrap_or_default())
    }
}

pub fn canonicalize(value: String, working_dir: &Path) -> Result {
    let absolute_value = to_absolute(value, working_dir)?;
    let absolute_path = Path::new(&absolute_value);

    match absolute_path.normalize() {
        Ok(result) => into_string(result.into_path_buf()).map(|mut result| {
            // Normalize unix vs windows behaviour
            if cfg!(windows)
                && result.ends_with(MAIN_SEPARATOR)
                && !matches!(
                    Path::new(&result).components().last(),
                    Some(Component::RootDir)
                )
            {
                result.pop();
            }
            result
        }),
        Err(error) => Err(ErrorKind::CanonicalizationFailed(AnyString(
            error.to_string(),
        ))),
    }
}

pub fn normalize(value: &str) -> Result {
    let mut normalized_components = Vec::new();

    for component in Path::new(value).components() {
        match &component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized_components.push(component);
            }
            Component::ParentDir => match normalized_components.last() {
                None | Some(Component::ParentDir) => normalized_components.push(component),
                Some(Component::Normal(_)) => {
                    normalized_components.pop();
                }
                _ => {}
            },
            Component::CurDir => {}
        }
    }

    let mut normalized_value = String::new();
    let mut name_added = false;

    if let Some(Component::Prefix(_)) = normalized_components.last() {
        normalized_components.push(Component::RootDir);
    }

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
        normalized_value.push_str(to_str(&Component::CurDir)?);
    }

    Ok(normalized_value)
}

pub fn get_parent_directory(value: String) -> Result {
    let path = Path::new(&value);
    match path.components().last() {
        Some(Component::Prefix(_)) => into_string(PathBuf::from(value).join(Component::RootDir)),
        Some(Component::RootDir) => Ok(value),
        Some(Component::CurDir) | Some(Component::ParentDir) | None => {
            into_string(PathBuf::from(value).join(Component::ParentDir))
        }
        Some(Component::Normal(_)) => {
            let parent = path.parent().unwrap_or_else(|| Path::new(""));
            if parent.components().count() > 0 {
                to_string(parent)
            } else {
                to_string(&Component::CurDir)
            }
        }
    }
}

pub fn remove_last_name(value: String) -> Result {
    if let Some(parent) = Path::new(&value).parent() {
        to_string(parent)
    } else {
        Ok(value)
    }
}

pub fn get_file_name(value: &str) -> Result {
    to_string(Path::new(value).file_name().unwrap_or_default())
}

pub fn get_last_name(value: &str) -> Result {
    match Path::new(value).components().last() {
        Some(component @ Component::Normal(_))
        | Some(component @ Component::CurDir)
        | Some(component @ Component::ParentDir) => to_string(&component),
        _ => Ok(String::new()),
    }
}

pub fn get_base_name(value: &str) -> Result {
    to_string(Path::new(value).file_stem().unwrap_or_default())
}

pub fn remove_extension(mut value: String) -> Result {
    if let Some(extension_len) = Path::new(&value).extension().map(OsStr::len) {
        value.replace_range((value.len() - extension_len - 1).., "");
    }
    Ok(value)
}

pub fn get_extension(value: &str) -> Result {
    to_string(Path::new(value).extension().unwrap_or_default())
}

pub fn get_extension_with_dot(value: &str) -> Result {
    let mut result = get_extension(value)?;
    if !result.is_empty() {
        result.insert(0, '.');
    }
    Ok(result)
}

pub fn ensure_trailing_separator(mut value: String) -> String {
    match value.chars().last() {
        Some(last_char) if is_separator(last_char) => {
            if last_char != MAIN_SEPARATOR {
                value.pop();
                value.push(MAIN_SEPARATOR);
            }
        }
        _ => {
            value.push(MAIN_SEPARATOR);
        }
    }
    value
}

pub fn remove_trailing_separator(mut value: String) -> String {
    if let Some(last_char) = value.chars().last() {
        if std::path::is_separator(last_char) {
            value.pop();
        }
    }
    value
}

pub fn into_string(value: PathBuf) -> Result {
    match value.into_os_string().into_string() {
        Ok(result) => Ok(result),
        Err(_) => Err(ErrorKind::InputNotUtf8),
    }
}

pub fn to_string<S: AsRef<OsStr> + ?Sized>(value: &S) -> Result {
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

    mod to_absolute {
        use super::*;

        #[test]
        fn empty() {
            let working_dir = std::env::current_dir().unwrap();
            assert_eq!(
                to_absolute(String::new(), &working_dir),
                Ok(working_dir.to_str().unwrap().to_string())
            );
        }

        #[test]
        fn relative() {
            let working_dir = std::env::current_dir().unwrap();
            #[cfg(unix)]
            assert_eq!(
                to_absolute(String::from("file.ext"), &working_dir),
                Ok(format!("{}/file.ext", working_dir.to_str().unwrap()))
            );
            #[cfg(windows)]
            assert_eq!(
                to_absolute(String::from("file.ext"), &working_dir),
                Ok(format!("{}\\file.ext", working_dir.to_str().unwrap()))
            );
        }

        #[test]
        fn absolute() {
            let working_dir = std::env::current_dir().unwrap();
            #[cfg(unix)]
            assert_eq!(
                to_absolute(String::from("/file.ext"), &working_dir),
                Ok(String::from("/file.ext"))
            );
            #[cfg(windows)]
            assert_eq!(
                to_absolute(String::from("C:\\file.ext"), &working_dir),
                Ok(String::from("C:\\file.ext"))
            );
        }
    }

    mod to_relative {
        use super::*;

        #[test]
        fn empty() {
            let working_dir = std::env::current_dir().unwrap();
            assert_eq!(to_relative(String::new(), &working_dir), Ok(String::new()));
        }

        #[test]
        fn relative() {
            let working_dir = std::env::current_dir().unwrap();
            assert_eq!(
                to_relative(String::from("file.ext"), &working_dir),
                Ok(String::from("file.ext"))
            );
        }

        #[test]
        fn absolute() {
            let working_dir = std::env::current_dir().unwrap();
            let value = working_dir
                .join("..")
                .join("file.ext")
                .to_str()
                .unwrap()
                .to_string();

            #[cfg(unix)]
            assert_eq!(
                to_relative(value, &working_dir),
                Ok(String::from("../file.ext"))
            );
            #[cfg(windows)]
            assert_eq!(
                to_relative(value, &working_dir),
                Ok(String::from("..\\file.ext"))
            );
        }
    }

    mod canonicalize {
        use super::*;

        #[test]
        fn empty() {
            let working_dir = std::env::current_dir().unwrap();
            assert_eq!(
                canonicalize(String::new(), &working_dir),
                Ok(working_dir.to_str().unwrap().to_string())
            );
        }

        #[test]
        fn non_existent() {
            let working_dir = std::env::current_dir().unwrap();
            assert_eq!(
                canonicalize(String::from("non-existent"), &working_dir),
                Err(ErrorKind::CanonicalizationFailed(AnyString(String::from(
                    "This string is not compared by assertion"
                ))))
            );
        }

        #[test]
        fn existent() {
            let working_dir = std::env::current_dir().unwrap();
            #[cfg(unix)]
            assert_eq!(
                canonicalize(String::from("src/"), &working_dir),
                Ok(format!("{}/src", working_dir.to_str().unwrap(),))
            );
            #[cfg(windows)]
            assert_eq!(
                canonicalize(String::from("src\\"), &working_dir),
                Ok(format!("{}\\src", working_dir.to_str().unwrap()))
            );
        }

        #[test]
        fn root() {
            let working_dir = std::env::current_dir().unwrap();
            #[cfg(unix)]
            assert_eq!(
                canonicalize(String::from("/"), &working_dir),
                Ok(String::from("/"))
            );
            #[cfg(windows)]
            assert_eq!(
                canonicalize(String::from("C:\\"), &working_dir),
                Ok(String::from("C:\\"))
            );
        }

        #[test]
        #[cfg(windows)]
        fn prefix() {
            let working_dir = std::env::current_dir().unwrap();
            assert_eq!(
                canonicalize(String::from("C:"), &working_dir),
                Ok(String::from("C:\\"))
            );
        }
    }

    mod normalize {
        use super::*;

        #[test]
        fn empty() {
            assert_normalized("", ".");
        }

        #[cfg(unix)]
        mod unix {
            use super::*;

            #[test]
            fn relative_separator() {
                assert_normalized("abc", "abc");
                assert_normalized("abc/", "abc");
                assert_normalized("abc/def", "abc/def");
                assert_normalized("abc/def/", "abc/def");
                assert_normalized("abc//", "abc");
                assert_normalized("abc//def", "abc/def");
                assert_normalized("abc//def//", "abc/def");
            }

            #[test]
            fn relative_dot() {
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
            fn relative_double_dot() {
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
            fn absolute_separator() {
                assert_normalized("/", "/");
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
            fn absolute_dot() {
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
            fn absolute_double_dot() {
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
            fn relative_separator() {
                assert_normalized("abc", "abc");
                assert_normalized("abc\\", "abc");
                assert_normalized("abc\\def", "abc\\def");
                assert_normalized("abc\\def\\", "abc\\def");
                assert_normalized("abc\\\\", "abc");
                assert_normalized("abc\\\\def", "abc\\def");
                assert_normalized("abc\\\\def\\\\", "abc\\def");
            }

            #[test]
            fn relative_forward_slashes() {
                assert_normalized("abc", "abc");
                assert_normalized("abc/", "abc");
                assert_normalized("abc/def", "abc\\def");
                assert_normalized("abc/def/", "abc\\def");
            }

            #[test]
            fn relative_dot() {
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
            fn relative_double_dot() {
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
            fn absolute_separator() {
                assert_normalized("C:", "C:\\");
                assert_normalized("C:\\", "C:\\");
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
            fn absolute_forward_slashes() {
                assert_normalized("C:/abc", "C:\\abc");
                assert_normalized("C:/abc/", "C:\\abc");
                assert_normalized("C:/abc/def", "C:\\abc\\def");
                assert_normalized("C:/abc/def/", "C:\\abc\\def");
            }

            #[test]
            fn absolute_dot() {
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
            fn absolute_double_dot() {
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

        fn assert_normalized(value: &str, result: &str) {
            assert_eq!(normalize(value), Ok(result.to_string()));
        }
    }

    mod get_parent_directory {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(get_parent_directory(String::new()), Ok(String::from("..")));
        }

        #[test]
        fn name() {
            assert_eq!(
                get_parent_directory(String::from("file.ext")),
                Ok(String::from("."))
            );
        }

        #[test]
        fn name_parent() {
            assert_eq!(
                get_parent_directory(String::from("dir/file.ext")),
                Ok(String::from("dir"))
            );
        }

        #[test]
        fn dot() {
            #[cfg(unix)]
            assert_eq!(
                get_parent_directory(String::from(".")),
                Ok(String::from("./.."))
            );
            #[cfg(windows)]
            assert_eq!(
                get_parent_directory(String::from(".")),
                Ok(String::from(".\\.."))
            );
        }

        #[test]
        fn dot_parent() {
            assert_eq!(
                get_parent_directory(String::from("./file.ext")),
                Ok(String::from("."))
            );
        }

        #[test]
        fn double_dot() {
            #[cfg(unix)]
            assert_eq!(
                get_parent_directory(String::from("..")),
                Ok(String::from("../.."))
            );
            #[cfg(windows)]
            assert_eq!(
                get_parent_directory(String::from("..")),
                Ok(String::from("..\\.."))
            );
        }

        #[test]
        fn double_dot_parent() {
            assert_eq!(
                get_parent_directory(String::from("../file.ext")),
                Ok(String::from(".."))
            );
        }

        #[test]
        fn root() {
            #[cfg(unix)]
            assert_eq!(
                get_parent_directory(String::from("/")),
                Ok(String::from("/"))
            );
            #[cfg(windows)]
            assert_eq!(
                get_parent_directory(String::from("C:\\")),
                Ok(String::from("C:\\"))
            );
        }

        #[test]
        fn root_parent() {
            #[cfg(unix)]
            assert_eq!(
                get_parent_directory(String::from("/file.ext")),
                Ok(String::from("/"))
            );
            #[cfg(windows)]
            assert_eq!(
                get_parent_directory(String::from("C:\\file.ext")),
                Ok(String::from("C:\\"))
            );
        }

        #[test]
        #[cfg(windows)]
        fn prefix() {
            assert_eq!(
                get_parent_directory(String::from("C:")),
                Ok(String::from("C:\\"))
            );
        }
    }

    mod remove_last_name {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(remove_last_name(String::new()), Ok(String::new()));
        }

        #[test]
        fn name() {
            assert_eq!(
                remove_last_name(String::from("file.ext")),
                Ok(String::new())
            );
        }

        #[test]
        fn name_parent() {
            assert_eq!(
                remove_last_name(String::from("dir/file.ext")),
                Ok(String::from("dir"))
            );
        }

        #[test]
        fn dot() {
            assert_eq!(remove_last_name(String::from(".")), Ok(String::new()));
        }

        #[test]
        fn dot_parent() {
            assert_eq!(
                remove_last_name(String::from("./file.ext")),
                Ok(String::from("."))
            );
        }

        #[test]
        fn double_dot() {
            assert_eq!(remove_last_name(String::from("..")), Ok(String::new()));
        }

        #[test]
        fn double_dot_parent() {
            assert_eq!(
                remove_last_name(String::from("../file.ext")),
                Ok(String::from(".."))
            );
        }

        #[test]
        fn root() {
            #[cfg(unix)]
            assert_eq!(remove_last_name(String::from("/")), Ok(String::from("/")));
            #[cfg(windows)]
            assert_eq!(
                remove_last_name(String::from("C:\\")),
                Ok(String::from("C:\\"))
            );
        }

        #[test]
        fn root_parent() {
            #[cfg(unix)]
            assert_eq!(
                remove_last_name(String::from("/file.ext")),
                Ok(String::from("/"))
            );
            #[cfg(windows)]
            assert_eq!(
                remove_last_name(String::from("C:\\file.ext")),
                Ok(String::from("C:\\"))
            );
        }

        #[test]
        #[cfg(windows)]
        fn prefix() {
            assert_eq!(remove_last_name(String::from("C:")), Ok(String::from("C:")));
        }
    }

    mod get_file_name {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(get_file_name(""), Ok(String::new()));
        }

        #[test]
        fn name() {
            assert_eq!(get_file_name("file.ext"), Ok(String::from("file.ext")));
        }

        #[test]
        fn name_parent() {
            assert_eq!(get_file_name("dir/file.ext"), Ok(String::from("file.ext")));
        }

        #[test]
        fn dot() {
            assert_eq!(get_file_name("."), Ok(String::new()));
        }

        #[test]
        fn dot_parent() {
            assert_eq!(get_file_name("./file.ext"), Ok(String::from("file.ext")));
        }

        #[test]
        fn double_dot() {
            assert_eq!(get_file_name(".."), Ok(String::new()));
        }

        #[test]
        fn double_dot_parent() {
            assert_eq!(get_file_name("../file.ext"), Ok(String::from("file.ext")));
        }

        #[test]
        fn root() {
            #[cfg(unix)]
            assert_eq!(get_file_name("/"), Ok(String::new()));
            #[cfg(windows)]
            assert_eq!(get_file_name("C:\\"), Ok(String::new()));
        }

        #[test]
        fn root_parent() {
            #[cfg(unix)]
            assert_eq!(get_file_name("/file.ext"), Ok(String::from("file.ext")));
            #[cfg(windows)]
            assert_eq!(get_file_name("C:\\file.ext"), Ok(String::from("file.ext")));
        }

        #[test]
        #[cfg(windows)]
        fn prefix() {
            assert_eq!(get_file_name("C:"), Ok(String::new()));
        }
    }

    mod get_last_name {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(get_last_name(""), Ok(String::new()));
        }

        #[test]
        fn name() {
            assert_eq!(get_last_name("file.ext"), Ok(String::from("file.ext")));
        }

        #[test]
        fn name_parent() {
            assert_eq!(get_last_name("dir/file.ext"), Ok(String::from("file.ext")));
        }

        #[test]
        fn dot() {
            assert_eq!(get_last_name("."), Ok(String::from(".")));
        }

        #[test]
        fn dot_parent() {
            assert_eq!(get_last_name("./file.ext"), Ok(String::from("file.ext")));
        }

        #[test]
        fn double_dot() {
            assert_eq!(get_last_name(".."), Ok(String::from("..")));
        }

        #[test]
        fn double_dot_parent() {
            assert_eq!(get_last_name("../file.ext"), Ok(String::from("file.ext")));
        }

        #[test]
        fn root() {
            #[cfg(unix)]
            assert_eq!(get_last_name("/"), Ok(String::new()));
            #[cfg(windows)]
            assert_eq!(get_last_name("C:\\"), Ok(String::new()));
        }

        #[test]
        fn root_parent() {
            #[cfg(unix)]
            assert_eq!(get_last_name("/file.ext"), Ok(String::from("file.ext")));
            #[cfg(windows)]
            assert_eq!(get_last_name("C:\\file.ext"), Ok(String::from("file.ext")));
        }

        #[test]
        #[cfg(windows)]
        fn prefix() {
            assert_eq!(get_last_name("C:"), Ok(String::new()));
        }
    }

    mod get_base_name {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(get_base_name(""), Ok(String::new()));
        }

        #[test]
        fn base() {
            assert_eq!(get_base_name("file"), Ok(String::from("file")));
        }

        #[test]
        fn name() {
            assert_eq!(get_base_name("file.ext"), Ok(String::from("file")));
        }

        #[test]
        fn name_parent() {
            assert_eq!(get_base_name("dir/file.ext"), Ok(String::from("file")));
        }

        #[test]
        fn dot() {
            assert_eq!(get_base_name("."), Ok(String::new()));
        }

        #[test]
        fn dot_parent() {
            assert_eq!(get_base_name("./file.ext"), Ok(String::from("file")));
        }

        #[test]
        fn double_dot() {
            assert_eq!(get_base_name(".."), Ok(String::new()));
        }

        #[test]
        fn double_dot_parent() {
            assert_eq!(get_base_name("../file.ext"), Ok(String::from("file")));
        }

        #[test]
        fn root() {
            #[cfg(unix)]
            assert_eq!(get_base_name("/"), Ok(String::new()));
            #[cfg(windows)]
            assert_eq!(get_base_name("C:\\"), Ok(String::new()));
        }

        #[test]
        fn root_parent() {
            #[cfg(unix)]
            assert_eq!(get_base_name("/file.ext"), Ok(String::from("file")));
            #[cfg(windows)]
            assert_eq!(get_base_name("C:\\file.ext"), Ok(String::from("file")));
        }

        #[test]
        #[cfg(windows)]
        fn prefix() {
            assert_eq!(get_base_name("C:"), Ok(String::new()));
        }
    }

    mod remove_extension {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(remove_extension(String::new()), Ok(String::new()));
        }

        #[test]
        fn base() {
            assert_eq!(
                remove_extension(String::from("file")),
                Ok(String::from("file"))
            );
        }

        #[test]
        fn name() {
            assert_eq!(
                remove_extension(String::from("file.ext")),
                Ok(String::from("file"))
            );
        }

        #[test]
        fn name_parent() {
            assert_eq!(
                remove_extension(String::from("dir/file.ext")),
                Ok(String::from("dir/file"))
            );
        }

        #[test]
        fn dot() {
            assert_eq!(remove_extension(String::from(".")), Ok(String::from(".")));
        }

        #[test]
        fn dot_parent() {
            assert_eq!(
                remove_extension(String::from("./file.ext")),
                Ok(String::from("./file"))
            );
        }

        #[test]
        fn double_dot() {
            assert_eq!(remove_extension(String::from("..")), Ok(String::from("..")));
        }

        #[test]
        fn double_dot_parent() {
            assert_eq!(
                remove_extension(String::from("../file.ext")),
                Ok(String::from("../file"))
            );
        }

        #[test]
        fn root() {
            #[cfg(unix)]
            assert_eq!(remove_extension(String::from("/")), Ok(String::from("/")));
            #[cfg(windows)]
            assert_eq!(
                remove_extension(String::from("C:\\")),
                Ok(String::from("C:\\"))
            );
        }

        #[test]
        fn root_parent() {
            #[cfg(unix)]
            assert_eq!(
                remove_extension(String::from("/file.ext")),
                Ok(String::from("/file"))
            );
            #[cfg(windows)]
            assert_eq!(
                remove_extension(String::from("C:\\file.ext")),
                Ok(String::from("C:\\file"))
            );
        }

        #[test]
        #[cfg(windows)]
        fn prefix() {
            assert_eq!(remove_extension(String::from("C:")), Ok(String::from("C:")));
        }
    }

    mod get_extension {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(get_extension(""), Ok(String::new()));
        }

        #[test]
        fn base() {
            assert_eq!(get_extension("file"), Ok(String::new()));
        }

        #[test]
        fn name() {
            assert_eq!(get_extension("file.ext"), Ok(String::from("ext")));
        }

        #[test]
        fn name_parent() {
            assert_eq!(get_extension("dir/file.ext"), Ok(String::from("ext")));
        }

        #[test]
        fn dot() {
            assert_eq!(get_extension("."), Ok(String::new()));
        }

        #[test]
        fn dot_parent() {
            assert_eq!(get_extension("./file.ext"), Ok(String::from("ext")));
        }

        #[test]
        fn double_dot() {
            assert_eq!(get_extension(".."), Ok(String::new()));
        }

        #[test]
        fn double_dot_parent() {
            assert_eq!(get_extension("../file.ext"), Ok(String::from("ext")));
        }

        #[cfg(unix)]
        mod unix {
            use super::*;

            #[test]
            fn root() {
                #[cfg(unix)]
                assert_eq!(get_extension("/"), Ok(String::new()));
                #[cfg(windows)]
                assert_eq!(get_extension("C:\\"), Ok(String::new()));
            }

            #[test]
            fn root_parent() {
                #[cfg(unix)]
                assert_eq!(get_extension("/file.ext"), Ok(String::from("ext")));
                #[cfg(windows)]
                assert_eq!(get_extension("C:\\file.ext"), Ok(String::from("ext")));
            }

            #[test]
            #[cfg(windows)]
            fn prefix() {
                assert_eq!(get_extension("C:"), Ok(String::new()));
            }
        }
    }

    mod get_extension_with_dot {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(get_extension_with_dot(""), Ok(String::new()));
        }

        #[test]
        fn base() {
            assert_eq!(get_extension_with_dot("file"), Ok(String::new()));
        }

        #[test]
        fn name() {
            assert_eq!(get_extension_with_dot("file.ext"), Ok(String::from(".ext")));
        }

        #[test]
        fn name_parent() {
            assert_eq!(
                get_extension_with_dot("dir/file.ext"),
                Ok(String::from(".ext"))
            );
        }

        #[test]
        fn dot() {
            assert_eq!(get_extension_with_dot("."), Ok(String::new()));
        }

        #[test]
        fn dot_parent() {
            assert_eq!(
                get_extension_with_dot("./file.ext"),
                Ok(String::from(".ext"))
            );
        }

        #[test]
        fn double_dot() {
            assert_eq!(get_extension_with_dot(".."), Ok(String::new()));
        }

        #[test]
        fn double_dot_parent() {
            assert_eq!(
                get_extension_with_dot("../file.ext"),
                Ok(String::from(".ext"))
            );
        }

        #[test]
        fn root() {
            #[cfg(unix)]
            assert_eq!(get_extension_with_dot("/"), Ok(String::new()));
            #[cfg(windows)]
            assert_eq!(get_extension_with_dot("C:\\"), Ok(String::new()));
        }

        #[test]
        fn root_parent() {
            #[cfg(unix)]
            assert_eq!(
                get_extension_with_dot("/file.ext"),
                Ok(String::from(".ext"))
            );
            #[cfg(windows)]
            assert_eq!(
                get_extension_with_dot("C:\\file.ext"),
                Ok(String::from(".ext"))
            );
        }

        #[test]
        #[cfg(windows)]
        fn prefix() {
            assert_eq!(get_extension_with_dot("C:"), Ok(String::new()));
        }
    }

    mod ensure_trailing_separator {
        use super::*;

        #[test]
        fn empty() {
            #[cfg(unix)]
            assert_eq!(ensure_trailing_separator(String::new()), String::from("/"));
            #[cfg(windows)]
            assert_eq!(ensure_trailing_separator(String::new()), String::from("\\"));
        }

        #[test]
        fn name() {
            #[cfg(unix)]
            assert_eq!(
                ensure_trailing_separator(String::from("dir")),
                String::from("dir/")
            );
            #[cfg(windows)]
            assert_eq!(
                ensure_trailing_separator(String::from("dir")),
                String::from("dir\\")
            );
        }

        #[test]
        fn name_separator() {
            #[cfg(unix)]
            assert_eq!(
                ensure_trailing_separator(String::from("dir/")),
                String::from("dir/")
            );
            #[cfg(windows)]
            assert_eq!(
                ensure_trailing_separator(String::from("dir\\")),
                String::from("dir\\")
            );
            #[cfg(windows)]
            assert_eq!(
                ensure_trailing_separator(String::from("dir/")),
                String::from("dir\\")
            );
        }

        #[test]
        fn root() {
            #[cfg(unix)]
            assert_eq!(
                ensure_trailing_separator(String::from("/")),
                String::from("/")
            );
            #[cfg(windows)]
            assert_eq!(
                ensure_trailing_separator(String::from("C:\\")),
                String::from("C:\\")
            );
            #[cfg(windows)]
            assert_eq!(
                ensure_trailing_separator(String::from("C:/")),
                String::from("C:\\")
            );
        }

        #[test]
        #[cfg(windows)]
        fn prefix() {
            assert_eq!(
                ensure_trailing_separator(String::from("C:")),
                String::from("C:\\")
            );
        }
    }

    mod remove_trailing_separator {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(remove_trailing_separator(String::new()), String::new());
        }

        #[test]
        fn name() {
            #[cfg(unix)]
            assert_eq!(
                remove_trailing_separator(String::from("dir")),
                String::from("dir")
            );
        }

        #[test]
        fn name_separator() {
            assert_eq!(
                remove_trailing_separator(String::from("dir/")),
                String::from("dir")
            );
            #[cfg(windows)]
            assert_eq!(
                remove_trailing_separator(String::from("dir\\")),
                String::from("dir")
            );
        }

        #[test]
        fn root() {
            #[cfg(unix)]
            assert_eq!(remove_trailing_separator(String::from("/")), String::new());
            #[cfg(windows)]
            assert_eq!(
                remove_trailing_separator(String::from("C:\\")),
                String::from("C:")
            );
            #[cfg(windows)]
            assert_eq!(
                remove_trailing_separator(String::from("C:/")),
                String::from("C:")
            );
        }

        #[test]
        #[cfg(windows)]
        fn prefix() {
            assert_eq!(
                remove_trailing_separator(String::from("C:")),
                String::from("C:")
            );
        }
    }

    mod into_string {
        use super::*;
        use crate::testing::make_non_utf8_os_string;

        #[test]
        fn utf8() {
            assert_eq!(into_string(PathBuf::from("abc")), Ok(String::from("abc")));
        }

        #[test]
        fn non_utf8() {
            assert_eq!(
                into_string(PathBuf::from(make_non_utf8_os_string())),
                Err(ErrorKind::InputNotUtf8)
            )
        }
    }

    mod to_str {
        use super::*;
        use crate::testing::make_non_utf8_os_string;

        #[test]
        fn utf8() {
            assert_eq!(to_str(OsStr::new("abc")), Ok("abc"));
        }

        #[test]
        fn non_utf8() {
            assert_eq!(
                to_str(&make_non_utf8_os_string()),
                Err(ErrorKind::InputNotUtf8)
            )
        }
    }
}
