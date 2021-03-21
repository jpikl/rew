use crate::pattern::char::{AsChar, Char};
use crate::pattern::column::Column;
use crate::pattern::index::IndexRange;
use crate::pattern::integer::parse_integer;
use crate::pattern::number::NumberRange;
use crate::pattern::padding::Padding;
use crate::pattern::reader::Reader;
use crate::pattern::regex::RegexHolder;
use crate::pattern::repetition::Repetition;
use crate::pattern::substitution::{EmptySubstitution, RegexSubstitution, StringSubstitution};
use crate::pattern::switch::RegexSwitch;
use crate::pattern::symbols::RANGE_DELIMITER;
use crate::pattern::uuid::random_uuid;
use crate::pattern::{eval, parse, path};
use std::fmt;
use unidecode::unidecode;

#[derive(Debug, PartialEq)]
pub enum Filter {
    WorkingDir,
    AbsolutePath,
    RelativePath,
    NormalizedPath,
    CanonicalPath,
    ParentDirectory,
    RemoveLastName,
    FileName,
    LastName,
    BaseName,
    RemoveExtension,
    Extension,
    ExtensionWithDot,
    EnsureTrailingDirSeparator,
    RemoveTrailingDirSeparator,
    Substring(IndexRange),
    SubstringBackward(IndexRange),
    GetColumn(Column),
    GetColumnBackward(Column),
    ReplaceFirst(StringSubstitution),
    ReplaceAll(StringSubstitution),
    ReplaceEmpty(EmptySubstitution),
    RegexMatch(RegexHolder),
    RegexReplaceFirst(RegexSubstitution),
    RegexReplaceAll(RegexSubstitution),
    RegexSwitch(RegexSwitch),
    RegexCapture(usize),
    Trim,
    ToLowercase,
    ToUppercase,
    ToAscii,
    RemoveNonAscii,
    LeftPad(Padding),
    RightPad(Padding),
    Repeat(Repetition),
    LocalCounter,
    GlobalCounter,
    RandomNumber(NumberRange),
    RandomUuid,
}

impl Filter {
    pub fn parse(reader: &mut Reader<Char>, config: &parse::Config) -> parse::Result<Self> {
        let position = reader.position();

        if let Some(char) = reader.read() {
            match char.as_char() {
                'w' => Ok(Self::WorkingDir),
                'a' => Ok(Self::AbsolutePath),
                'A' => Ok(Self::RelativePath),
                'p' => Ok(Self::NormalizedPath),
                'P' => Ok(Self::CanonicalPath),
                'd' => Ok(Self::ParentDirectory),
                'D' => Ok(Self::RemoveLastName),
                'f' => Ok(Self::FileName),
                'F' => Ok(Self::LastName),
                'b' => Ok(Self::BaseName),
                'B' => Ok(Self::RemoveExtension),
                'e' => Ok(Self::Extension),
                'E' => Ok(Self::ExtensionWithDot),
                'z' => Ok(Self::EnsureTrailingDirSeparator),
                'Z' => Ok(Self::RemoveTrailingDirSeparator),
                '#' => {
                    if reader.read_expected(RANGE_DELIMITER) {
                        Ok(Self::SubstringBackward(IndexRange::parse(reader)?))
                    } else {
                        Ok(Self::Substring(IndexRange::parse(reader)?))
                    }
                }
                '&' => {
                    let separator = &config.separator;
                    if reader.read_expected(RANGE_DELIMITER) {
                        Ok(Self::GetColumnBackward(Column::parse(reader, separator)?))
                    } else {
                        Ok(Self::GetColumn(Column::parse(reader, separator)?))
                    }
                }
                'r' => Ok(Self::ReplaceFirst(StringSubstitution::parse(reader)?)),
                'R' => Ok(Self::ReplaceAll(StringSubstitution::parse(reader)?)),
                '?' => Ok(Self::ReplaceEmpty(EmptySubstitution::parse(reader)?)),
                '=' => Ok(Self::RegexMatch(RegexHolder::parse(reader)?)),
                's' => Ok(Self::RegexReplaceFirst(RegexSubstitution::parse(reader)?)),
                'S' => Ok(Self::RegexReplaceAll(RegexSubstitution::parse(reader)?)),
                '@' => Ok(Self::RegexSwitch(RegexSwitch::parse(reader)?)),
                '$' => Ok(Self::RegexCapture(parse_integer(reader)?)),
                't' => Ok(Self::Trim),
                'v' => Ok(Self::ToLowercase),
                '^' => Ok(Self::ToUppercase),
                'i' => Ok(Self::ToAscii),
                'I' => Ok(Self::RemoveNonAscii),
                '<' => Ok(Self::LeftPad(Padding::parse(reader, '<')?)),
                '>' => Ok(Self::RightPad(Padding::parse(reader, '>')?)),
                '*' => Ok(Self::Repeat(Repetition::parse(reader)?)),
                'c' => Ok(Self::LocalCounter),
                'C' => Ok(Self::GlobalCounter),
                'u' => Ok(Self::RandomNumber(NumberRange::parse(reader)?)),
                'U' => Ok(Self::RandomUuid),
                _ => Err(parse::Error {
                    kind: parse::ErrorKind::UnknownFilter(char.clone()),
                    range: position..reader.position(),
                }),
            }
        } else {
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedFilter,
                range: position..reader.end(),
            })
        }
    }

    pub fn eval(&self, mut value: String, context: &eval::Context) -> eval::BaseResult<String> {
        match self {
            Self::WorkingDir => path::to_string(context.working_dir),
            Self::AbsolutePath => path::to_absolute(value, context.working_dir),
            Self::RelativePath => path::to_relative(value, context.working_dir),
            Self::NormalizedPath => path::normalize(&value),
            Self::CanonicalPath => path::canonicalize(value, context.working_dir),
            Self::ParentDirectory => path::get_parent_directory(value),
            Self::RemoveLastName => path::remove_last_name(value),
            Self::FileName => path::get_file_name(&value),
            Self::LastName => path::get_last_name(&value),
            Self::BaseName => path::get_base_name(&value),
            Self::RemoveExtension => path::remove_extension(value),
            Self::Extension => path::get_extension(&value),
            Self::ExtensionWithDot => path::get_extension_with_dot(&value),
            Self::EnsureTrailingDirSeparator => Ok(path::ensure_trailing_dir_separator(value)),
            Self::RemoveTrailingDirSeparator => Ok(path::remove_trailing_dir_separator(value)),
            Self::Substring(range) => Ok(range.substr(value)),
            Self::SubstringBackward(range) => Ok(range.substr_back(value)),
            Self::GetColumn(column) => Ok(column.get(&value).to_string()),
            Self::GetColumnBackward(column) => Ok(column.get_backward(&value).to_string()),
            Self::ReplaceFirst(substitution) => Ok(substitution.replace_first(&value)),
            Self::ReplaceAll(substitution) => Ok(substitution.replace_all(&value)),
            Self::ReplaceEmpty(substitution) => Ok(substitution.replace(value)),
            Self::RegexMatch(regex) => Ok(regex.first_match(&value)),
            Self::RegexReplaceFirst(substitution) => Ok(substitution.replace_first(&value)),
            Self::RegexReplaceAll(substitution) => Ok(substitution.replace_all(&value)),
            Self::RegexSwitch(switch) => Ok(switch.eval(&value).to_string()),
            Self::RegexCapture(number) => Ok(context.regex_capture(*number).to_string()),
            Self::Trim => Ok(value.trim().to_string()),
            Self::ToLowercase => Ok(value.to_lowercase()),
            Self::ToUppercase => Ok(value.to_uppercase()),
            Self::ToAscii => Ok(unidecode(&value)),
            Self::RemoveNonAscii => {
                value.retain(|ch| ch.is_ascii());
                Ok(value)
            }
            Self::LeftPad(padding) => Ok(padding.apply_left(value)),
            Self::RightPad(padding) => Ok(padding.apply_right(value)),
            Self::Repeat(repetition) => Ok(repetition.expand()),
            Self::LocalCounter => Ok(context.local_counter.to_string()),
            Self::GlobalCounter => Ok(context.global_counter.to_string()),
            Self::RandomNumber(interval) => Ok(interval.random().to_string()),
            Self::RandomUuid => Ok(random_uuid()),
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WorkingDir => write!(formatter, "Working directory"),
            Self::AbsolutePath => write!(formatter, "Absolute path"),
            Self::RelativePath => write!(formatter, "Relative path"),
            Self::NormalizedPath => write!(formatter, "Normalized path"),
            Self::CanonicalPath => write!(formatter, "Canonical path"),
            Self::ParentDirectory => write!(formatter, "Parent directory"),
            Self::RemoveLastName => write!(formatter, "Remove last name"),
            Self::FileName => write!(formatter, "File name"),
            Self::LastName => write!(formatter, "Last name"),
            Self::BaseName => write!(formatter, "Base name"),
            Self::RemoveExtension => write!(formatter, "Remove extension"),
            Self::Extension => write!(formatter, "Extension"),
            Self::ExtensionWithDot => write!(formatter, "Extension with dot"),
            Self::EnsureTrailingDirSeparator => {
                write!(formatter, "Ensure trailing directory separator")
            }
            Self::RemoveTrailingDirSeparator => {
                write!(formatter, "Remove trailing directory separator")
            }
            Self::Substring(range) => write!(formatter, "Substring from {}", range),
            Self::SubstringBackward(range) => {
                write!(formatter, "Substring from {} backward", range)
            }
            Self::GetColumn(column) => write!(formatter, "Get {}", column),
            Self::GetColumnBackward(column) => write!(formatter, "Get {} backward", column),
            Self::ReplaceFirst(substitution) => write!(formatter, "Replace first {}", substitution),
            Self::ReplaceAll(substitution) => write!(formatter, "Replace all {}", substitution),
            Self::ReplaceEmpty(substitution) => {
                write!(formatter, "Replace {}", substitution)
            }
            Self::RegexMatch(substitution) => {
                write!(formatter, "Match of regular expression '{}'", substitution)
            }
            Self::RegexReplaceFirst(substitution) => write!(
                formatter,
                "Replace first match of regular expression {}",
                substitution
            ),
            Self::RegexReplaceAll(substitution) => write!(
                formatter,
                "Replace all matches of regular expression {}",
                substitution
            ),
            Self::RegexSwitch(switch) => {
                write!(formatter, "Regular expression switch with {}", switch)
            }
            Self::RegexCapture(number) => {
                write!(
                    formatter,
                    "Capture group #{} of a global regular expression",
                    number
                )
            }
            Self::Trim => write!(formatter, "Trim"),
            Self::ToLowercase => write!(formatter, "To lowercase"),
            Self::ToUppercase => write!(formatter, "To uppercase"),
            Self::ToAscii => write!(formatter, "To ASCII"),
            Self::RemoveNonAscii => write!(formatter, "Remove non-ASCII"),
            Self::LeftPad(padding) => write!(formatter, "Left pad with {}", padding),
            Self::RightPad(padding) => write!(formatter, "Right pad with {}", padding),
            Self::Repeat(repetition) => write!(formatter, "Repeat {}", repetition),
            Self::LocalCounter => write!(formatter, "Local counter"),
            Self::GlobalCounter => write!(formatter, "Global counter"),
            Self::RandomNumber(interval) => write!(formatter, "Random number from {}", interval),
            Self::RandomUuid => write!(formatter, "Random UUID"),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate regex;

    use super::Filter;
    use crate::pattern::char::Char;
    use crate::pattern::column::Column;
    use crate::pattern::index::{Index, IndexRange};
    use crate::pattern::number::{Number, NumberRange};
    use crate::pattern::padding::Padding;
    use crate::pattern::parse::Separator;
    use crate::pattern::range::Range;
    use crate::pattern::repetition::Repetition;
    use crate::pattern::substitution::{
        EmptySubstitution, RegexSubstitution, StringSubstitution, Substitution,
    };
    use crate::pattern::switch::{Case, RegexSwitch};
    use crate::pattern::symbols::DEFAULT_SEPARATOR;
    use crate::utils::Empty;
    use crate::utils::{AnyString, ByteRange};
    use test_case::test_case;

    mod parse {
        use super::*;
        use crate::pattern::parse::{Config, Error, ErrorKind};
        use crate::pattern::reader::Reader;
        use test_case::test_case;

        #[test_case("", ErrorKind::ExpectedFilter, 0..0; "empty")]
        #[test_case("-", ErrorKind::UnknownFilter(Char::Raw('-')), 0..1; "unknown")]
        #[test_case("#", ErrorKind::ExpectedRange, 1..1; "substring expected range")]
        #[test_case("#-", ErrorKind::ExpectedRange, 2..2; "substring backward expected range")]
        #[test_case("&", ErrorKind::ExpectedNumber, 1..1; "column expected number")]
        #[test_case("&-", ErrorKind::ExpectedNumber, 2..2; "column backward expected number")]
        #[test_case("&1:", ErrorKind::ExpectedColumnSeparator, 3..3; "column expected separator")]
        #[test_case("&-1:", ErrorKind::ExpectedColumnSeparator, 4..4; "column backward expected separator")]
        #[test_case("&1/[0-9", ErrorKind::RegexInvalid(AnyString::any()), 3..7; "column regex invalid")]
        #[test_case("&-1/[0-9", ErrorKind::RegexInvalid(AnyString::any()), 4..8; "column backward regex invalid")]
        #[test_case("r", ErrorKind::ExpectedSubstitution, 1..1; "replace expected substitution")]
        #[test_case("R", ErrorKind::ExpectedSubstitution, 1..1; "replace all expected substitution")]
        #[test_case("=", ErrorKind::ExpectedRegex, 1..1; "regex match expected regex")]
        #[test_case("=[0-9", ErrorKind::RegexInvalid(AnyString::any()), 1..5; "regex match invalid regex")]
        #[test_case("s", ErrorKind::ExpectedSubstitution, 1..1; "regex replace expected substitution")]
        #[test_case("s/[0-9/", ErrorKind::RegexInvalid(AnyString::any()), 2..6; "regex replace invalid regex")]
        #[test_case("S", ErrorKind::ExpectedSubstitution, 1..1; "regex replace all expected substitution")]
        #[test_case("S/[0-9/", ErrorKind::RegexInvalid(AnyString::any()), 2..6; "regex replace all invalid regex")]
        #[test_case("@:[0-9:digit:alpha", ErrorKind::RegexInvalid(AnyString::any()), 2..6; "regex switch invalid regex")]
        #[test_case("$", ErrorKind::ExpectedNumber, 1..1; "regex capture expected number")]
        #[test_case("<x", ErrorKind::PaddingPrefixInvalid('<', Some(Char::Raw('x'))), 1..2; "padding left prefix invalid")]
        #[test_case(">y", ErrorKind::PaddingPrefixInvalid('>', Some(Char::Raw('y'))), 1..2; "padding right prefix invalid")]
        fn err(input: &str, kind: ErrorKind, range: ByteRange) {
            assert_eq!(
                Filter::parse(&mut Reader::from(input), &Config::fixture()),
                Err(Error { kind, range }),
            )
        }

        #[test_case("w", Filter::WorkingDir; "working dir")]
        #[test_case("a", Filter::AbsolutePath; "absolute path")]
        #[test_case("A", Filter::RelativePath; "relative path")]
        #[test_case("p", Filter::NormalizedPath; "normalized path")]
        #[test_case("P", Filter::CanonicalPath; "canonical path")]
        #[test_case("d", Filter::ParentDirectory; "parent directory")]
        #[test_case("D", Filter::RemoveLastName; "remove last name")]
        #[test_case("f", Filter::FileName; "file name")]
        #[test_case("F", Filter::LastName; "last name")]
        #[test_case("b", Filter::BaseName; "base name")]
        #[test_case("B", Filter::RemoveExtension; "remove extension")]
        #[test_case("e", Filter::Extension; "extension")]
        #[test_case("E", Filter::ExtensionWithDot; "extension with dot")]
        #[test_case("z", Filter::EnsureTrailingDirSeparator; "ensure trailing separator")]
        #[test_case("Z", Filter::RemoveTrailingDirSeparator; "remove trailing separator")]
        #[test_case("#2", Filter::Substring(index_range_at()); "substring at")]
        #[test_case("#2-", Filter::Substring(index_range_from()); "substring from")]
        #[test_case("#2-3", Filter::Substring(index_range_between()); "substring between")]
        #[test_case("#2+3", Filter::Substring(index_range_length()); "substring length")]
        #[test_case("#-2", Filter::SubstringBackward(index_range_at()); "substring backward at")]
        #[test_case("#-2-", Filter::SubstringBackward(index_range_from()); "substring backward from")]
        #[test_case("#-2-3", Filter::SubstringBackward(index_range_between()); "substring backward between")]
        #[test_case("#-2+3", Filter::SubstringBackward(index_range_length()); "substring backward length")]
        #[test_case("&2", Filter::GetColumn(column_global()); "column global separator")]
        #[test_case("&2:,", Filter::GetColumn(column_string()); "column string separator")]
        #[test_case("&2/[, ]+", Filter::GetColumn(column_regex()); "column regex separator")]
        #[test_case("&-2", Filter::GetColumnBackward(column_global()); "column backward global separator")]
        #[test_case("&-2:,", Filter::GetColumnBackward(column_string()); "column backward string separator")]
        #[test_case("&-2/[, ]+", Filter::GetColumnBackward(column_regex()); "column backward regex separator")]
        #[test_case("r/ab", Filter::ReplaceFirst(substitution_string_1()); "remove first")]
        #[test_case("r/ab/x", Filter::ReplaceFirst(substitution_string_2()); "replace first")]
        #[test_case("R/ab", Filter::ReplaceAll(substitution_string_1()); "remove all")]
        #[test_case("R/ab/x", Filter::ReplaceAll(substitution_string_2()); "replace all")]
        #[test_case("?x", Filter::ReplaceEmpty(substitution_empty()); "replace empty")]
        #[test_case("=[0-9]+", Filter::RegexMatch("[0-9]+".into()); "regex match")]
        #[test_case("s/[0-9]+", Filter::RegexReplaceFirst(substitution_regex_1()); "regex remove first")]
        #[test_case("s/[0-9]+/x", Filter::RegexReplaceFirst(substitution_regex_2()); "regex replace first")]
        #[test_case("S/[0-9]+", Filter::RegexReplaceAll(substitution_regex_1()); "regex remove all")]
        #[test_case("S/[0-9]+/x", Filter::RegexReplaceAll(substitution_regex_2()); "regex replace all")]
        #[test_case("@:[0-9]+:digit:alpha", Filter::RegexSwitch(regex_switch()); "regex switch ")]
        #[test_case("$0", Filter::RegexCapture(0); "regex capture 0")]
        #[test_case("$10", Filter::RegexCapture(10); "regex capture 10")]
        #[test_case("t", Filter::Trim; "trim")]
        #[test_case("v", Filter::ToLowercase; "to lowercase")]
        #[test_case("^", Filter::ToUppercase; "to uppercase")]
        #[test_case("i", Filter::ToAscii; "to ascii")]
        #[test_case("I", Filter::RemoveNonAscii; "remove non-ascii")]
        #[test_case("<<abcd", Filter::LeftPad(padding_fixed()); "left pad fixed")]
        #[test_case("<2:abc", Filter::LeftPad(padding_repeated()); "left pad repeated")]
        #[test_case(">>abcd", Filter::RightPad(padding_fixed()); "right pad fixed")]
        #[test_case(">2:abc", Filter::RightPad(padding_repeated()); "right pad repeated")]
        #[test_case("*2:abc", Filter::Repeat(repetition()); "repetition ")]
        #[test_case("c", Filter::LocalCounter; "local counter")]
        #[test_case("C", Filter::GlobalCounter; "global counter")]
        #[test_case("u", Filter::RandomNumber(number_range_full()); "random number")]
        #[test_case("u2-", Filter::RandomNumber(number_range_from()); "random number from")]
        #[test_case("u2-10", Filter::RandomNumber(number_range_between()); "random number between")]
        #[test_case("U", Filter::RandomUuid; "random uuid")]
        fn ok(input: &str, filter: Filter) {
            assert_eq!(
                Filter::parse(&mut Reader::from(input), &Config::fixture()),
                Ok(filter),
            )
        }

        #[test_case("a_", 1; "no params")]
        #[test_case("#1-2_", 4; "with params")]
        #[test_case("#_", 1; "error params")]
        fn keep_chars_after(input: &str, position: usize) {
            let mut reader = Reader::from(input);
            Filter::parse(&mut reader, &Config::fixture()).unwrap_or(Filter::FileName);
            assert_eq!(reader.position(), position);
        }
    }

    mod eval {
        use super::*;
        use crate::pattern::eval::{Context, ErrorKind};
        use crate::pattern::uuid::assert_uuid;
        use test_case::test_case;

        #[test_case("non-existent", Filter::CanonicalPath, ErrorKind::CanonicalizationFailed(AnyString::any()); "canonicalization failed")]
        fn err(input: &str, filter: Filter, kind: ErrorKind) {
            assert_eq!(filter.eval(input.into(), &Context::fixture()), Err(kind))
        }

        #[cfg_attr(unix, test_case("", Filter::WorkingDir, "/work"; "working dir"))]
        #[cfg_attr(unix, test_case("parent/file.ext", Filter::AbsolutePath, "/work/parent/file.ext"; "absolute path"))]
        #[cfg_attr(unix, test_case("root/parent/../new-parent/./dir/", Filter::NormalizedPath, "root/new-parent/dir"; "normalized path"))]
        #[cfg_attr(unix, test_case("/parent/file.ext", Filter::RelativePath, "../parent/file.ext"; "relative path"))]
        #[cfg_attr(unix, test_case("./Cargo.toml", Filter::CanonicalPath, "/work/Cargo.toml"; "canonical path"))]
        #[cfg_attr(windows, test_case("", Filter::WorkingDir, "C:\\work"; "working dir"))]
        #[cfg_attr(windows, test_case("parent/file.ext", Filter::AbsolutePath, "C:\\work\\parent\\file.ext"; "absolute path"))]
        #[cfg_attr(windows, test_case("/parent/file.ext", Filter::RelativePath, "..\\parent\\file.ext"; "relative path"))]
        #[cfg_attr(windows, test_case("root/parent/../new-parent/./dir/", Filter::NormalizedPath, "root\\new-parent[0-9]ir"; "normalized path"))]
        #[cfg_attr(windows, test_case("./Cargo.toml", Filter::CanonicalPath, "C:\\work\\Cargo.toml"; "canonical path"))]
        #[test_case("root/parent/file.ext", Filter::ParentDirectory, "root/parent"; "parent directory")]
        #[test_case("root/parent/file.ext", Filter::RemoveLastName, "root/parent"; "remove last name")]
        #[test_case("root/parent/file.ext", Filter::FileName, "file.ext"; "file name")]
        #[test_case("root/parent/file.ext", Filter::LastName, "file.ext"; "last name")]
        #[test_case("root/parent/file.ext", Filter::BaseName, "file"; "base name")]
        #[test_case("root/parent/file.ext", Filter::RemoveExtension, "root/parent/file"; "remove extension")]
        #[test_case("root/parent/file.ext", Filter::Extension, "ext"; "extension")]
        #[test_case("root/parent/file.ext", Filter::ExtensionWithDot, ".ext"; "extension with dot")]
        #[test_case("root/parent", Filter::EnsureTrailingDirSeparator, "root/parent/"; "ensure trailing separator")]
        #[test_case("root/parent/", Filter::RemoveTrailingDirSeparator, "root/parent"; "remove trailing separator")]
        #[test_case("abcde", Filter::Substring(index_range_at()), "b"; "substring at")]
        #[test_case("abcde", Filter::Substring(index_range_from()), "bcde"; "substring from")]
        #[test_case("abcde", Filter::Substring(index_range_between()), "bc"; "substring between")]
        #[test_case("abcde", Filter::Substring(index_range_length()), "bcd"; "substring length")]
        #[test_case("abcde", Filter::SubstringBackward(index_range_at()), "d"; "substring backward at")]
        #[test_case("abcde", Filter::SubstringBackward(index_range_from()), "abcd"; "substring backward from")]
        #[test_case("abcde", Filter::SubstringBackward(index_range_between()), "cd"; "substring backward between")]
        #[test_case("abcde", Filter::SubstringBackward(index_range_length()), "bcd"; "substring backward length")]
        #[test_case("a , b , c , d", Filter::GetColumn(column_string()), " b "; "column string separator")]
        #[test_case("a , b , c , d", Filter::GetColumn(column_regex()), "b"; "column regex separator")]
        #[test_case("a , b , c , d", Filter::GetColumnBackward(column_string()), " c "; "column backward string separator")]
        #[test_case("a , b , c , d", Filter::GetColumnBackward(column_regex()), "c"; "column backward regex separator")]
        #[test_case("abcd_abcd", Filter::ReplaceFirst(substitution_string_1()), "cd_abcd"; "remove first")]
        #[test_case("abcd_abcd", Filter::ReplaceFirst(substitution_string_2()), "xcd_abcd"; "replace first")]
        #[test_case("abcd_abcd", Filter::ReplaceAll(substitution_string_1()), "cd_cd"; "remove all")]
        #[test_case("abcd_abcd", Filter::ReplaceAll(substitution_string_2()), "xcd_xcd"; "replace all")]
        #[test_case("", Filter::ReplaceEmpty(substitution_empty()), "x"; "replace empty")]
        #[test_case("a123y", Filter::RegexMatch("[0-9]+".into()), "123"; "regex match")]
        #[test_case("12_34", Filter::RegexReplaceFirst(substitution_regex_1()), "_34"; "regex remove first")]
        #[test_case("12_34", Filter::RegexReplaceFirst(substitution_regex_2()), "x_34"; "regex replace first")]
        #[test_case("12_34", Filter::RegexReplaceAll(substitution_regex_1()), "_"; "regex remove all")]
        #[test_case("12_34", Filter::RegexReplaceAll(substitution_regex_2()), "x_x"; "regex replace all")]
        #[test_case("1", Filter::RegexSwitch(regex_switch()), "digit"; "regex switch case")]
        #[test_case("a", Filter::RegexSwitch(regex_switch()), "alpha"; "regex switch default")]
        #[test_case("", Filter::RegexCapture(1), "a"; "regex capture")]
        #[test_case(" abcd ", Filter::Trim, "abcd"; "trim")]
        #[test_case("ábčdÁBČD", Filter::ToLowercase, "ábčdábčd"; "to lowercase")]
        #[test_case("ábčdÁBČD", Filter::ToUppercase, "ÁBČDÁBČD"; "to uppercase")]
        #[test_case("ábčdÁBČD", Filter::ToAscii, "abcdABCD"; "to ascii")]
        #[test_case("ábčdÁBČD", Filter::RemoveNonAscii, "bdBD"; "remove non-ascii")]
        #[test_case("01", Filter::LeftPad(padding_fixed()), "ab01"; "left pad fixed")]
        #[test_case("01", Filter::LeftPad(padding_repeated()), "abca01"; "left pad repeated")]
        #[test_case("01", Filter::RightPad(padding_fixed()), "01cd"; "right pad fixed")]
        #[test_case("01", Filter::RightPad(padding_repeated()), "01cabc"; "right pad repeated")]
        #[test_case("", Filter::Repeat(repetition()), "abcabc"; "repetition ")]
        #[test_case("", Filter::LocalCounter, "1"; "local counter")]
        #[test_case("", Filter::GlobalCounter, "2"; "global counter")]
        #[test_case("", Filter::RandomNumber(number_range_zero()), "0"; "random number")]
        #[test_case("", Filter::RandomUuid, ""; "random uuid")]
        fn ok(input: &str, filter: Filter, output: &str) {
            match filter {
                Filter::CanonicalPath => {
                    let real_working_dir = std::env::current_dir().unwrap(); // Canonical path filter actually checks existence of the directory
                    let mut context = Context::fixture();
                    let output = output.replace(
                        context.working_dir.to_str().unwrap(),
                        real_working_dir.to_str().unwrap(),
                    );
                    context.working_dir = &real_working_dir;
                    assert_eq!(filter.eval(input.into(), &context), Ok(output))
                }
                Filter::RandomUuid => {
                    assert_uuid(&filter.eval(input.into(), &Context::fixture()).unwrap());
                }
                _ => {
                    assert_eq!(
                        filter.eval(input.into(), &Context::fixture()),
                        Ok(output.into())
                    )
                }
            }
        }
    }

    #[test_case(Filter::WorkingDir, "Working directory"; "working directory")]
    #[test_case(Filter::AbsolutePath, "Absolute path"; "absolute path")]
    #[test_case(Filter::RelativePath, "Relative path"; "relative path")]
    #[test_case(Filter::NormalizedPath, "Normalized path"; "normalized path")]
    #[test_case(Filter::CanonicalPath, "Canonical path"; "canonical path")]
    #[test_case(Filter::ParentDirectory, "Parent directory"; "parent directory")]
    #[test_case(Filter::RemoveLastName, "Remove last name"; "remove last name")]
    #[test_case(Filter::FileName, "File name"; "file name")]
    #[test_case(Filter::LastName, "Last name"; "last name")]
    #[test_case(Filter::BaseName, "Base name"; "base name")]
    #[test_case(Filter::RemoveExtension, "Remove extension"; "remove extension")]
    #[test_case(Filter::Extension, "Extension"; "extension")]
    #[test_case(Filter::ExtensionWithDot, "Extension with dot"; "extension with dot")]
    #[test_case(Filter::EnsureTrailingDirSeparator, "Ensure trailing directory separator"; "ensure trailing separator")]
    #[test_case(Filter::RemoveTrailingDirSeparator, "Remove trailing directory separator"; "remove trailing separator")]
    #[test_case(Filter::Substring(index_range_at()), "Substring from 2..2"; "substring at")]
    #[test_case(Filter::Substring(index_range_from()), "Substring from 2.."; "substring from")]
    #[test_case(Filter::Substring(index_range_between()), "Substring from 2..3"; "substring between")]
    #[test_case(Filter::Substring(index_range_length()), "Substring from 2..4"; "substring length")]
    #[test_case(Filter::SubstringBackward(index_range_at()), "Substring from 2..2 backward"; "substring backward at")]
    #[test_case(Filter::SubstringBackward(index_range_from()), "Substring from 2.. backward"; "substring backward from")]
    #[test_case(Filter::SubstringBackward(index_range_between()), "Substring from 2..3 backward"; "substring backward between")]
    #[test_case(Filter::SubstringBackward(index_range_length()), "Substring from 2..4 backward"; "substring backward length")]
    #[test_case(Filter::GetColumn(column_string()), "Get column #2 (',' separator)"; "column string separator")]
    #[test_case(Filter::GetColumn(column_regex()), "Get column #2 (regular expression '[, ]+' separator)"; "column regex separator")]
    #[test_case(Filter::GetColumnBackward(column_string()), "Get column #2 (',' separator) backward"; "column backward string separator")]
    #[test_case(Filter::GetColumnBackward(column_regex()), "Get column #2 (regular expression '[, ]+' separator) backward"; "column backward regex separator")]
    #[test_case(Filter::ReplaceFirst(substitution_string_1()), "Replace first 'ab' with ''"; "remove first")]
    #[test_case(Filter::ReplaceFirst(substitution_string_2()), "Replace first 'ab' with 'x'"; "replace first")]
    #[test_case(Filter::ReplaceAll(substitution_string_1()), "Replace all 'ab' with ''"; "remove all")]
    #[test_case(Filter::ReplaceAll(substitution_string_2()), "Replace all 'ab' with 'x'"; "replace all")]
    #[test_case(Filter::ReplaceEmpty(substitution_empty()), "Replace empty with 'x'"; "replace empty")]
    #[test_case(Filter::RegexMatch("[0-9]+".into()), "Match of regular expression '[0-9]+'"; "regex match")]
    #[test_case(Filter::RegexReplaceFirst(substitution_regex_1()), "Replace first match of regular expression '[0-9]+' with ''"; "regex remove first")]
    #[test_case(Filter::RegexReplaceFirst(substitution_regex_2()), "Replace first match of regular expression '[0-9]+' with 'x'"; "regex replace first")]
    #[test_case(Filter::RegexReplaceAll(substitution_regex_1()), "Replace all matches of regular expression '[0-9]+' with ''"; "regex remove all")]
    #[test_case(Filter::RegexReplaceAll(substitution_regex_2()), "Replace all matches of regular expression '[0-9]+' with 'x'"; "regex replace all")]
    #[test_case(
        Filter::RegexSwitch(regex_switch()),
        "Regular expression switch with variable output:\n\n    if input matches '[0-9]+'\n        output is 'digit'\n    else\n        output is 'alpha'";
        "regex switch "
    )]
    #[test_case(Filter::RegexCapture(1), "Capture group #1 of a global regular expression"; "regex capture")]
    #[test_case(Filter::Trim, "Trim"; "trim")]
    #[test_case(Filter::ToLowercase, "To lowercase"; "to lowercase")]
    #[test_case(Filter::ToUppercase, "To uppercase"; "to uppercase")]
    #[test_case(Filter::ToAscii, "To ASCII"; "to ascii")]
    #[test_case(Filter::RemoveNonAscii, "Remove non-ASCII"; "remove non-ascii")]
    #[test_case(Filter::LeftPad(padding_fixed()), "Left pad with 'abcd'"; "left pad fixed")]
    #[test_case(Filter::LeftPad(padding_repeated()), "Left pad with 2x 'abc'"; "left pad repeated")]
    #[test_case(Filter::RightPad(padding_fixed()), "Right pad with 'abcd'"; "right pad fixed")]
    #[test_case(Filter::RightPad(padding_repeated()), "Right pad with 2x 'abc'"; "right pad repeated")]
    #[test_case(Filter::Repeat(repetition()), "Repeat 2x 'abc'"; "repetition ")]
    #[test_case(Filter::LocalCounter, "Local counter"; "local counter")]
    #[test_case(Filter::GlobalCounter, "Global counter"; "global counter")]
    #[test_case(Filter::RandomNumber(number_range_full()), "Random number from [0, 2^64)"; "random number")]
    #[test_case(Filter::RandomNumber(number_range_from()), "Random number from [2, 2^64)"; "random number from")]
    #[test_case(Filter::RandomNumber(number_range_between()), "Random number from [2, 10]"; "random number between")]
    #[test_case(Filter::RandomUuid, "Random UUID"; "random uuid")]
    fn display(filter: Filter, result: &str) {
        assert_eq!(filter.to_string(), result);
    }

    fn index_range_at() -> IndexRange {
        Range::<Index>(1, Some(2))
    }

    fn index_range_from() -> IndexRange {
        Range::<Index>(1, None)
    }

    fn index_range_between() -> IndexRange {
        Range::<Index>(1, Some(3))
    }

    fn index_range_length() -> IndexRange {
        Range::<Index>(1, Some(4))
    }

    fn column_global() -> Column {
        Column {
            index: 1,
            separator: Separator::String(DEFAULT_SEPARATOR.into()),
        }
    }

    fn column_string() -> Column {
        Column {
            index: 1,
            separator: Separator::String(','.into()),
        }
    }

    fn column_regex() -> Column {
        Column {
            index: 1,
            separator: Separator::Regex("[, ]+".into()),
        }
    }

    fn substitution_empty() -> EmptySubstitution {
        Substitution {
            target: Empty,
            replacement: "x".into(),
        }
    }

    fn substitution_string_1() -> StringSubstitution {
        Substitution {
            target: "ab".into(),
            replacement: "".into(),
        }
    }

    fn substitution_string_2() -> StringSubstitution {
        Substitution {
            target: "ab".into(),
            replacement: "x".into(),
        }
    }

    fn substitution_regex_1() -> RegexSubstitution {
        Substitution {
            target: "[0-9]+".into(),
            replacement: "".into(),
        }
    }

    fn substitution_regex_2() -> RegexSubstitution {
        Substitution {
            target: "[0-9]+".into(),
            replacement: "x".into(),
        }
    }

    fn regex_switch() -> RegexSwitch {
        RegexSwitch {
            cases: vec![Case {
                matcher: "[0-9]+".into(),
                result: "digit".into(),
            }],
            default: "alpha".into(),
        }
    }

    fn padding_fixed() -> Padding {
        Padding::Fixed("abcd".into())
    }

    fn padding_repeated() -> Padding {
        Padding::Repeated(repetition())
    }

    fn repetition() -> Repetition {
        Repetition {
            count: 2,
            value: "abc".into(),
        }
    }

    fn number_range_full() -> NumberRange {
        Range::<Number>(0, None)
    }

    fn number_range_from() -> NumberRange {
        Range::<Number>(2, None)
    }

    fn number_range_between() -> NumberRange {
        Range::<Number>(2, Some(11))
    }

    fn number_range_zero() -> NumberRange {
        Range::<Number>(0, Some(0))
    }
}
