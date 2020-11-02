use indoc::indoc;
use std::io::{Result, Write};

const PATTERN_HELP: &str = indoc! {r#"
PATTERN SYNTAX
==============

Pattern is a string describing how to generate output from an input.

Use `--explain` flag to print detailed explanation what a certain pattern does.

    $ rew --explain 'file_{c|<000}.{e}'

By default, pattern characters are directly copied to output.

    INPUT    PATTERN    OUTPUT
    *        abc        abc

Characters between `{` and `}` form an expression which it is evaluated against input.

    INPUT       PATTERN    OUTPUT     EXPRESSION DESCRIPTION
    file.txt    {b}        file       Base name
    file.txt    new.{e}    new.txt    Extension

Expression `{v|f1|f2|...}` is made of a variable `v` and zero or more filters `f1`, `f2`, ..., separated by `|`.

    INPUT       PATTERN          OUTPUT      EXPRESSION DESCRIPTION
    img.JPEG    new.{e}          new.JPEG    Extension
    img.JPEG    new.{e|l}        new.jpeg    Extension + Lowercase
    img.JPEG    new.{e|l|r:e}    new.jpg     Extension + Lowercase + Remove `e`

Use `--help-vars` flag to print variable reference.
Use `--help-filters` flag to print filter reference.

Character `#` starts an escape sequence.

    SEQUENCE    DESCRIPTION
    #n          New line
    #r          Carriage return
    #t          Horizontal tab
    #0          Null
    #{          Escaped `{`
    #|          Escaped `|`
    #}          Escaped `{`
    ##          Escaped `#`

Use `--escape` option to set a different escape character.

    $ rew '{p|R:#t: }'              # Replace tabs by spaces in path
    $ rew '{p|R:\t: }' --escape='\' # Same thing, different escape character
"#};

const VARIABLES_HELP: &str = indoc! {"
VARIABLE REFERENCE
==================

    VARIABLE    DESCRIPTION
    p           Path (equal to input value).
    a           Absolute path.
    A           Canonical path.
    f           File name.
    b           Base name.
    e           Extension.
    E           Extension with dot.
    d           Parent path.
    D           Parent file name.
    c           Local counter.
    C           Global counter.
    u           Randomly generated UUID (v4).
    1, 2, ...   Regex capture group N.

EXAMPLES
========

Let us assume the following directory structure:

    /
    └── home
        ├── alice
        │   └── docs
        │       └── notes.txt
        |
        └── bob <-- current working directory

    INPUT                      PATTERN    OUTPUT
    ../alice/docs/notes.txt    {p}        ../alice/dir/notes.txt
    ../alice/docs/notes.txt    {a}        /home/bob/../alice/dir/notes.txt
    ../alice/docs/notes.txt    {A}        /home/alice/dir/notes.txt
    ../alice/docs/notes.txt    {f}        notes.txt
    ../alice/docs/notes.txt    {b}        notes
    ../alice/docs/notes.txt    {e}        txt
    ../alice/docs/notes.txt    {E}        .txt
    ../alice/docs/notes.txt    {d}        ../alice/docs
    ../alice/docs/notes.txt    {D}        docs

Global counter `C` is incremented for every input.

    INPUT    GLOBAL    LOCAL
    a/x      1         1
    a/y      2         2
    b/x      3         1
    b/y      4         2

Use `-e, --regex` option to match regular expression against filename.
Use `-E, --regex-full` option to match regular expression against whole path.
The matched capture groups can be referenced using `1`, `2`, ...

    $ rew -e '([0-9]+)' '{1}' # Print the first number in filename
    $ rew -E '([0-9]+)' '{1}' # Print the first number in whole path
"};

const FILTERS_HELP: &str = indoc! {"
FILTER REFERENCE
================

    FILTER    DESCRIPTION
    nA-B      Substring from index A to B.
              Indices start from 1 and are both inclusive.
    nA-       Substring from index A to end.
    n-B       Substring from start index B.
    N         Same as `n` but we are indexing from end to start.
    r:X:Y     Replace first occurrence of X by Y.
    r:X       Remove first occurrence of X.
              Any other character than `:` can be also used as a separator.
    R         Same as `r` but removes/replaces all occurrences.
    s         Same as `r` but X is an regular expression.
              Y can reference capture groups from X using $1, $2, ...
    S         Same as `s` but removes/replaces all occurrences.
    t         Trim whitespaces from bother sides.
    u         To uppercase.
    l         To lowercase.
    a         Convert non-ASCII characters ASCII.
    A         Remove non-ASCII characters.
    <M        Left pad with mask M.
    >M        Right pad with mask M.
    ?D        Replace empty input by D.

EXAMPLES
========

TODO
"};

pub fn write_pattern_help<O: Write>(output: &mut O) -> Result<()> {
    writeln!(output, "{}", PATTERN_HELP)
}

pub fn write_variables_help<O: Write>(output: &mut O) -> Result<()> {
    writeln!(output, "{}", VARIABLES_HELP)
}

pub fn write_filters_help<O: Write>(output: &mut O) -> Result<()> {
    writeln!(output, "{}", FILTERS_HELP)
}

#[cfg(tests)]
mod tests {
    use super::*;
    use common::testing::ColoredOuput;

    #[test]
    fn writes_pattern_help() {
        let mut ouput = ColoredOuput::new();
        write_pattern_help(&mut ouput).unwrap();
        assert_eq!(ouput.chunks().is_empty(), false);
    }

    #[test]
    fn writes_variables_help() {
        let mut ouput = ColoredOuput::new();
        write_variables_help(&mut ouput).unwrap();
        assert_eq!(ouput.chunks().is_empty(), false);
    }

    #[test]
    fn writes_filters_help() {
        let mut ouput = ColoredOuput::new();
        write_filters_help(&mut ouput).unwrap();
        assert_eq!(ouput.chunks().is_empty(), false);
    }
}
