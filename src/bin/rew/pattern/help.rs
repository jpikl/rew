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
    --------------------------
    *        abc        abc

Characters between `{` and `}` form an expression which it is evaluated against input.

    INPUT       PATTERN    OUTPUT     EXPRESSION DESCRIPTION
    --------------------------------------------------------
    file.txt    {b}        file       Base name
    file.txt    new.{e}    new.txt    Extension

Expression `{v|f1|f2|...}` is made of a variable `v` and zero or more filters `f1`, `f2`, ..., delimited by `|`.

    INPUT       PATTERN          OUTPUT      EXPRESSION DESCRIPTION
    ---------------------------------------------------------------------------
    img.JPEG    new.{e}          new.JPEG    Extension
    img.JPEG    new.{e|l}        new.jpeg    Extension + Lowercase
    img.JPEG    new.{e|l|r:e}    new.jpg     Extension + Lowercase + Remove `e`

Use `--help-vars` flag to print variable reference.
Use `--help-filters` flag to print filter reference.

Character `#` starts an escape sequence.

    SEQUENCE    DESCRIPTION
    ---------------------------
    #n          New line
    #r          Carriage return
    #t          Horizontal tab
    #0          Null
    #{          Escaped {
    #|          Escaped |
    #}          Escaped {
    ##          Escaped #

Use `--escape` option to set a different escape character.

    $ rew '{p|R:#t: }'              # Replace tabs by spaces in path
    $ rew '{p|R:\t: }' --escape='\' # Same thing, different escape character
"#};

const VARIABLES_HELP: &str = indoc! {"
VARIABLE REFERENCE
==================

    VARIABLE    DESCRIPTION
    -----------------------------------
    p           Input path
    a           Absolute path
    A           Canonical path
    f           File name
    b           Base name
    e           Extension
    E           Extension with dot
    d           Parent path
    D           Parent file name
    c           Local counter
    C           Global counter
    u           Randomly generated UUID
    1, 2, ...   Regex capture group N

Let us assume the following directory structure:

    /
    └── home
        ├── alice
        │   └── docs
        │       └── notes.txt
        |
        └── bob

For working directory `/home/bob` and input `../alice/docs/notes.txt`,
variables would be evaluated as:

    VARIABLE    OUTPUT
    ---------------------------------------------
    p           ../alice/docs/notes.txt
    a           /home/bob/../alice/docs/notes.txt
    A           /home/alice/docs/notes.txt
    f           notes.txt
    b           notes
    e           txt
    E           .txt
    d           ../alice/docs
    D           docs

Global counter `C` is incremented for every input.
Local counter `c` is incremented per directory.

    INPUT    GLOBAL    LOCAL
    ------------------------
    a/x      1         1
    a/y      2         2
    b/x      3         1
    b/y      4         2

Use `--gc-init, --gc-step` options to set initial/step value for global counter.
Use `--lc-init, --lc-step` options to set initial/step value for local counter.

    $ rew --gc-init=0 --gc-step=2 '{C}' # Start from 0, increment by 2
    $ rew --lc-init=1 --lc-step=3 '{c}' # Start from 1, increment by 3

Use `-e, --regex` option to match regular expression against filename.
Use `-E, --regex-full` option to match regular expression against path.
The matched capture groups can be referenced using 1, 2, ...

    $ echo 'a/b/c.d' | rew -e '([a-z])' '{1}' # Will print 'c'
    $ echo 'a/b/c.d' | rew -E '([a-z])' '{1}' # Will print 'a'
"};

const FILTERS_HELP: &str = indoc! {"
FILTER REFERENCE
================

Substring filters

    FILTER    DESCRIPTION
    ------------------------------------------------------------
    nA-B      Substring from index A to B.
              Indices start from 1 and are both inclusive.
    nA-       Substring from index A to end.
    n-B       Substring from start to index B.
    nA        Character at index A (equivalent to `nA-A`)
    N         Same as `n` but we are indexing from end to start.

Examples of substring filters

    INPUT    FILTER    OUTPUT
    -------------------------
    abcde    n2-3      bc
    abcde    N2-3      cd
    abcde    n2-       bcde
    abcde    N2-       abcd
    abcde    n-2       ab
    abcde    N-2       de
    abcde    n2        b
    abcde    N2        d

Replace filters

    FILTER   DESCRIPTION
    ----------------------------------------------------------------------
    r:X:Y    Replace first occurrence of X by Y.
             Any other character than `:` can be also used as a delimiter.
    r:X      Remove first occurrence of X.
    s        Same as `r` but X is an regular expression.
             Y can reference capture groups from X using $1, $2, ...
    R        Same as `r` but removes/replaces all occurrences.
    S        Same as `s` but removes/replaces all occurrences.
    ?D       Replace empty input with D.

Examples of replace filters

    INPUT      FILTER                   OUTPUT
    ------------------------------------------
    ab_ab      r:ab:xy                  xy_ab
    ab_ab      R:ab:xy                  xy_xy
    ab_ab      r:ab                     _ab
    ab_ab      R:ab                     _
    12_34      s:[0-9]:x                x2_34
    12_34      S:[0-9]:x                xx_xx
    12_34      s:([0-9])([0-9]):$2$1    21_34
    12_34      S:([0-9])([0-9]):$2$1    21_43
    abc        ?def                     abc
    (empty)    ?def                     def

Other filters

    FILTER    DESCRIPTION
    ------------------------------------------------
    t         Trim white-spaces from both sides.
    u         Convert to uppercase.
    l         Convert to lowercase.
    a         Convert non-ASCII characters to ASCII.
    A         Remove non-ASCII characters.
    <M        Left pad with mask M.
    >M        Right pad with mask M.

Examples of other filters

    INPUT       FILTER    OUTPUT
    ----------------------------
    ..a..b..    t         a..b    (dots are white-spaces)
    aBčĎ        u         ABČĎ
    aBčĎ        l         abčď
    aBčĎ        a         aBcD
    aBčĎ        A         aB
    abc         <12345    12abc
    abc         >12345    abc45
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
