use crate::pattern::eval::ErrorKind;
use crate::pattern::filter::error::Result;
use crate::utils::AnyString;
use normpath::PathExt;
use std::ffi::OsStr;
use std::path::Path;

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
    if let Some(str) = value.as_ref().to_str() {
        Ok(str.to_string())
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
    fn absolute_from_absolute() {
        assert_eq!(
            get_absolute(String::from(make_absolute_path_str()), &Path::new("root")),
            Ok(String::from(make_absolute_path_str()))
        );
    }

    #[test]
    fn absolute_from_empty() {
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
    fn canonical_from_empty() {
        assert_eq!(
            get_canonical(String::new(), &Path::new("root")),
            Err(ErrorKind::CanonicalizationFailed(AnyString(String::from(
                "This string is not compared by assertion"
            ))))
        );
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
    fn parent_path_from_empty() {
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
    fn file_name_from_empty() {
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
    fn base_name_from_empty() {
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
    fn base_name_with_path_from_empty() {
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
    fn extension_from_empty() {
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
    fn extension_with_dot_from_empty() {
        assert_eq!(get_extension_with_dot(String::new()), Ok(String::new()));
    }

    #[test]
    fn to_string_utf8_error() {
        assert_eq!(
            to_string(make_non_utf8_os_str()),
            Err(ErrorKind::InputNotUtf8)
        )
    }

    #[cfg(any(unix))]
    fn make_absolute_path_str() -> &'static str {
        "/root/parent/file.ext"
    }

    #[cfg(windows)]
    fn make_absolute_path_str() -> &'static str {
        "C:/parent/file.ext"
    }
}
