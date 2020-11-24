use std::ffi::OsString;

#[cfg(unix)]
pub fn make_non_utf8_os_string() -> OsString {
    use std::ffi::OsStr;
    use std::os::unix::prelude::*;
    OsString::from(OsStr::from_bytes(&[0x66, 0x6f, 0x80, 0x6f][..]))
}

#[cfg(windows)]
pub fn make_non_utf8_os_string() -> OsString {
    use std::os::windows::prelude::*;
    OsString::from_wide(&[0x0066, 0x006f, 0xD800, 0x006f][..])
}

#[cfg(test)]
mod tests {
    use claim::*;

    #[test]
    fn make_non_utf8_os_string() {
        use super::*;
        assert_none!(make_non_utf8_os_string().to_str());
    }
}
