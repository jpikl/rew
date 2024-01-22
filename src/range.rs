use anyhow::anyhow;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum Index<T> {
    Forward(T),
    Backward(T),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Range<T>(pub Option<T>, pub Option<T>);

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct StartRange<T>(pub T, pub Option<T>);

#[allow(dead_code)]
#[allow(clippy::module_name_repetitions)]
pub type IndexRange<T> = Range<Index<T>>;

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
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some((start, end)) = input.split_once("..") {
            let start =
                option_from_str(start).map_err(|_| anyhow!("invalid lower bound '{}'", start))?;
            let end = option_from_str(end).map_err(|_| anyhow!("invalid upper bound '{}'", end))?;
            Ok(Self(start, end))
        } else {
            let number = T::from_str(input).map_err(|_| anyhow!("invalid number '{}'", input))?;
            Ok(Self(Some(number.clone()), Some(number)))
        }
    }
}

impl<T: FromStr + Clone> FromStr for StartRange<T> {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if let Some((start, end)) = input.split_once("..") {
            let start =
                T::from_str(start).map_err(|_| anyhow!("invalid lower bound '{}'", start))?;
            let end = option_from_str(end).map_err(|_| anyhow!("invalid upper bound '{}'", end))?;
            Ok(Self(start, end))
        } else {
            Err(anyhow!("missing bounds delimiter '..'"))
        }
    }
}

fn option_from_str<T: FromStr>(input: &str) -> Result<Option<T>, T::Err> {
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(T::from_str(input)?))
    }
}

impl<T: Display> Display for Index<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forward(value) | Self::Backward(value) => value.fmt(f),
        }
    }
}

impl<T: Display> Display for Range<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(start) = &self.0 {
            start.fmt(f)?;
        }
        "..".fmt(f)?;
        if let Some(end) = &self.1 {
            end.fmt(f)?;
        }
        Ok(())
    }
}

impl<T: Display> Display for StartRange<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)?;
        "..".fmt(f)?;
        if let Some(end) = &self.1 {
            end.fmt(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_err_eq;
    use claims::assert_ok_eq;
    use std::num::IntErrorKind;

    #[test]
    fn index() {
        assert_ok_eq!(Index::<usize>::from_str("12"), Index::Forward(12));
        assert_ok_eq!(Index::<usize>::from_str("-3"), Index::Backward(3));

        assert_err_eq!(
            Index::<usize>::from_str("").map_err(|err| err.kind().clone()),
            IntErrorKind::Empty
        );

        assert_err_eq!(
            Index::<usize>::from_str("3x").map_err(|err| err.kind().clone()),
            IntErrorKind::InvalidDigit
        );
    }

    #[test]
    fn index_range() {
        assert_ok_eq!(IndexRange::<usize>::from_str(".."), Range(None, None));

        assert_ok_eq!(
            IndexRange::<usize>::from_str("..-2"),
            Range(None, Some(Index::Backward(2)))
        );

        assert_ok_eq!(
            IndexRange::<usize>::from_str("1.."),
            Range(Some(Index::Forward(1)), None)
        );

        assert_ok_eq!(
            IndexRange::<usize>::from_str("1..-2"),
            Range(Some(Index::Forward(1)), Some(Index::Backward(2)))
        );

        assert_ok_eq!(
            IndexRange::<usize>::from_str("3"),
            Range(Some(Index::Forward(3)), Some(Index::Forward(3)))
        );

        assert_err_eq!(
            IndexRange::<usize>::from_str("x..-2").map_err(|err| err.to_string()),
            "invalid lower bound 'x'"
        );

        assert_err_eq!(
            IndexRange::<usize>::from_str("1..x").map_err(|err| err.to_string()),
            "invalid upper bound 'x'"
        );

        assert_err_eq!(
            IndexRange::<usize>::from_str("1.-2").map_err(|err| err.to_string()),
            "invalid number '1.-2'"
        );

        assert_err_eq!(
            IndexRange::<usize>::from_str("").map_err(|err| err.to_string()),
            "invalid number ''"
        );
    }

    #[test]
    fn start_range() {
        assert_err_eq!(
            StartRange::<i32>::from_str("..").map_err(|err| err.to_string()),
            "invalid lower bound ''"
        );

        assert_err_eq!(
            StartRange::<i32>::from_str("..-2").map_err(|err| err.to_string()),
            "invalid lower bound ''"
        );

        assert_ok_eq!(StartRange::<i32>::from_str("1.."), StartRange(1, None));

        assert_ok_eq!(
            StartRange::<i32>::from_str("1..-2"),
            StartRange(1, Some(-2))
        );

        assert_err_eq!(
            StartRange::<i32>::from_str("x..-2").map_err(|err| err.to_string()),
            "invalid lower bound 'x'"
        );

        assert_err_eq!(
            StartRange::<i32>::from_str("1..x").map_err(|err| err.to_string()),
            "invalid upper bound 'x'"
        );

        assert_err_eq!(
            StartRange::<i32>::from_str("3").map_err(|err| err.to_string()),
            "missing bounds delimiter '..'"
        );

        assert_err_eq!(
            StartRange::<i32>::from_str("1.-2").map_err(|err| err.to_string()),
            "missing bounds delimiter '..'"
        );

        assert_err_eq!(
            StartRange::<i32>::from_str("").map_err(|err| err.to_string()),
            "missing bounds delimiter '..'"
        );
    }
}
