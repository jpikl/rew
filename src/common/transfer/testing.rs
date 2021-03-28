use fs_extra::error::{Error, ErrorKind};

pub fn unpack_fse_error(error: Error) -> (String, String) {
    let message = error.to_string();
    (debug_fse_error_kind(error.kind), message)
}

pub fn debug_fse_error_kind(error_kind: ErrorKind) -> String {
    format!("ErrorKind::{:?}", error_kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack_fse_error() {
        assert_eq!(
            super::unpack_fse_error(Error::new(ErrorKind::Other, "test")),
            ("ErrorKind::Other".into(), "test".into())
        );
    }

    #[test]
    fn debug_fse_error_kind() {
        assert_eq!(
            super::debug_fse_error_kind(ErrorKind::Other),
            "ErrorKind::Other"
        );
    }
}
