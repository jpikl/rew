use crate::global::PathStyle;
use bstr::ByteSlice;
use std::ops::Range;

enum PathSeparator {
    Unix,
    Windows,
    Combined,
}

impl PathSeparator {
    fn is(&self, b: u8) -> bool {
        match self {
            PathSeparator::Unix => b == b'/',
            PathSeparator::Windows => b == b'\\',
            PathSeparator::Combined => b == b'/' || b == b'\\',
        }
    }

    fn find_in(&self, path: &[u8]) -> Option<usize> {
        match self {
            PathSeparator::Unix => path.find_byte(b'/'),
            PathSeparator::Windows => path.find_byte(b'\\'),
            PathSeparator::Combined => path.find_byteset(b"\\/"),
        }
    }

    fn rfind_in(&self, path: &[u8]) -> Option<usize> {
        match self {
            PathSeparator::Unix => path.rfind_byte(b'/'),
            PathSeparator::Windows => path.rfind_byte(b'\\'),
            PathSeparator::Combined => path.rfind_byteset(b"\\/"),
        }
    }

    fn count_front(&self, path: &[u8]) -> usize {
        let mut pos = 0;
        while pos < path.len() && self.is(path[pos]) {
            pos += 1;
        }
        pos
    }

    fn count_back(&self, path: &[u8]) -> usize {
        let mut pos = path.len();
        while pos > 0 && self.is(path[pos - 1]) {
            pos -= 1;
        }
        path.len() - pos
    }
}

fn get_separator(path: &[u8], style: PathStyle) -> PathSeparator {
    match style {
        PathStyle::Unix => PathSeparator::Unix,
        PathStyle::Windows if path.starts_with(br"\\?\") => PathSeparator::Windows,
        PathStyle::Windows => PathSeparator::Combined,
    }
}

pub fn get_prefix_len(path: &[u8], style: PathStyle) -> usize {
    match style {
        PathStyle::Unix => 0,
        PathStyle::Windows => get_win_prefix_len(path),
    }
}

pub fn get_win_prefix_len(path: &[u8]) -> usize {
    if path.len() >= 2 && path[0].is_ascii_alphabetic() && path[1] == b':' {
        return 2;
    }
    if let Some(verbatim) = path.strip_prefix(br"\\?\") {
        if let Some(unc) = verbatim.strip_prefix(br"UNC\") {
            return 8 + get_win_unc_prefix_len(unc, true, PathSeparator::Windows);
        }
        return 4 + PathSeparator::Windows.find_in(verbatim).unwrap_or(verbatim.len());
    }
    if let Some(device) = path.strip_prefix(br"\\.\") {
        return 4 + PathSeparator::Combined.find_in(device).unwrap_or(device.len());
    }
    if let Some(unc) = path.strip_prefix(br"\\") {
        let unc_len = get_win_unc_prefix_len(unc, false, PathSeparator::Combined);
        if unc_len > 0 {
            return 2 + unc_len;
        }
    }
    0
}

fn get_win_unc_prefix_len(path: &[u8], allow_empty: bool, separator: PathSeparator) -> usize {
    let Some(host_len) = separator.find_in(path) else {
        return 0;
    };
    if host_len == 0 && !allow_empty {
        return 0;
    }
    let suffix = &path[host_len + 1..];
    let share_len = separator.find_in(suffix).unwrap_or(suffix.len());
    if share_len == 0 && !allow_empty {
        return 0;
    }
    host_len + share_len + 1
}

pub fn get_dir(path: &[u8], style: PathStyle) -> &[u8] {
    let separator = get_separator(path, style);
    let start = get_prefix_len(path, style);
    let mut end = path.len();

    end -= separator.count_back(&path[start..end]);
    end = start + separator.rfind_in(&path[start..end]).unwrap_or_default();
    end -= separator.count_back(&path[start..end]);

    if end > start {
        return &path[..end];
    }

    if start > 0 {
        return &path[..start];
    }

    if !path.is_empty() && separator.is(path[0]) {
        return &path[..1];
    }

    b"."
}

pub fn get_base(path: &[u8], style: PathStyle, with_dir: bool, shortest: bool) -> &[u8] {
    let Some(file_range) = get_file_position(path, style) else {
        return b"";
    };

    let file = &path[file_range.clone()];
    if let Some(dot_pos) = get_dot_position(file, !shortest) {
        if with_dir {
            return &path[..file_range.start + dot_pos];
        }
        return &file[..dot_pos];
    }

    if with_dir {
        return &path[..file_range.end];
    }
    file
}

pub fn get_file(path: &[u8], style: PathStyle) -> &[u8] {
    let Some(file_range) = get_file_position(path, style) else {
        return b"";
    };
    &path[file_range]
}

pub fn get_extension(path: &[u8], style: PathStyle, with_dot: bool, longest: bool) -> &[u8] {
    let Some(file_range) = get_file_position(path, style) else {
        return b"";
    };

    let file = &path[file_range];
    if let Some(dot_pos) = get_dot_position(file, !longest) {
        if with_dot {
            return &file[dot_pos..];
        }
        return &file[dot_pos + 1..];
    }

    b""
}

pub fn get_file_position(path: &[u8], style: PathStyle) -> Option<Range<usize>> {
    let separator = get_separator(path, style);
    let mut start = get_prefix_len(path, style);
    let mut end = path.len();

    start += separator.count_front(&path[start..end]);
    end -= separator.count_back(&path[start..end]);

    if let Some(pos) = separator.rfind_in(&path[start..end]) {
        start = pos + 1;
    }

    if start >= end {
        return None;
    }

    Some(start..end)
}

pub fn get_dot_position(file: &[u8], reverse: bool) -> Option<usize> {
    if file.is_empty() || file == b"." || file == b".." {
        return None;
    }

    // Ignore initial dot in names like `.gitignore`
    let start = if file[0] == b'.' { 1 } else { 0 };
    let slice = &file[start..];

    let pos = match reverse {
        true => slice.rfind_byte(b'.'),
        false => slice.find_byte(b'.'),
    };

    pos.map(|pos| pos + start)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global::PathStyle;
    use rstest::rstest;

    #[rstest]
    #[case(PathStyle::Unix, "", ".")]
    #[case(PathStyle::Unix, ".", ".")]
    #[case(PathStyle::Unix, "./", ".")]
    #[case(PathStyle::Unix, ".//", ".")]
    #[case(PathStyle::Unix, "./dir", ".")]
    #[case(PathStyle::Unix, "./dir/", ".")]
    #[case(PathStyle::Unix, ".//dir//", ".")]
    #[case(PathStyle::Unix, "./dir/name.ext", "./dir")]
    #[case(PathStyle::Unix, "./dir/name.ext/", "./dir")]
    #[case(PathStyle::Unix, ".//dir//name.ext//", ".//dir")]
    #[case(PathStyle::Unix, "..", ".")]
    #[case(PathStyle::Unix, "../", ".")]
    #[case(PathStyle::Unix, "..//", ".")]
    #[case(PathStyle::Unix, "../dir", "..")]
    #[case(PathStyle::Unix, "../dir/", "..")]
    #[case(PathStyle::Unix, "..//dir//", "..")]
    #[case(PathStyle::Unix, "../dir/name.ext", "../dir")]
    #[case(PathStyle::Unix, "../dir/name.ext/", "../dir")]
    #[case(PathStyle::Unix, "..//dir//name.ext//", "..//dir")]
    #[case(PathStyle::Unix, "parent", ".")]
    #[case(PathStyle::Unix, "parent/", ".")]
    #[case(PathStyle::Unix, "parent//", ".")]
    #[case(PathStyle::Unix, "parent/dir", "parent")]
    #[case(PathStyle::Unix, "parent/dir/", "parent")]
    #[case(PathStyle::Unix, "parent//dir//", "parent")]
    #[case(PathStyle::Unix, "parent/dir/name.ext", "parent/dir")]
    #[case(PathStyle::Unix, "parent/dir/name.ext/", "parent/dir")]
    #[case(PathStyle::Unix, "parent//dir//name.ext//", "parent//dir")]
    #[case(PathStyle::Unix, "/", "/")]
    #[case(PathStyle::Unix, "//", "/")]
    #[case(PathStyle::Unix, "/root", "/")]
    #[case(PathStyle::Unix, "/root/", "/")]
    #[case(PathStyle::Unix, "//root//", "/")]
    #[case(PathStyle::Unix, "/root/dir", "/root")]
    #[case(PathStyle::Unix, "/root/dir/", "/root")]
    #[case(PathStyle::Unix, "//root//dir//", "//root")]
    #[case(PathStyle::Unix, "/root/dir/name.ext", "/root/dir")]
    #[case(PathStyle::Unix, "/root/dir/name.ext/", "/root/dir")]
    #[case(PathStyle::Unix, "//root//dir//name.ext//", "//root//dir")]
    fn dir(#[case] style: PathStyle, #[case] input: &str, #[case] output: &str) {
        assert_eq!(get_dir(input.as_bytes(), style).to_str_lossy(), output);
    }

    #[rstest]
    #[case(r"c:", r"c:")] // 1
    #[case(r"c:dir", r"c:")] // 2
    #[case(r"c:\", r"c:")] // 3
    #[case(r"c:/", r"c:")] // 4
    #[case(r"C:", r"C:")] // 5
    #[case(r"C:dir", r"C:")] // 6
    #[case(r"C:\", r"C:")] // 7
    #[case(r"C:/", r"C:")] // 8
    #[case(r"\", r"")] // 9
    #[case(r"\\", r"")] // 10
    #[case(r"\\\", r"")] // 11
    #[case(r"\\\dir", r"")] // 12
    #[case(r"/", r"")] // 13
    #[case(r"//", r"")] // 14
    #[case(r"///dir", r"")] // 15
    #[case(r"\\server", r"")] // 16
    #[case(r"\\server\", r"")] // 17
    #[case(r"\\server\share", r"\\server\share")] // 18
    #[case(r"\\server\share\", r"\\server\share")] // 19
    #[case(r"\\server\share\dir", r"\\server\share")] // 20
    #[case(r"\\server\\share", r"")] // 21
    #[case(r"\\server\\share\\dir", r"")] // 22
    #[case(r"\\server/share", r"\\server/share")] // 23
    #[case(r"\\server//share", r"")] // 24
    #[case(r"\\server//share//dir", r"")] // 25
    #[case(r"\\?", r"")] // 26
    #[case(r"\\?\", r"\\?\")] // 27
    #[case(r"\\?\device", r"\\?\device")] // 28
    #[case(r"\\?\device\", r"\\?\device")] // 29
    #[case(r"\\?\device\\", r"\\?\device")] // 30
    #[case(r"\\?\device/", r"\\?\device/")] // 31
    #[case(r"\\?\device\dir", r"\\?\device")] // 32
    #[case(r"\\?\device\\dir", r"\\?\device")] // 33
    #[case(r"\\?\device/dir", r"\\?\device/dir")] // 34
    #[case(r"\\?\UNC", r"\\?\UNC")] // 35
    #[case(r"\\?\UNC\", r"\\?\UNC\")] // 36
    #[case(r"\\?\UNC\\", r"\\?\UNC\")] // 37
    #[case(r"\\?\UNC/", r"\\?\UNC/")] // 38
    #[case(r"\\?\UNC\server", r"\\?\UNC\server")] // 39
    #[case(r"\\?\UNC\\server", r"\\?\UNC\\server")] // 40
    #[case(r"\\?\UNC\server\share", r"\\?\UNC\server\share")] // 41
    #[case(r"\\?\UNC\server\\share", r"\\?\UNC\server")] // 42
    #[case(r"\\?\UNC\\server\share", r"\\?\UNC\\server")] // 43
    #[case(r"\\?\UNC\\server\\share", r"\\?\UNC\\server")] // 44
    #[case(r"\\?\UNC\server\share\dir", r"\\?\UNC\server\share")] // 45
    #[case(r"\\?\UNC\server\share/dir", r"\\?\UNC\server\share/dir")] // 46
    #[case(r"\\?\UNC\server/share/dir", r"\\?\UNC\server/share/dir")] // 47
    #[case(r"\\.", r"")] // 48
    #[case(r"\\.\", r"\\.\")] // 49
    #[case(r"\\.\device", r"\\.\device")] // 50
    #[case(r"\\.\device\dir", r"\\.\device")] // 51
    #[case(r"\\.\device\\dir", r"\\.\device")] // 52
    #[case(r"\\.\device/dir", r"\\.\device")] // 53
    fn windows_prefix_len(#[case] input: &str, #[case] output: &str) {
        assert_eq!(&input[..get_win_prefix_len(input.as_bytes())], output)
    }
}
