use derive_more::Display;
use std::str::FromStr;

#[derive(Debug, Display, PartialEq, Clone)]
pub enum Index<T> {
    Forward(T),
    #[display("-{_0}")]
    Backward(T),
}

#[derive(Debug, Display, PartialEq, Clone)]
#[display(bound(T: Display))]
#[display("{}..{}", format_opt(_0), format_opt(_1))]
pub struct Range<T>(pub Option<T>, pub Option<T>);

#[derive(Debug, Display, PartialEq, Clone)]
#[display(bound(T: Display))]
#[display("{_0}..{}", format_opt(_1))]
#[allow(clippy::module_name_repetitions)]
pub struct StartRange<T>(pub T, pub Option<T>);

#[derive(Debug, Display, derive_more::Error, PartialEq)]
pub enum Error {
    #[display("missing delimiter '..'")]
    MissingDelimiter,
    #[display("missing lower bound")]
    MissingLowerBound,
    #[display("invalid lower bound '{_0}'")]
    InvalidLowerBound(#[error(not(source))] String),
    #[display("invalid upper bound '{_0}'")]
    InvalidUpperBound(#[error(not(source))] String),
}

impl<T: FromStr> FromStr for Index<T> {
    type Err = T::Err;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some(after_prefix) = input.strip_prefix('-') {
            Ok(Self::Backward(T::from_str(after_prefix)?))
        } else {
            Ok(Self::Forward(T::from_str(input)?))
        }
    }
}

impl<T: FromStr + Clone> FromStr for Range<T> {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some((start, end)) = input.split_once("..") {
            let start = opt_from_str(start).map_err(|_| Error::InvalidLowerBound(start.into()))?;
            let end = opt_from_str(end).map_err(|_| Error::InvalidUpperBound(end.into()))?;
            Ok(Self(start, end))
        } else {
            Err(Error::MissingDelimiter)
        }
    }
}

impl<T: FromStr + Clone> FromStr for StartRange<T> {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some((start, end)) = input.split_once("..") {
            let start = T::from_str(start).map_err(|_| {
                if start.is_empty() {
                    Error::MissingLowerBound
                } else {
                    Error::InvalidLowerBound(start.into())
                }
            })?;
            let end = opt_from_str(end).map_err(|_| Error::InvalidUpperBound(end.into()))?;
            Ok(Self(start, end))
        } else {
            Err(Error::MissingDelimiter)
        }
    }
}

fn opt_from_str<T: FromStr>(input: &str) -> Result<Option<T>, T::Err> {
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(T::from_str(input)?))
    }
}

fn format_opt<T: Display>(value: &Option<T>) -> String {
    value.as_ref().map(ToString::to_string).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_err_eq;
    use claims::assert_ok_eq;
    use rstest::rstest;
    use std::num::IntErrorKind;

    #[rstest]
    #[case(Index::Forward(1), "1")]
    #[case(Index::Forward(23), "23")]
    #[case(Index::Backward(4), "-4")]
    #[case(Index::Backward(56), "-56")]
    fn index_display(#[case] input: Index<usize>, #[case] output: &str) {
        assert_eq!(input.to_string(), output);
    }

    #[rstest]
    #[case("1", Index::Forward(1))]
    #[case("23", Index::Forward(23))]
    #[case("-4", Index::Backward(4))]
    #[case("-56", Index::Backward(56))]
    fn index_parse_ok(#[case] input: &str, #[case] output: Index<usize>) {
        assert_ok_eq!(Index::from_str(input), output);
    }

    #[rstest]
    #[case("", IntErrorKind::Empty)]
    #[case("x", IntErrorKind::InvalidDigit)]
    fn index_parse_err(#[case] input: &str, #[case] output: IntErrorKind) {
        assert_err_eq!(
            Index::<i32>::from_str(input).map_err(|err| err.kind().clone()),
            output
        );
    }

    #[rstest]
    #[case(Range(None, None), "..")]
    #[case(Range(Some(1), None), "1..")]
    #[case(Range(None, Some(-2)), "..-2")]
    #[case(Range(Some(-34), Some(56)), "-34..56")]
    fn range_display(#[case] input: Range<i32>, #[case] output: &str) {
        assert_eq!(input.to_string(), output);
    }

    #[rstest]
    #[case("..", Range(None, None))]
    #[case("1..", Range(Some(1), None))]
    #[case("..2", Range(None, Some(2)))]
    #[case("34..-56", Range(Some(34), Some(-56)))]
    #[case("-34..56", Range(Some(-34), Some(56)))]
    fn range_parse_ok(#[case] input: &str, #[case] output: Range<i32>) {
        assert_ok_eq!(Range::<i32>::from_str(input), output);
    }

    #[rstest]
    #[case("", Error::MissingDelimiter)]
    #[case(".", Error::MissingDelimiter)]
    #[case("lo..", Error::InvalidLowerBound("lo".into()))]
    #[case("lo..up", Error::InvalidLowerBound("lo".into()))]
    #[case("..up", Error::InvalidUpperBound("up".into()))]
    fn range_parse_err(#[case] input: &str, #[case] output: Error) {
        assert_err_eq!(Range::<i32>::from_str(input), output);
    }

    #[rstest]
    #[case(StartRange(1, None), "1..")]
    #[case(StartRange(-34, Some(56)), "-34..56")]
    fn start_range_display(#[case] input: StartRange<i32>, #[case] output: &str) {
        assert_eq!(input.to_string(), output);
    }

    #[rstest]
    #[case("1..", StartRange(1, None))]
    #[case("-2..", StartRange(-2, None))]
    #[case("34..-56", StartRange(34, Some(-56)))]
    #[case("-34..56", StartRange(-34, Some(56)))]
    fn start_range_parse_ok(#[case] input: &str, #[case] output: StartRange<i32>) {
        assert_ok_eq!(StartRange::<i32>::from_str(input), output);
    }

    #[rstest]
    #[case("", Error::MissingDelimiter)]
    #[case(".", Error::MissingDelimiter)]
    #[case("..", Error::MissingLowerBound)]
    #[case("..2", Error::MissingLowerBound)]
    #[case("lo..", Error::InvalidLowerBound("lo".into()))]
    #[case("lo..up", Error::InvalidLowerBound("lo".into()))]
    #[case("..up", Error::MissingLowerBound)]
    fn start_range_parse_err(#[case] input: &str, #[case] output: Error) {
        assert_err_eq!(StartRange::<i32>::from_str(input), output);
    }
}
