use crate::pattern::char::{AsChar, Char};
use crate::pattern::escape::escape_str;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::regex::{add_capture_group_brackets, RegexHolder};
use std::borrow::Cow;
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct RegexSwitch {
    pub cases: Vec<Case>,
    pub default: String,
}

#[derive(Debug, PartialEq)]
pub struct Case {
    pub matcher: RegexHolder,
    pub result: String,
}

impl RegexSwitch {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        if let Some(delimiter) = reader.read().cloned() {
            let mut cases = Vec::new();

            loop {
                let value_start = reader.position();
                let value = reader.read_until(&delimiter);
                let value_end = value_start + value.len_utf8();
                let value = value.to_string();

                if reader.position() > value_end {
                    if value.is_empty() {
                        let delimiter_start = value_start - delimiter.len_utf8();
                        let delimiter_index = 2 * cases.len();

                        return Err(Error {
                            kind: ErrorKind::SwitchWithoutMatcher(delimiter, delimiter_index),
                            range: delimiter_start..value_end,
                        });
                    }

                    // There was a delimiter after value
                    let matcher = value.try_into().map_err(|kind| Error {
                        kind,
                        range: value_start..value_end,
                    })?;

                    let result = reader.read_until(&delimiter).to_string();
                    cases.push(Case { matcher, result })
                } else {
                    return Ok(RegexSwitch {
                        cases,
                        default: value,
                    });
                }
            }
        } else {
            Err(Error {
                kind: ErrorKind::ExpectedSwitch,
                range: reader.position()..reader.end(),
            })
        }
    }

    pub fn eval<'a>(&'a self, value: &'a str) -> Cow<'a, str> {
        for case in &self.cases {
            if let Some(result) = case.matcher.find(value) {
                return if case.result.contains('$') {
                    case.matcher.replace(
                        &value[result.range()],
                        add_capture_group_brackets(&case.result).as_ref(),
                    )
                } else {
                    Cow::from(&case.result)
                };
            }
        }
        Cow::from(&self.default)
    }
}

impl fmt::Display for RegexSwitch {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.cases.is_empty() {
            write!(formatter, "constant output '{}'", escape_str(&self.default))
        } else {
            writeln!(formatter, "variable output:")?;
            for (index, case) in self.cases.iter().enumerate() {
                write!(formatter, "\n    ")?;
                if index > 0 {
                    write!(formatter, "else ")?;
                }
                write!(
                    formatter,
                    "if input matches '{}'\n        output is '{}'",
                    case.matcher,
                    escape_str(&case.result)
                )?;
            }
            write!(
                formatter,
                "\n    else\n        output is '{}'",
                escape_str(&self.default)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{AnyString, ByteRange};
    use test_case::test_case;

    #[test_case("", ErrorKind::ExpectedSwitch, 0..0; "empty")]
    #[test_case("::", ErrorKind::SwitchWithoutMatcher(Char::Raw(':'), 0), 0..1; "delimiter delimiter")]
    #[test_case(":^[a-z+:", ErrorKind::RegexInvalid(AnyString::any()), 1..7; "invalid")]
    #[test_case(":^[a-z]+$:::", ErrorKind::SwitchWithoutMatcher(Char::Raw(':'), 2), 10..11; "matcher delimiter delimiter")]
    #[test_case(":^[a-z]+$:lower:^[A-Z+:", ErrorKind::RegexInvalid(AnyString::any()), 16..22; "matcher result invalid")]
    fn parse_err(input: &str, kind: ErrorKind, range: ByteRange) {
        assert_eq!(
            RegexSwitch::parse(&mut Reader::from(input)),
            Err(Error { kind, range })
        );
    }

    #[test_case(":", &[], ""; "delimiter")]
    #[test_case(":mixed", &[], "mixed"; "default")]
    #[test_case(":^[a-z]+$:", &[("^[a-z]+$", "")], ""; "matcher")]
    #[test_case(":^[a-z]+$::", &[("^[a-z]+$", "")], ""; "matcher delimiter")]
    #[test_case(":^[a-z]+$:lower", &[("^[a-z]+$", "lower")], ""; "matcher result")]
    #[test_case(":^[a-z]+$:lower:", &[("^[a-z]+$", "lower")], ""; "matcher result delimiter")]
    #[test_case(":^[a-z]+$:lower:mixed", &[("^[a-z]+$", "lower")], "mixed"; "matcher result default")]
    #[test_case(":^[a-z]+$:lower:^[A-Z]+$:", &[("^[a-z]+$", "lower"), ("^[A-Z]+$", "")], ""; "matcher result matcher")]
    #[test_case(":^[a-z]+$:lower:^[A-Z]+$:upper", &[("^[a-z]+$", "lower"), ("^[A-Z]+$", "upper")], ""; "matcher result matcher result")]
    #[test_case(":^[a-z]+$:lower:^[A-Z]+$:upper:", &[("^[a-z]+$", "lower"), ("^[A-Z]+$", "upper")], ""; "matcher result matcher result delimiter")]
    #[test_case(":^[a-z]+$:lower:^[A-Z]+$:upper:mixed", &[("^[a-z]+$", "lower"), ("^[A-Z]+$", "upper")], "mixed"; "matcher result matcher result default")]
    fn parse_ok(input: &str, cases: &[(&str, &str)], default: &str) {
        assert_eq!(
            RegexSwitch::parse(&mut Reader::from(input)),
            Ok(RegexSwitch {
                cases: cases
                    .iter()
                    .map(|(matcher, result)| Case {
                        matcher: (*matcher).into(),
                        result: (*result).into()
                    })
                    .collect(),
                default: default.into(),
            })
        );
    }

    mod eval {
        use super::*;
        use test_case::test_case;

        #[test_case("", ""; "empty")]
        #[test_case("abc", ""; "nonempty")]
        fn empty_default(input: &str, output: &str) {
            assert_eq!(
                RegexSwitch {
                    cases: Vec::new(),
                    default: String::new(),
                }
                .eval(input),
                output
            );
        }

        #[test_case("", "default"; "empty")]
        #[test_case("abc", "default"; "nonempty")]
        fn nonempty_default(input: &str, output: &str) {
            assert_eq!(
                RegexSwitch {
                    cases: Vec::new(),
                    default: "default".into(),
                }
                .eval(input),
                output
            );
        }

        #[test_case("", "other"; "default empty")]
        #[test_case("Ab", "other"; "default nonempty")]
        #[test_case("a12b", "contains consecutive digits"; "first matcher")]
        #[test_case("a1b", "contains digit"; "second matcher")]
        #[test_case("ab", "all lowercase"; "third matcher")]
        fn cases_nonempty_default(input: &str, output: &str) {
            assert_eq!(
                RegexSwitch {
                    cases: vec![
                        Case {
                            matcher: "\\d\\d".into(),
                            result: "contains consecutive digits".into(),
                        },
                        Case {
                            matcher: "\\d".into(),
                            result: "contains digit".into(),
                        },
                        Case {
                            matcher: "^[a-z]+$".into(),
                            result: "all lowercase".into(),
                        },
                    ],
                    default: "other".into(),
                }
                .eval(input),
                output
            );
        }

        #[test_case("", "$0, $1 are not capture groups"; "default empty")]
        #[test_case("world", "$0, $1 are not capture groups"; "default nonempty")]
        #[test_case("a34b", "contains consecutive digits 3 and 4"; "first matcher")]
        #[test_case("a3b", "contains digit 3"; "second matcher")]
        #[test_case("aBc", "first uppercase letter of 'aBc' is 'B'"; "third matcher")]
        fn cases_nonempty_default_captures(input: &str, output: &str) {
            assert_eq!(
                RegexSwitch {
                    cases: vec![
                        Case {
                            matcher: "(\\d)(\\d)".into(),
                            result: "contains consecutive digits $1 and $2".into(),
                        },
                        Case {
                            matcher: "\\d".into(),
                            result: "contains digit $0".into(),
                        },
                        Case {
                            matcher: "^.*([A-Z]).*$".into(),
                            result: "first uppercase letter of '$0' is '$1'".into(),
                        },
                    ],
                    default: "$0, $1 are not capture groups".into(),
                }
                .eval(input),
                output
            );
        }
    }

    mod display {
        use super::*;
        use indoc::indoc;
        use test_case::test_case;

        #[test_case("", "constant output ''"; "empty")]
        #[test_case("abc", "constant output 'abc'"; "nonempty")]
        fn empty_default(default: &str, result: &str) {
            assert_eq!(
                RegexSwitch {
                    cases: Vec::new(),
                    default: default.into()
                }
                .to_string(),
                result
            );
        }

        #[test]
        fn single_case() {
            assert_eq!(
                RegexSwitch {
                    cases: vec![Case {
                        matcher: "^[a-z]+$".into(),
                        result: "lower".into()
                    }],
                    default: String::new()
                }
                .to_string(),
                indoc! {"
                    variable output:
                    
                        if input matches '^[a-z]+$'
                            output is 'lower'
                        else
                            output is ''"
                }
            );
        }

        #[test]
        fn multiple_cases() {
            assert_eq!(
                RegexSwitch {
                    cases: vec![
                        Case {
                            matcher: "^[a-z]+$".into(),
                            result: "lower".into()
                        },
                        Case {
                            matcher: "^[A-Z]+$".into(),
                            result: "upper".into()
                        }
                    ],
                    default: "mixed".into()
                }
                .to_string(),
                indoc! {"
                    variable output:
                    
                        if input matches '^[a-z]+$'
                            output is 'lower'
                        else if input matches '^[A-Z]+$'
                            output is 'upper'
                        else
                            output is 'mixed'"
                }
            );
        }
    }
}
