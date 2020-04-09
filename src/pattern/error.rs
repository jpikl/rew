use crate::pattern::char::Char;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorType {
    ExpectedNumber,
    ExpectedPattern,
    ExpectedPipeOrExprEnd(Char),
    ExpectedRange,
    ExpectedSubstitution,
    ExpectedTransform,
    ExpectedVariable,
    ExprStartInsideExpr,
    RangeEndBeforeStart(usize, usize),
    RangeInvalid(String),
    RangeUnexpectedChars(String),
    RangeZeroIndex,
    RegexZeroCaptureGroup,
    SubstituteWithoutValue(Char),
    UnknownTransform(Char),
    UnknownVariable(Char),
    UnmatchedExprEnd,
    UnterminatedExprStart,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use ErrorType::*;
        match self {
            ExpectedNumber => write!(formatter, "Expected number"),
            ExpectedPattern => write!(formatter, "Expected pattern but got empty string"),
            ExpectedPipeOrExprEnd(char) => {
                write!(formatter, "Expected '|' or closing '}}' but got {}", char)
            }
            ExpectedRange => write!(formatter, "Transformation requires range as a parameter"),
            ExpectedSubstitution => write!(
                formatter,
                "Transformation requires substitution as a parameter"
            ),
            ExpectedTransform => write!(formatter, "Expected transformation after '|'"),
            ExpectedVariable => write!(formatter, "Expected variable after '{{'"),
            ExprStartInsideExpr => writeln!(formatter, "Unescaped '{{' inside expression"),
            RangeEndBeforeStart(end, start) => write!(
                formatter,
                "Range end ({}) cannot precede its start ({})",
                end, start
            ),
            RangeInvalid(value) => write!(formatter, "Invalid range '{}'", value),
            RangeUnexpectedChars(value) => write!(
                formatter,
                "Unexpected characters '{}' in range parameter",
                value
            ),
            RangeZeroIndex => write!(formatter, "Range indices start from 1, not 0"),
            RegexZeroCaptureGroup => write!(formatter, "Regex capture groups starts from 1, not 0"),
            SubstituteWithoutValue(separator) => write!(
                formatter,
                "Substitution ({} is separator) has no value",
                separator
            ),
            UnknownTransform(Char::Raw(char)) => {
                write!(formatter, "Unknown transformation '{}'", char)
            }
            UnknownTransform(char) => write!(formatter, "Expected transformation but got {}", char),
            UnknownVariable(Char::Raw(char)) => write!(formatter, "Unknown variable '{}'", char),
            UnknownVariable(char) => write!(formatter, "Expected variable but got {}", char),
            UnmatchedExprEnd => write!(
                formatter,
                "End of expression'}}' does not have matching '{{'"
            ),
            UnterminatedExprStart => write!(formatter, "Unterminated start of expression '{{'"),
        }
    }
}
