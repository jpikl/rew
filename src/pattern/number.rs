use crate::pattern::error::ParseError;
use crate::pattern::source::Source;

pub fn parse_usize(source: &mut Source) -> Result<usize, ParseError> {
    match source.peek() {
        Some('0') => {
            source.consume();
            Ok(0)
        }
        Some(ch @ '1'..='9') => {
            source.consume();
            let mut number = ch.to_digit(10).unwrap() as usize;
            while let Some(ch @ '0'..='9') = source.peek() {
                number = 10 * number + ch.to_digit(10).unwrap() as usize;
                source.consume();
            }
            Ok(number)
        }
        _ => Err(ParseError {
            message: "Expected number",
            position: 0,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_as_error() {
        let mut source = Source::new("");
        assert_eq!(
            parse_usize(&mut source),
            Err(ParseError {
                message: "Expected number",
                position: 0,
            })
        );
        assert_eq!(source.position(), 0);
    }

    #[test]
    fn parse_non_digit_as_error() {
        let mut source = Source::new("a");
        assert_eq!(
            parse_usize(&mut source),
            Err(ParseError {
                message: "Expected number",
                position: 0,
            })
        );
        assert_eq!(source.position(), 0);
    }

    #[test]
    fn parse_zero() {
        let mut source = Source::new("0");
        assert_eq!(parse_usize(&mut source), Ok(0));
        assert_eq!(source.position(), 1);
    }

    #[test]
    fn parse_zero_ignore_rest() {
        let mut source = Source::new("0a");
        assert_eq!(parse_usize(&mut source), Ok(0));
        assert_eq!(source.position(), 1);
    }

    #[test]
    fn parse_only_a_first_zero() {
        let mut source = Source::new("00");
        assert_eq!(parse_usize(&mut source), Ok(0));
        assert_eq!(source.position(), 1);
    }

    #[test]
    fn parse_positive_number_single_digit() {
        let mut source = Source::new("1");
        assert_eq!(parse_usize(&mut source), Ok(1));
        assert_eq!(source.position(), 1);
    }

    #[test]
    fn parse_positive_number_single_digit_ignore_rest() {
        let mut source = Source::new("1a");
        assert_eq!(parse_usize(&mut source), Ok(1));
        assert_eq!(source.position(), 1);
    }

    #[test]
    fn parse_positive_number_multiple_digits() {
        let mut source = Source::new("1234567890");
        assert_eq!(parse_usize(&mut source), Ok(1234567890));
        assert_eq!(source.position(), 10);
    }

    #[test]
    fn parse_positive_number_multiple_digits_ignore_rest() {
        let mut source = Source::new("1234567890a");
        assert_eq!(parse_usize(&mut source), Ok(1234567890));
        assert_eq!(source.position(), 10);
    }
}
