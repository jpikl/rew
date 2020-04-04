use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorType {
    ExpectedNumber,
    ExpectedPattern,
    ExpectedPipeOrExprEnd,
    ExpectedRange,
    ExpectedSubstitution,
    ExpectedTransform,
    ExpectedVariable,
    RangeEndBeforeStart,
    RangeZeroIndex,
    RegexCaptureGroupOverflow,
    RegexCaptureGroupZero,
    SubstituteNoValue,
    UnexpectedCharacters,
    UnexpectedExprEnd,
    UnknownTransform,
    UnknownVariable,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use ErrorType::*;
        let description = match self {
            ExpectedNumber => "Expected number",
            ExpectedPattern => "Expected pattern",
            ExpectedPipeOrExprEnd => "Expected pipe or end of expression",
            ExpectedRange => "Expected range",
            ExpectedSubstitution => "Expected substitution",
            ExpectedTransform => "Expected transformation",
            ExpectedVariable => "Expected variable",
            RangeEndBeforeStart => "Range end cannot precede start",
            RangeZeroIndex => "Range indices starts from 1",
            RegexCaptureGroupZero => "Regex capture groups starts from 1",
            RegexCaptureGroupOverflow => "Value exceeded number of regex capture groups",
            SubstituteNoValue => "No value to substitute",
            UnexpectedCharacters => "Unexpected characters",
            UnexpectedExprEnd => "Unexpected end of expression",
            UnknownTransform => "Unknown transformation",
            UnknownVariable => "Unknown variable",
        };
        write!(formatter, "{}", description)
    }
}
