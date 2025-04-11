use std::ffi::OsString;

pub fn into_bytes(value: OsString) -> Vec<u8> {
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::ffi::OsStringExt;
        value.into_vec()
    }
    #[cfg(not(target_family = "unix"))]
    {
        value.to_string_lossy().into_owned().into_bytes()
    }
}
