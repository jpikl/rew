use crate::pattern::char::{AsChar, Char};
use crate::pattern::escape::escape_str;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::regex::{add_capture_group_brackets, RegexHolder};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Case {
    pub matcher: RegexHolder,
    pub result: String,
}

#[derive(Debug, PartialEq)]
pub struct RegexSwitch {
    pub cases: Vec<Case>,
    pub default: String,
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
                    let matcher = RegexHolder::try_from(value).map_err(|kind| Error {
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
    use crate::utils::AnyString;
    use regex::Regex;

    mod parse {
        use super::*;

        #[test]
        fn empty() {
            let mut reader = Reader::from("");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Err(Error {
                    kind: ErrorKind::ExpectedSwitch,
                    range: 0..0,
                })
            );
            assert_eq!(reader.position(), 0);
        }

        #[test]
        fn delimiter() {
            let mut reader = Reader::from(":");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Ok(RegexSwitch {
                    cases: Vec::new(),
                    default: String::new(),
                }),
            );
            assert_eq!(reader.position(), 1);
        }

        #[test]
        fn delimiter_delimiter() {
            let mut reader = Reader::from("::");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Err(Error {
                    kind: ErrorKind::SwitchWithoutMatcher(Char::Raw(':'), 0),
                    range: 0..1
                }),
            );
            assert_eq!(reader.position(), 2);
        }

        #[test]
        fn default() {
            let mut reader = Reader::from(":mixed");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Ok(RegexSwitch {
                    cases: Vec::new(),
                    default: String::from("mixed"),
                }),
            );
            assert_eq!(reader.position(), 6);
        }

        #[test]
        fn invalid() {
            let mut reader = Reader::from(":^[a-z+:");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Err(Error {
                    kind: ErrorKind::RegexInvalid(AnyString(String::from(
                        "This string is not compared by assertion"
                    ))),
                    range: 1..7,
                })
            );
            assert_eq!(reader.position(), 8);
        }

        #[test]
        fn matcher() {
            let mut reader = Reader::from(":^[a-z]+$:");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Ok(RegexSwitch {
                    cases: vec![Case {
                        matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                        result: String::new()
                    }],
                    default: String::new(),
                }),
            );
            assert_eq!(reader.position(), 10);
        }

        #[test]
        fn matcher_result() {
            let mut reader = Reader::from(":^[a-z]+$:lower");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Ok(RegexSwitch {
                    cases: vec![Case {
                        matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                        result: String::from("lower")
                    }],
                    default: String::new(),
                }),
            );
            assert_eq!(reader.position(), 15);
        }

        #[test]
        fn matcher_result_delimiter() {
            let mut reader = Reader::from(":^[a-z]+$:lower:");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Ok(RegexSwitch {
                    cases: vec![Case {
                        matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                        result: String::from("lower")
                    }],
                    default: String::new(),
                }),
            );
            assert_eq!(reader.position(), 16);
        }

        #[test]
        fn matcher_delimiter() {
            let mut reader = Reader::from(":^[a-z]+$::");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Ok(RegexSwitch {
                    cases: vec![Case {
                        matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                        result: String::from("")
                    }],
                    default: String::new(),
                }),
            );
            assert_eq!(reader.position(), 11);
        }

        #[test]
        fn matcher_delimiter_delimiter() {
            let mut reader = Reader::from(":^[a-z]+$:::");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Err(Error {
                    kind: ErrorKind::SwitchWithoutMatcher(Char::Raw(':'), 2),
                    range: 10..11
                }),
            );
            assert_eq!(reader.position(), 12);
        }

        #[test]
        fn matcher_result_default() {
            let mut reader = Reader::from(":^[a-z]+$:lower:mixed");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Ok(RegexSwitch {
                    cases: vec![Case {
                        matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                        result: String::from("lower")
                    }],
                    default: String::from("mixed"),
                }),
            );
            assert_eq!(reader.position(), 21);
        }

        #[test]
        fn matcher_result_invalid() {
            let mut reader = Reader::from(":^[a-z]+$:lower:^[A-Z+:");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Err(Error {
                    kind: ErrorKind::RegexInvalid(AnyString(String::from(
                        "This string is not compared by assertion"
                    ))),
                    range: 16..22,
                }),
            );
            assert_eq!(reader.position(), 23);
        }

        #[test]
        fn matcher_result_matcher() {
            let mut reader = Reader::from(":^[a-z]+$:lower:^[A-Z]+$:");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Ok(RegexSwitch {
                    cases: vec![
                        Case {
                            matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                            result: String::from("lower")
                        },
                        Case {
                            matcher: RegexHolder(Regex::new("^[A-Z]+$").unwrap()),
                            result: String::new()
                        }
                    ],
                    default: String::new(),
                }),
            );
            assert_eq!(reader.position(), 25);
        }

        #[test]
        fn matcher_result_matcher_result() {
            let mut reader = Reader::from(":^[a-z]+$:lower:^[A-Z]+$:upper");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Ok(RegexSwitch {
                    cases: vec![
                        Case {
                            matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                            result: String::from("lower")
                        },
                        Case {
                            matcher: RegexHolder(Regex::new("^[A-Z]+$").unwrap()),
                            result: String::from("upper")
                        }
                    ],
                    default: String::new(),
                }),
            );
            assert_eq!(reader.position(), 30);
        }

        #[test]
        fn matcher_result_matcher_result_delimiter() {
            let mut reader = Reader::from(":^[a-z]+$:lower:^[A-Z]+$:upper:");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Ok(RegexSwitch {
                    cases: vec![
                        Case {
                            matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                            result: String::from("lower")
                        },
                        Case {
                            matcher: RegexHolder(Regex::new("^[A-Z]+$").unwrap()),
                            result: String::from("upper")
                        }
                    ],
                    default: String::new(),
                }),
            );
            assert_eq!(reader.position(), 31);
        }

        #[test]
        fn matcher_result_matcher_result_default() {
            let mut reader = Reader::from(":^[a-z]+$:lower:^[A-Z]+$:upper:mixed");
            assert_eq!(
                RegexSwitch::parse(&mut reader),
                Ok(RegexSwitch {
                    cases: vec![
                        Case {
                            matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                            result: String::from("lower")
                        },
                        Case {
                            matcher: RegexHolder(Regex::new("^[A-Z]+$").unwrap()),
                            result: String::from("upper")
                        }
                    ],
                    default: String::from("mixed"),
                }),
            );
            assert_eq!(reader.position(), 36);
        }
    }

    mod eval {
        use super::*;

        #[test]
        fn empty_default() {
            let switch = RegexSwitch {
                cases: Vec::new(),
                default: String::new(),
            };
            assert_eq!(switch.eval(""), "");
            assert_eq!(switch.eval("abc"), "");
        }

        #[test]
        fn nonempty_default() {
            let switch = RegexSwitch {
                cases: Vec::new(),
                default: String::from("default"),
            };
            assert_eq!(switch.eval(""), "default");
            assert_eq!(switch.eval("abc"), "default");
        }

        #[test]
        fn cases_nonempty_default() {
            let switch = RegexSwitch {
                cases: vec![
                    Case {
                        matcher: RegexHolder(Regex::new("\\d\\d").unwrap()),
                        result: String::from("contains consecutive digits"),
                    },
                    Case {
                        matcher: RegexHolder(Regex::new("\\d").unwrap()),
                        result: String::from("contains digit"),
                    },
                    Case {
                        matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                        result: String::from("all lowercase"),
                    },
                ],
                default: String::from("other"),
            };
            assert_eq!(switch.eval(""), "other");
            assert_eq!(switch.eval("a12b"), "contains consecutive digits");
            assert_eq!(switch.eval("a1b"), "contains digit");
            assert_eq!(switch.eval("ab"), "all lowercase");
            assert_eq!(switch.eval("Ab"), "other");
        }

        #[test]
        fn cases_nonempty_default_captures() {
            let switch = RegexSwitch {
                cases: vec![
                    Case {
                        matcher: RegexHolder(Regex::new("(\\d)(\\d)").unwrap()),
                        result: String::from("contains consecutive digits $1 and $2"),
                    },
                    Case {
                        matcher: RegexHolder(Regex::new("\\d").unwrap()),
                        result: String::from("contains digit $0"),
                    },
                    Case {
                        matcher: RegexHolder(Regex::new("^.*([A-Z]).*$").unwrap()),
                        result: String::from("first uppercase letter of '$0' is '$1'"),
                    },
                ],
                default: String::from("$0, $1 are not capture groups"),
            };
            assert_eq!(switch.eval(""), "$0, $1 are not capture groups");
            assert_eq!(switch.eval("a34b"), "contains consecutive digits 3 and 4");
            assert_eq!(switch.eval("a3b"), "contains digit 3");
            assert_eq!(switch.eval("aBc"), "first uppercase letter of 'aBc' is 'B'");
            assert_eq!(switch.eval("world"), "$0, $1 are not capture groups");
        }
    }

    mod display {
        use super::*;
        use indoc::indoc;

        #[test]
        fn empty_default() {
            assert_eq!(
                RegexSwitch {
                    cases: Vec::new(),
                    default: String::new()
                }
                .to_string(),
                "constant output ''"
            );
        }

        #[test]
        fn nonempty_default() {
            assert_eq!(
                RegexSwitch {
                    cases: Vec::new(),
                    default: String::from("abc")
                }
                .to_string(),
                "constant output 'abc'"
            );
        }

        #[test]
        fn single_case() {
            assert_eq!(
                RegexSwitch {
                    cases: vec![Case {
                        matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                        result: String::from("lower")
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
                            matcher: RegexHolder(Regex::new("^[a-z]+$").unwrap()),
                            result: String::from("lower")
                        },
                        Case {
                            matcher: RegexHolder(Regex::new("^[A-Z]+$").unwrap()),
                            result: String::from("upper")
                        }
                    ],
                    default: String::from("mixed")
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
