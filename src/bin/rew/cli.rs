use clap::{AppSettings, Clap};
use common::color::{parse_color, COLOR_VALUES};
use common::run::Options;
use regex::Regex;
use std::path::PathBuf;
use termcolor::ColorChoice;

#[derive(Debug, Clap)]
#[clap(
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    verbatim_doc_comment
)]
/// Rewrite FS paths using a pattern.
///
/// Pattern is a string describing how to generate output from an input.
///
/// By default, characters from pattern are directly copied to output.
///
///     INPUT    PATTERN    OUTPUT
///     *        abc        abc
///
/// Characters between '{' and '}' form an expresion which it is evaluated against input.
///
///     INPUT       PATTERN    OUTPUT
///     file.txt    {b}        file       # Basename
///     file.txt    new.{e}    new.txt    # Extension
///
/// Expression '{v|f1|f2|...}' is made of a variable 'v' and zero or more filters f1, f2, ..., separated by '|'.
///
///     INPUT       PATTERN          OUTPUT
///     img.JPEG    new.{b}          new.JPEG    # Basename
///     img.JPEG    new.{b|l}        new.jpeg    # Basename + Lowercase
///     img.JPEG    new.{b|l|r:e}    new.jpg     # Basename + Lowercase + Remove 'e'
///
/// Character '#' starts an escape sequence.
///
///     SEQUENCE    DESCRIPTION
///     #n          New line
///     #r          Carriage return
///     #t          Horizontal tab
///     #0          Null
///     #{          Escaped '{'
///     #|          Escaped '|'
///     #}          Escaped '{'
///     ##          Escaped '#'
///
/// Variable reference:
///
///     VARIABLE    DESCRIPTION
///     p           Path (equal to input value).
///     a           Absolute path.
///     A           Canonical path.
///     f           File name.
///     b           Base name.
///     e           Extension.
///     E           Extension with dot.
///     d           Parent path.
///     D           Parent file name.
///     c           Local counter.
///     C           Global counter.
///     1, 2, ...   Reference to regex capture group N.
///     u           Randomly generated UUID (v4).
///
/// Variable examples:
///
///     Assumptions:  
///     - There is as file '/home/user/dir/file.txt'.
///     - Working directory is '/home/user'.
///
///     INPUT                   PATTERN    OUTPUT
///     ../user/dir/file.txt    {p}        ../user/dir/file.txt
///     ../user/dir/file.txt    {a}        /home/user/../user/dir/file.txt
///     ../user/dir/file.txt    {A}        /home/user/dir/file.txt
///     ../user/dir/file.txt    {f}        file.txt
///     ../user/dir/file.txt    {b}        file
///     ../user/dir/file.txt    {e}        txt
///     ../user/dir/file.txt    {E}        .txt
///     ../user/dir/file.txt    {d}        ../user/dir
///     ../user/dir/file.txt    {D}        dir
///
///     Global counter 'C' is incremented for every input.
///
///     INPUT    PATTERN    OUTPUT
///     a/x      {C}        1
///     b/x                 2
///     b/y                 3
///     a/y                 4
///
///     Local counter 'c' is incremented per directory.
///
///     INPUT    PATTERN    OUTPUT
///     a/x      {c}        1
///     b/x                 1
///     b/y                 2
///     a/y                 2
///    
///     Option `-e, --regex` matches a regular expression against '{f}'.
///     Option `-E, --regex-full` matches a regular expression against '{p}'.
///     Their capture groups can be referenced using '1', '2', ....
///
///     INPUT      OPTION             PATTERN    OUTPUT
///     a/b/c.d    -e '^(.).*(.)$'    {1}_{2}    c_d
///     a/b/c.d    -E '^(.).*(.)$'    {1}_{2}    a_b
///
/// Filter reference:
///
///     FILTER    DESCRIPTION
///     nA-B      Substring from index A to B.
///               Indices starts from 1 and are both inclusive.
///     nA-       Substring from index A to end.
///     n-B       Substring from start index B.     
///     N         Same as 'n' filter but we are indexing from end to start.
///     r:X       Remove first occurrence of X.
///     r:X:Y     Replace first occurrence of X by Y.
///               Any other character than ':' can be also used as a separator.
///     R         Same as 'r' filter but removes/replaces all occurrences.
///     s         Same as 'r' but X is an regular expression.
///               Y can reference capture groups from X using $1, $2, ...
///     S         Same as 'R' but X is an regular expression.
///     t         Trim whitespaces from bother sides.
///     u         To uppercase.
///     l         To lowercase.
///     a         Convert non-ASCII characters to ASCII.
///     A         Remove non-ASCII characters.
///     <M        Left pad with mask M.
///     >M        Right pad with mask M.
///     ?D        Replace empty input by D.
///
/// Accompanying utilities `mvb` and `cpb` can be used to move/copy files based on `rew` output:
///
///   $ find -name '*.txt' | rew -b '{p}.bak' | cpb
pub struct Cli {
    /// Output pattern
    pub pattern: String,

    /// Paths to rewrite (read from stdin by default)
    #[clap(value_name = "path")]
    pub paths: Vec<PathBuf>,

    /// Read paths delimited by NUL, not newline
    #[clap(short = 'z', long, conflicts_with = "read-raw")]
    pub read_nul: bool,

    /// Read the whole input as a single path
    #[clap(short = 'r', long, conflicts_with = "read-nul")]
    pub read_raw: bool,

    /// Print results delimited by NUL, not newline
    #[clap(short = 'Z', long, conflicts_with = "print-raw")]
    pub print_nul: bool,

    /// Print results without any delimiter
    #[clap(short = 'R', long, conflicts_with = "print-nul")]
    pub print_raw: bool,

    /// Print machine-readable transformations as a results
    #[clap(short = 'b', long, conflicts_with = "pretty")]
    pub bulk: bool,

    /// Print human-readable transformations as a results
    #[clap(short = 'p', long, conflicts_with = "bulk")]
    pub pretty: bool,

    /// Continue after a path processing error, fail at end
    #[clap(short = 'c', long)]
    pub fail_at_end: bool,

    /// Print explanation of a given pattern
    #[clap(long)]
    pub explain: bool,

    /// Regular expression matched against file name
    #[clap(short = 'e', long)]
    pub regex: Option<Regex>,

    /// Regular expression matched against path
    #[clap(short = 'E', long, value_name = "regex")]
    pub regex_full: Option<Regex>,

    /// Global counter initial value
    #[clap(long, value_name = "number")]
    pub gc_init: Option<u32>,

    /// Global counter step
    #[clap(long, value_name = "number")]
    pub gc_step: Option<u32>,

    /// Local counter initial value
    #[clap(long, value_name = "number")]
    pub lc_init: Option<u32>,

    /// Local counter step
    #[clap(long, value_name = "number")]
    pub lc_step: Option<u32>,

    /// Custom escape character to use in pattern
    #[clap(long, value_name = "char")]
    pub escape: Option<char>,

    /// When to use colors
    #[clap(
        long,
        value_name = "when",
        possible_values = COLOR_VALUES,
        parse(try_from_str = parse_color),
    )]
    pub color: Option<ColorChoice>,
}

impl Options for Cli {
    fn color(&self) -> Option<ColorChoice> {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        assert!(Cli::try_parse_from(&["rew", "pattern"]).is_ok());
    }

    #[test]
    fn color() {
        let cli = Cli::try_parse_from(&["rew", "pattern", "--color=always"]).unwrap();
        assert_eq!(Options::color(&cli), Some(ColorChoice::Always));
    }
}
