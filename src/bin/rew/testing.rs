use std::ffi::OsStr;

#[cfg(any(unix))]
pub fn make_non_utf8_os_str<'a>() -> &'a OsStr {
    use std::os::unix::ffi::OsStrExt;
    OsStr::from_bytes(&[0x66, 0x6f, 0x80, 0x6f][..])
}

#[cfg(any(windows))]
pub fn make_non_utf8_os_str<'a>() -> &'a OsStr {
    use std::ffi::OsString;
    use std::os::windows::prelude::*;
    OsString::from_wide(&[0x0066, 0x006f, 0xD800, 0x006f][..]).as_os_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_non_utf8_os_str() {
        assert!(make_non_utf8_os_str().to_str().is_none());
    }
}
