use crate::pattern::eval::BaseResult;
use crate::pattern::eval::ErrorKind;
use crate::utils::AnyString;
use normpath::PathExt;
use pathdiff::diff_paths;
use std::ffi::OsStr;
use std::path::{is_separator, Component, Path, PathBuf, MAIN_SEPARATOR};

pub fn to_absolute(value: String, working_dir: &Path) -> BaseResult<String> {
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

pub fn to_relative(value: String, working_dir: &Path) -> BaseResult<String> {
    let path = Path::new(&value);
    if path.is_relative() {
        Ok(value)
    } else {
        into_string(diff_paths(path, working_dir).unwrap_or_default())
    }
}

pub fn canonicalize(value: String, working_dir: &Path) -> BaseResult<String> {
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

pub fn normalize(value: &str) -> BaseResult<String> {
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

pub fn get_parent_directory(value: String) -> BaseResult<String> {
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

pub fn remove_last_name(value: String) -> BaseResult<String> {
    if let Some(parent) = Path::new(&value).parent() {
        to_string(parent)
    } else {
        Ok(value)
    }
}

pub fn get_file_name(value: &str) -> BaseResult<String> {
    to_string(Path::new(value).file_name().unwrap_or_default())
}

pub fn get_last_name(value: &str) -> BaseResult<String> {
    match Path::new(value).components().last() {
        Some(component @ Component::Normal(_))
        | Some(component @ Component::CurDir)
        | Some(component @ Component::ParentDir) => to_string(&component),
        _ => Ok(String::new()),
    }
}

pub fn get_base_name(value: &str) -> BaseResult<String> {
    to_string(Path::new(value).file_stem().unwrap_or_default())
}

pub fn remove_extension(mut value: String) -> BaseResult<String> {
    if let Some(extension_len) = Path::new(&value).extension().map(OsStr::len) {
        value.replace_range((value.len() - extension_len - 1).., "");
    }
    Ok(value)
}

pub fn get_extension(value: &str) -> BaseResult<String> {
    to_string(Path::new(value).extension().unwrap_or_default())
}

pub fn get_extension_with_dot(value: &str) -> BaseResult<String> {
    let mut result = get_extension(value)?;
    if !result.is_empty() {
        result.insert(0, '.');
    }
    Ok(result)
}

pub fn ensure_trailing_dir_separator(mut value: String) -> String {
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

pub fn remove_trailing_dir_separator(mut value: String) -> String {
    if let Some(last_char) = value.chars().last() {
        if std::path::is_separator(last_char) {
            value.pop();
        }
    }
    value
}

pub fn into_string(value: PathBuf) -> BaseResult<String> {
    match value.into_os_string().into_string() {
        Ok(result) => Ok(result),
        Err(_) => Err(ErrorKind::InputNotUtf8),
    }
}

pub fn to_string<S: AsRef<OsStr> + ?Sized>(value: &S) -> BaseResult<String> {
    to_str(value).map(str::to_string)
}

fn to_str<S: AsRef<OsStr> + ?Sized>(value: &S) -> BaseResult<&str> {
    if let Some(str) = value.as_ref().to_str() {
        Ok(str)
    } else {
        Err(ErrorKind::InputNotUtf8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use test_case::test_case;

    #[test_case("", "{working_dir}"; "empty")]
    #[cfg_attr(unix, test_case("file.ext", "{working_dir}/file.ext"; "relative"))]
    #[cfg_attr(unix, test_case("/file.ext", "/file.ext"; "absolute"))]
    #[cfg_attr(window, test_case("file.ext", "{working_dir}\\file.ext"; "relative"))]
    #[cfg_attr(window, test_case("C:\\file.ext", "C:\\file.ext"; "absolute"))]
    fn to_absolute(input: &str, output: &str) {
        let working_dir = std::env::current_dir().unwrap();
        assert_eq!(
            super::to_absolute(input.into(), &working_dir),
            Ok(fmt_working_dir(output, &working_dir))
        );
    }

    #[test_case("", ""; "empty")]
    #[cfg_attr(unix, test_case("file.ext", "file.ext"; "relative" ))]
    #[cfg_attr(unix, test_case("{working_dir}/../file.ext", "../file.ext"; "absolute"))]
    #[cfg_attr(window, test_case("file.ext", "{working_dir}\\file.ext"; "relative"))]
    #[cfg_attr(window, test_case("{working_dir}\\file.ext", "..\\file.ext"; "absolute"))]
    fn to_relative(input: &str, output: &str) {
        let working_dir = std::env::current_dir().unwrap();
        assert_eq!(
            super::to_relative(fmt_working_dir(input, &working_dir), &working_dir),
            Ok(fmt_working_dir(output, &working_dir))
        );
    }

    #[test_case("", "{working_dir}"; "empty")]
    #[cfg_attr(unix, test_case("src/", "{working_dir}/src"; "existent"))]
    #[cfg_attr(unix, test_case("/", "/"; "root" ))]
    #[cfg_attr(windows, test_case("src\\", "{working_dir}\\src"; "existent"))]
    #[cfg_attr(windows, test_case("C:\\", "C:\\"; "root"))]
    #[cfg_attr(windows, test_case("C:", "C:\\"; "prefix"))]
    fn canonicalize(input: &str, output: &str) {
        let working_dir = std::env::current_dir().unwrap();
        assert_eq!(
            super::canonicalize(input.into(), &working_dir),
            Ok(fmt_working_dir(output, &working_dir))
        );
    }

    #[test]
    fn canonicalize_err() {
        let working_dir = std::env::current_dir().unwrap();
        assert_eq!(
            super::canonicalize("non-existent".into(), &working_dir),
            Err(ErrorKind::CanonicalizationFailed(AnyString::any()))
        );
    }

    #[test_case("", "."; "empty")]
    #[test_case("abc", "abc"; "relative separator 1")]
    #[cfg_attr(unix, test_case("abc/", "abc"; "relative separator 2"))]
    #[cfg_attr(unix, test_case("abc/def", "abc/def"; "relative separator 3"))]
    #[cfg_attr(unix, test_case("abc/def/", "abc/def"; "relative separator 4"))]
    #[cfg_attr(unix, test_case("abc//", "abc"; "relative separator 5"))]
    #[cfg_attr(unix, test_case("abc//def", "abc/def"; "relative separator 6"))]
    #[cfg_attr(unix, test_case("abc//def//", "abc/def"; "relative separator 7"))]
    #[cfg_attr(windows, test_case("abc\\", "abc"; "relative separator 2"))]
    #[cfg_attr(windows, test_case("abc\\def", "abc\\def"; "relative separator 3"))]
    #[cfg_attr(windows, test_case("abc\\def\\", "abc\\def"; "relative separator 4"))]
    #[cfg_attr(windows, test_case("abc\\\\", "abc"; "relative separator 5"))]
    #[cfg_attr(windows, test_case("abc\\\\def", "abc\\def"; "relative separator 6"))]
    #[cfg_attr(windows, test_case("abc\\\\def\\\\", "abc\\def"; "relative separator 7"))]
    #[cfg_attr(windows, test_case("abc/", "abc"; "relative unix separator 1"))]
    #[cfg_attr(windows, test_case("abc/def", "abc\\def"; "relative unix separator 2"))]
    #[cfg_attr(windows, test_case("abc/def/", "abc\\def"; "relative unix separator 3"))]
    #[test_case(".", "."; "relative dot 1")]
    #[cfg_attr(unix, test_case("./", "."; "relative dot 2"))]
    #[cfg_attr(unix, test_case("./.", "."; "relative dot 3"))]
    #[cfg_attr(unix, test_case("././", "."; "relative dot 4"))]
    #[cfg_attr(unix, test_case("./abc", "abc"; "relative dot 5"))]
    #[cfg_attr(unix, test_case("./abc/", "abc"; "relative dot 6"))]
    #[cfg_attr(unix, test_case("abc/.", "abc"; "relative dot 7"))]
    #[cfg_attr(unix, test_case("abc/./", "abc"; "relative dot 8"))]
    #[cfg_attr(windows, test_case(".\\", "."; "relative dot 2"))]
    #[cfg_attr(windows, test_case(".\\.", "."; "relative dot 3"))]
    #[cfg_attr(windows, test_case(".\\.\\", "."; "relative dot 4"))]
    #[cfg_attr(windows, test_case(".\\abc", "abc"; "relative dot 5"))]
    #[cfg_attr(windows, test_case(".\\abc\\", "abc"; "relative dot 6"))]
    #[cfg_attr(windows, test_case("abc\\.", "abc"; "relative dot 7"))]
    #[cfg_attr(windows, test_case("abc\\.\\", "abc"; "relative dot 8"))]
    #[test_case("..", ".."; "relative double dot 1")]
    #[cfg_attr(unix, test_case("../", ".."; "relative double dot 2"))]
    #[cfg_attr(unix, test_case("../..", "../.."; "relative double dot 3"))]
    #[cfg_attr(unix, test_case("../../", "../.."; "relative double dot 4"))]
    #[cfg_attr(unix, test_case("../abc", "../abc"; "relative double dot 5"))]
    #[cfg_attr(unix, test_case("../abc/", "../abc"; "relative double dot 6"))]
    #[cfg_attr(unix, test_case("abc/..", "."; "relative double dot 7"))]
    #[cfg_attr(unix, test_case("abc/../", "."; "relative double dot 8"))]
    #[cfg_attr(unix, test_case("abc/../def", "def"; "relative double dot 9"))]
    #[cfg_attr(unix, test_case("abc/../def/", "def"; "relative double dot 10"))]
    #[cfg_attr(unix, test_case("abc/../def/ghi", "def/ghi"; "relative double dot 11"))]
    #[cfg_attr(unix, test_case("abc/../def/ghi/", "def/ghi"; "relative double dot 12"))]
    #[cfg_attr(unix, test_case("abc/../../ghi", "../ghi"; "relative double dot 13"))]
    #[cfg_attr(unix, test_case("abc/../../ghi/", "../ghi"; "relative double dot 14"))]
    #[cfg_attr(unix, test_case("abc/def/../../ghi", "ghi"; "relative double dot 15"))]
    #[cfg_attr(unix, test_case("abc/def/../../ghi/", "ghi"; "relative double dot 16"))]
    #[cfg_attr(windows, test_case("..\\", ".."; "relative double dot 2"))]
    #[cfg_attr(windows, test_case("..\\..", "..\\.."; "relative double dot 3"))]
    #[cfg_attr(windows, test_case("..\\..\\", "..\\.."; "relative double dot 4"))]
    #[cfg_attr(windows, test_case("..\\abc", "..\\abc"; "relative double dot 5"))]
    #[cfg_attr(windows, test_case("..\\abc\\", "..\\abc"; "relative double dot 6"))]
    #[cfg_attr(windows, test_case("abc\\..", "."; "relative double dot 7"))]
    #[cfg_attr(windows, test_case("abc\\..\\", "."; "relative double dot 8"))]
    #[cfg_attr(windows, test_case("abc\\..\\def", "def"; "relative double dot 9"))]
    #[cfg_attr(windows, test_case("abc\\..\\def\\", "def"; "relative double dot 10"))]
    #[cfg_attr(windows, test_case("abc\\..\\def\\ghi", "def\\ghi"; "relative double dot 11"))]
    #[cfg_attr(windows, test_case("abc\\..\\def\\ghi\\", "def\\ghi"; "relative double dot 12"))]
    #[cfg_attr(windows, test_case("abc\\..\\..\\ghi", "..\\ghi"; "relative double dot 13"))]
    #[cfg_attr(windows, test_case("abc\\..\\..\\ghi\\", "..\\ghi"; "relative double dot 14"))]
    #[cfg_attr(windows, test_case("abc\\def\\..\\..\\ghi", "ghi"; "relative double dot 15"))]
    #[cfg_attr(windows, test_case("abc\\def\\..\\..\\ghi\\", "ghi"; "relative double dot 16"))]
    #[cfg_attr(unix, test_case("/", "/"; "absolute separator 1"))]
    #[cfg_attr(unix, test_case("/abc", "/abc"; "absolute separator 2"))]
    #[cfg_attr(unix, test_case("/abc/", "/abc"; "absolute separator 3"))]
    #[cfg_attr(unix, test_case("/abc/def", "/abc/def"; "absolute separator 4"))]
    #[cfg_attr(unix, test_case("/abc/def/", "/abc/def"; "absolute separator 5"))]
    #[cfg_attr(unix, test_case("//abc", "/abc"; "absolute separator 6"))]
    #[cfg_attr(unix, test_case("//abc//", "/abc"; "absolute separator 7"))]
    #[cfg_attr(unix, test_case("//abc//def", "/abc/def"; "absolute separator 8"))]
    #[cfg_attr(unix, test_case("//abc//def//", "/abc/def"; "absolute separator 9"))]
    #[cfg_attr(windows, test_case("C:", "C:\\"; "absolute separator 0"))]
    #[cfg_attr(windows, test_case("C:\\", "C:\\"; "absolute separator 1"))]
    #[cfg_attr(windows, test_case("C:\\abc", "C:\\abc"; "absolute separator 2"))]
    #[cfg_attr(windows, test_case("C:\\abc\\", "C:\\abc"; "absolute separator 3"))]
    #[cfg_attr(windows, test_case("C:\\abc\\def", "C:\\abc\\def"; "absolute separator 4"))]
    #[cfg_attr(windows, test_case("C:\\abc\\def\\", "C:\\abc\\def"; "absolute separator 5"))]
    #[cfg_attr(windows, test_case("C:\\\\abc", "C:\\abc"; "absolute separator 6"))]
    #[cfg_attr(windows, test_case("C:\\\\abc\\\\", "C:\\abc"; "absolute separator 7"))]
    #[cfg_attr(windows, test_case("C:\\\\abc\\\\def", "C:\\abc\\def"; "absolute separator 8"))]
    #[cfg_attr(windows, test_case("C:\\\\abc\\\\def\\\\", "C:\\abc\\def"; "absolute separator 9"))]
    #[cfg_attr(windows, test_case("C:/abc", "C:\\abc"; "absolute unix separator 1"))]
    #[cfg_attr(windows, test_case("C:/abc/", "C:\\abc"; "absolute unix separator 2"))]
    #[cfg_attr(windows, test_case("C:/abc/def", "C:\\abc\\def"; "absolute unix separator 3"))]
    #[cfg_attr(windows, test_case("C:/abc/def/", "C:\\abc\\def"; "absolute unix separator 4"))]
    #[cfg_attr(unix, test_case("/.", "/"; "absolute dot 1"))]
    #[cfg_attr(unix, test_case("/./", "/"; "absolute dot 2"))]
    #[cfg_attr(unix, test_case("/./.", "/"; "absolute dot 3"))]
    #[cfg_attr(unix, test_case("/././", "/"; "absolute dot 4"))]
    #[cfg_attr(unix, test_case("/./abc", "/abc"; "absolute dot 5"))]
    #[cfg_attr(unix, test_case("/./abc/", "/abc"; "absolute dot 6"))]
    #[cfg_attr(unix, test_case("/abc/.", "/abc"; "absolute dot 7"))]
    #[cfg_attr(unix, test_case("/abc/./", "/abc"; "absolute dot 8"))]
    #[cfg_attr(windows, test_case("C:\\.", "C:\\"; "absolute dot 1"))]
    #[cfg_attr(windows, test_case("C:\\.\\", "C:\\"; "absolute dot 2"))]
    #[cfg_attr(windows, test_case("C:\\.\\.", "C:\\"; "absolute dot 3"))]
    #[cfg_attr(windows, test_case("C:\\.\\.\\", "C:\\"; "absolute dot 4"))]
    #[cfg_attr(windows, test_case("C:\\.\\abc", "C:\\abc"; "absolute dot 5"))]
    #[cfg_attr(windows, test_case("C:\\.\\abc\\", "C:\\abc"; "absolute dot 6"))]
    #[cfg_attr(windows, test_case("C:\\abc\\.", "C:\\abc"; "absolute dot 7"))]
    #[cfg_attr(windows, test_case("C:\\abc\\.\\", "C:\\abc"; "absolute dot 8"))]
    #[cfg_attr(unix, test_case("/..", "/"; "absolute double dot 1"))]
    #[cfg_attr(unix, test_case("/../", "/"; "absolute double dot 2"))]
    #[cfg_attr(unix, test_case("/../..", "/"; "absolute double dot 3"))]
    #[cfg_attr(unix, test_case("/../../", "/"; "absolute double dot 4"))]
    #[cfg_attr(unix, test_case("/../abc", "/abc"; "absolute double dot 5"))]
    #[cfg_attr(unix, test_case("/../abc/", "/abc"; "absolute double dot 6"))]
    #[cfg_attr(unix, test_case("/abc/..", "/"; "absolute double dot 7"))]
    #[cfg_attr(unix, test_case("/abc/../", "/"; "absolute double dot 8"))]
    #[cfg_attr(unix, test_case("/abc/../def", "/def"; "absolute double dot 9"))]
    #[cfg_attr(unix, test_case("/abc/../def/", "/def"; "absolute double dot 10"))]
    #[cfg_attr(unix, test_case("/abc/../def/ghi", "/def/ghi"; "absolute double dot 11"))]
    #[cfg_attr(unix, test_case("/abc/../def/ghi/", "/def/ghi"; "absolute double dot 12"))]
    #[cfg_attr(unix, test_case("/abc/../../ghi", "/ghi"; "absolute double dot 13"))]
    #[cfg_attr(unix, test_case("/abc/../../ghi/", "/ghi"; "absolute double dot 14"))]
    #[cfg_attr(unix, test_case("/abc/def/../../ghi", "/ghi"; "absolute double dot 15"))]
    #[cfg_attr(unix, test_case("/abc/def/../../ghi/", "/ghi"; "absolute double dot 16"))]
    #[cfg_attr(windows, test_case("C:\\..", "C:\\"; "absolute double dot 1"))]
    #[cfg_attr(windows, test_case("C:\\..\\", "C:\\"; "absolute double dot 2"))]
    #[cfg_attr(windows, test_case("C:\\..\\..", "C:\\"; "absolute double dot 3"))]
    #[cfg_attr(windows, test_case("C:\\..\\..\\", "C:\\"; "absolute double dot 4"))]
    #[cfg_attr(windows, test_case("C:\\..\\abc", "C:\\abc"; "absolute double dot 5"))]
    #[cfg_attr(windows, test_case("C:\\..\\abc\\", "C:\\abc"; "absolute double dot 6"))]
    #[cfg_attr(windows, test_case("C:\\abc\\..", "C:\\"; "absolute double dot 7"))]
    #[cfg_attr(windows, test_case("C:\\abc\\..\\", "C:\\"; "absolute double dot 8"))]
    #[cfg_attr(windows, test_case("C:\\abc\\..\\def", "C:\\def"; "absolute double dot 9"))]
    #[cfg_attr(windows, test_case("C:\\abc\\..\\def\\", "C:\\def"; "absolute double dot 10"))]
    #[cfg_attr(windows, test_case("C:\\abc\\..\\def\\ghi", "C:\\def\\ghi"; "absolute double dot 11"))]
    #[cfg_attr(windows, test_case("C:\\abc\\..\\def\\ghi\\", "C:\\def\\ghi"; "absolute double dot 12"))]
    #[cfg_attr(windows, test_case("C:\\abc\\..\\..\\ghi", "C:\\ghi"; "absolute double dot 13"))]
    #[cfg_attr(windows, test_case("C:\\abc\\..\\..\\ghi\\", "C:\\ghi"; "absolute double dot 14"))]
    #[cfg_attr(windows, test_case("C:\\abc\\def\\..\\..\\ghi", "C:\\ghi"; "absolute double dot 15"))]
    #[cfg_attr(windows, test_case("C:\\abc\\def\\..\\..\\ghi\\", "C:\\ghi"; "absolute double dot 16"))]
    fn normalize(input: &str, output: &str) {
        assert_eq!(super::normalize(input), Ok(output.into()));
    }

    #[test_case("", ".."; "empty")]
    #[test_case("file.ext", "."; "name")]
    #[cfg_attr(unix, test_case("/", "/"; "root"))]
    #[cfg_attr(unix, test_case("/file.ext", "/"; "root parent"))]
    #[cfg_attr(unix, test_case(".", "./.."; "dot"))]
    #[cfg_attr(unix, test_case("..", "../.."; "double dot"))]
    #[cfg_attr(unix, test_case("./file.ext", "."; "dot parent"))]
    #[cfg_attr(unix, test_case("../file.ext", ".."; "double dot parent"))]
    #[cfg_attr(unix, test_case("dir/file.ext", "dir"; "name parent"))]
    #[cfg_attr(windows, test_case("C:", "C:\\"; "prefix"))]
    #[cfg_attr(windows, test_case("C:\\", "C:\\"; "root"))]
    #[cfg_attr(windows, test_case("C:\\file.ext", "C:\\"; "root parent"))]
    #[cfg_attr(windows, test_case(".", ".\\.."; "dot"))]
    #[cfg_attr(windows, test_case("..", "..\\.."; "double dot"))]
    #[cfg_attr(windows, test_case(".\\file.ext", "."; "dot parent"))]
    #[cfg_attr(windows, test_case("..\\file.ext", ".."; "double dot parent"))]
    #[cfg_attr(windows, test_case("dir\\file.ext", "dir"; "name parent"))]
    fn get_parent_directory(input: &str, output: &str) {
        assert_eq!(super::get_parent_directory(input.into()), Ok(output.into()));
    }

    #[test_case("", ""; "empty")]
    #[test_case(".", ""; "dot")]
    #[test_case("..", ""; "double dot")]
    #[test_case("file.ext", ""; "name")]
    #[cfg_attr(unix, test_case("/", "/"; "root"))]
    #[cfg_attr(unix, test_case("/file.ext", "/"; "root parent"))]
    #[cfg_attr(unix, test_case("./file.ext", "."; "dot parent"))]
    #[cfg_attr(unix, test_case("../file.ext", ".."; "double dot parent"))]
    #[cfg_attr(unix, test_case("dir/file.ext", "dir"; "name parent"))]
    #[cfg_attr(windows, test_case("C:", "C:"; "prefix"))]
    #[cfg_attr(windows, test_case("C:\\", "C:\\"; "root"))]
    #[cfg_attr(windows, test_case("C:\\file.ext", "C:\\"; "root parent"))]
    #[cfg_attr(windows, test_case(".\\file.ext", "."; "dot parent"))]
    #[cfg_attr(windows, test_case("..\\file.ext", ".."; "double dot parent"))]
    #[cfg_attr(windows, test_case("dir\\file.ext", "dir"; "name parent"))]
    fn remove_last_name(input: &str, output: &str) {
        assert_eq!(super::remove_last_name(input.into()), Ok(output.into()));
    }

    #[test_case("", ""; "empty")]
    #[test_case(".", ""; "dot")]
    #[test_case("..", ""; "double dot")]
    #[test_case("file.ext", "file.ext"; "name")]
    #[cfg_attr(unix, test_case("/", ""; "root"))]
    #[cfg_attr(unix, test_case("/file.ext", "file.ext"; "root parent"))]
    #[cfg_attr(unix, test_case("./file.ext", "file.ext"; "dot parent"))]
    #[cfg_attr(unix, test_case("../file.ext", "file.ext"; "double dot parent"))]
    #[cfg_attr(unix, test_case("dir/file.ext", "file.ext"; "name parent"))]
    #[cfg_attr(windows, test_case("C:", ""; "prefix"))]
    #[cfg_attr(windows, test_case("C:\\", ""; "root"))]
    #[cfg_attr(windows, test_case("C:\\file.ext", "file.ext"; "root parent"))]
    #[cfg_attr(windows, test_case(".\\file.ext", "file.ext"; "dot parent"))]
    #[cfg_attr(windows, test_case("..\\file.ext", "file.ext"; "double dot parent"))]
    #[cfg_attr(windows, test_case("dir\\file.ext", "file.ext"; "name parent"))]
    fn get_file_name(input: &str, output: &str) {
        assert_eq!(super::get_file_name(input), Ok(output.into()));
    }

    #[test_case("", ""; "empty")]
    #[test_case(".", "."; "dot")]
    #[test_case("..", ".."; "double dot")]
    #[test_case("file.ext", "file.ext"; "name")]
    #[cfg_attr(unix, test_case("/", ""; "root"))]
    #[cfg_attr(unix, test_case("/file.ext", "file.ext"; "root parent"))]
    #[cfg_attr(unix, test_case("./file.ext", "file.ext"; "dot parent"))]
    #[cfg_attr(unix, test_case("../file.ext", "file.ext"; "double dot parent"))]
    #[cfg_attr(unix, test_case("dir/file.ext", "file.ext"; "name parent"))]
    #[cfg_attr(windows, test_case("C:", ""; "prefix"))]
    #[cfg_attr(windows, test_case("C:\\", ""; "root"))]
    #[cfg_attr(windows, test_case("C:\\file.ext", "file.ext"; "root parent"))]
    #[cfg_attr(windows, test_case(".\\file.ext", "file.ext"; "dot parent"))]
    #[cfg_attr(windows, test_case("..\\file.ext", "file.ext"; "double dot parent"))]
    #[cfg_attr(windows, test_case("dir\\file.ext", "file.ext"; "name parent"))]
    fn get_last_name(input: &str, output: &str) {
        assert_eq!(super::get_last_name(input), Ok(output.into()));
    }

    #[test_case("", ""; "empty")]
    #[test_case(".", ""; "dot")]
    #[test_case("..", ""; "double dot")]
    #[test_case("file.ext", "file"; "name")]
    #[cfg_attr(unix, test_case("/", ""; "root"))]
    #[cfg_attr(unix, test_case("/file.ext", "file"; "root parent"))]
    #[cfg_attr(unix, test_case("./file.ext", "file"; "dot parent"))]
    #[cfg_attr(unix, test_case("../file.ext", "file"; "double dot parent"))]
    #[cfg_attr(unix, test_case("dir/file.ext", "file"; "name parent"))]
    #[cfg_attr(windows, test_case("C:", ""; "prefix"))]
    #[cfg_attr(windows, test_case("C:\\", ""; "root"))]
    #[cfg_attr(windows, test_case("C:\\file.ext", "file"; "root parent"))]
    #[cfg_attr(windows, test_case(".\\file.ext", "file"; "dot parent"))]
    #[cfg_attr(windows, test_case("..\\file.ext", "file"; "double dot parent"))]
    #[cfg_attr(windows, test_case("dir\\file.ext", "file"; "name parent"))]
    fn get_base_name(input: &str, output: &str) {
        assert_eq!(super::get_base_name(input), Ok(output.into()));
    }

    #[test_case("", ""; "empty")]
    #[test_case(".", "."; "dot")]
    #[test_case("..", ".."; "double dot")]
    #[test_case("file.ext", "file"; "name")]
    #[cfg_attr(unix, test_case("/", "/"; "root"))]
    #[cfg_attr(unix, test_case("/file.ext", "/file"; "root parent"))]
    #[cfg_attr(unix, test_case("./file.ext", "./file"; "dot parent"))]
    #[cfg_attr(unix, test_case("../file.ext", "../file"; "double dot parent"))]
    #[cfg_attr(unix, test_case("dir/file.ext", "dir/file"; "name parent"))]
    #[cfg_attr(windows, test_case("C:", "C:"; "prefix"))]
    #[cfg_attr(windows, test_case("C:\\", "C:\\"; "root"))]
    #[cfg_attr(windows, test_case("C:\\file.ext", "C:\\file"; "root parent"))]
    #[cfg_attr(windows, test_case(".\\file.ext", ".\\file"; "dot parent"))]
    #[cfg_attr(windows, test_case("..\\file.ext", "..\\file"; "double dot parent"))]
    #[cfg_attr(windows, test_case("dir\\file.ext", "dir\\file"; "name parent"))]
    fn remove_extension(input: &str, output: &str) {
        assert_eq!(super::remove_extension(input.into()), Ok(output.into()));
    }

    #[test_case("", ""; "empty")]
    #[test_case(".", ""; "dot")]
    #[test_case("..", ""; "double dot")]
    #[test_case("file.ext", "ext"; "name")]
    #[cfg_attr(unix, test_case("/", ""; "root"))]
    #[cfg_attr(unix, test_case("/file.ext", "ext"; "root parent"))]
    #[cfg_attr(unix, test_case("./file.ext", "ext"; "dot parent"))]
    #[cfg_attr(unix, test_case("../file.ext", "ext"; "double dot parent"))]
    #[cfg_attr(unix, test_case("dir/file.ext", "ext"; "name parent"))]
    #[cfg_attr(windows, test_case("C:", ""; "prefix"))]
    #[cfg_attr(windows, test_case("C:\\", ""; "root"))]
    #[cfg_attr(windows, test_case("C:\\file.ext", "ext"; "root parent"))]
    #[cfg_attr(windows, test_case(".\\file.ext", "ext"; "dot parent"))]
    #[cfg_attr(windows, test_case("..\\file.ext", "ext"; "double dot parent"))]
    #[cfg_attr(windows, test_case("dir\\file.ext", "ext"; "name parent"))]
    fn get_extension(input: &str, output: &str) {
        assert_eq!(super::get_extension(input), Ok(output.into()));
    }

    #[test_case("", ""; "empty")]
    #[test_case(".", ""; "dot")]
    #[test_case("..", ""; "double dot")]
    #[test_case("file.ext", ".ext"; "name")]
    #[cfg_attr(unix, test_case("/", ""; "root"))]
    #[cfg_attr(unix, test_case("/file.ext", ".ext"; "root parent"))]
    #[cfg_attr(unix, test_case("./file.ext", ".ext"; "dot parent"))]
    #[cfg_attr(unix, test_case("../file.ext", ".ext"; "double dot parent"))]
    #[cfg_attr(unix, test_case("dir/file.ext", ".ext"; "name parent"))]
    #[cfg_attr(windows, test_case("C:", ""; "prefix"))]
    #[cfg_attr(windows, test_case("C:\\", ""; "root"))]
    #[cfg_attr(windows, test_case("C:\\file.ext", ".ext"; "root parent"))]
    #[cfg_attr(windows, test_case(".\\file.ext", ".ext"; "dot parent"))]
    #[cfg_attr(windows, test_case("..\\file.ext", ".ext"; "double dot parent"))]
    #[cfg_attr(windows, test_case("dir\\file.ext", ".ext"; "name parent"))]
    fn get_extension_with_dot(input: &str, output: &str) {
        assert_eq!(super::get_extension_with_dot(input), Ok(output.into()));
    }

    #[cfg_attr(unix, test_case("", "/"; "empty"))]
    #[cfg_attr(unix, test_case("/", "/"; "root"))]
    #[cfg_attr(unix, test_case("dir", "dir/"; "name"))]
    #[cfg_attr(unix, test_case("dir/", "dir/"; "name separator"))]
    #[cfg_attr(windows, test_case("", "\\"; "empty"))]
    #[cfg_attr(windows, test_case("C:", "C:\\"; "prefix"))]
    #[cfg_attr(windows, test_case("C:\\", "C:\\"; "root"))]
    #[cfg_attr(windows, test_case("C:/", "C:\\"; "root unix separator"))]
    #[cfg_attr(windows, test_case("dir", "dir\\"; "name"))]
    #[cfg_attr(windows, test_case("dir\\", "dir\\"; "name separator"))]
    #[cfg_attr(windows, test_case("dir/", "dir\\"; "name unix separator"))]
    fn ensure_trailing_dir_separator(input: &str, output: &str) {
        assert_eq!(super::ensure_trailing_dir_separator(input.into()), output);
    }

    #[cfg_attr(unix, test_case("", ""; "empty"))]
    #[cfg_attr(unix, test_case("/", ""; "root"))]
    #[cfg_attr(unix, test_case("dir", "dir"; "name"))]
    #[cfg_attr(unix, test_case("dir/", "dir"; "name separator"))]
    #[cfg_attr(windows, test_case("", ""; "empty"))]
    #[cfg_attr(windows, test_case("C:", "C:"; "prefix"))]
    #[cfg_attr(windows, test_case("C:\\", "C:"; "root"))]
    #[cfg_attr(windows, test_case("C:/", "C:"; "root unix separator"))]
    #[cfg_attr(windows, test_case("dir", "dir"; "name"))]
    #[cfg_attr(windows, test_case("dir\\", "dir"; "name separator"))]
    #[cfg_attr(windows, test_case("dir/", "dir"; "name unix separator"))]
    fn remove_trailing_dir_separator(input: &str, output: &str) {
        assert_eq!(super::remove_trailing_dir_separator(input.into()), output);
    }

    #[test_case("abc", Ok("abc".into()); "utf-8")]
    #[test_case(make_non_utf8_os_string(), Err(ErrorKind::InputNotUtf8); "non utf-8")]
    fn into_string<T: Into<PathBuf>>(input: T, result: BaseResult<String>) {
        assert_eq!(super::into_string(input.into()), result);
    }

    #[test_case("abc", Ok("abc"); "utf-8")]
    #[test_case(make_non_utf8_os_string(), Err(ErrorKind::InputNotUtf8); "non utf-8")]
    fn to_str<T: Into<OsString>>(input: T, result: BaseResult<&str>) {
        assert_eq!(super::to_str(&input.into()), result);
    }

    #[cfg(unix)]
    pub fn make_non_utf8_os_string() -> OsString {
        use std::os::unix::prelude::*;
        OsString::from(OsStr::from_bytes(&[0x66, 0x6f, 0x80, 0x6f][..]))
    }

    #[cfg(windows)]
    pub fn make_non_utf8_os_string() -> OsString {
        use std::os::windows::prelude::*;
        OsString::from_wide(&[0x0066, 0x006f, 0xD800, 0x006f][..])
    }

    fn fmt_working_dir(template: &str, working_dir: &Path) -> String {
        template.replace("{working_dir}", working_dir.to_str().unwrap())
    }
}
