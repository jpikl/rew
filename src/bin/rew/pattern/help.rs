use common::help::highlight_help;
use indoc::indoc;
use std::io::{Result, Write};
use termcolor::WriteColor;

const PATTERN_HELP: &str = indoc! {r#"
========================================
 Pattern syntax
========================================

Pattern is a string describing how to generate output from an input.

Use `--explain` flag to print detailed explanation what a certain pattern does.

    $> rew --explain 'file_{c|<3:0}.{e}'

By default, pattern characters are directly copied to output.

    INPUT    PATTERN    OUTPUT
    --------------------------
    *        `abc`        abc

Characters `{` and `}` form an expression which is evaluated and replaced in output.

Empty exrpession `{}` evaluates directly to input value.

    INPUT    PATTERN      OUTPUT
    ------------------------------------
    world    `{}`           world
    world    `Hello_{}_!`   Hello_world_!

Expression may contain one or more filters, delimited by `|`, which are consecutively applied on input value.

    INPUT       PATTERN          OUTPUT      DESCRIPTION
    ---------------------------------------------------------------------------
    old.JPEG    `new.{e}`          new.JPEG    Extension
    old.JPEG    `new.{e|l}`        new.jpeg    Extension + Lowercase
    old.JPEG    `new.{e|l|r:e}`    new.jpg     Extension + Lowercase + Remove 'e'

Use `--help-filters` flag to print filter reference.

Character `#` starts an escape sequence.

    SEQUENCE    DESCRIPTION
    ---------------------------
    `#n`          New line
    `#r`          Carriage return
    `#t`          Horizontal tab
    `#0`          Null
    `#{`          Escaped {
    `#|`          Escaped |
    `#}`          Escaped {
    `##`          Escaped #

Use `--escape` option to set a different escape character.

    $> rew '{R:#t: }'              # Replace tabs with spaces
    $> rew '{R:\t: }' --escape='\' # Same thing, different escape character
"#};

const FILTERS_HELP: &str = indoc! {"
========================================
 Path filters
========================================

    FILTER    DESCRIPTION
    ---------------------------------------------------
    `a`         Absolute path
    `p`         Normalized path
    `P`         Canonical path
    `d`         Parent directory
    `D`         Remove last name
    `f`         File name
    `b`         Base name
    `B`         Remove extension
    `e`         Extension
    `E`         Extension with dot
              Dot is not printed for missing extension.
    `z`         Ensure trailing separator
    `Z`         Remove trailing separator

Let us assume the following directory structure:

    /
    └── home
        ├── alice
        │   └── notes.txt
        |
        └── bob

For working directory `/home/bob` and input `../alice/notes.txt`, filters would evaluate to:

    FILTER    OUTPUT
    --------------------------------------
    `a`         /home/bob/../alice/notes.txt
    `p`         ../alice/notes.txt
    `P`         /home/alice/notes.txt
    `d`         ../alice
    `D`         ../alice
    `f`         notes.txt
    `b`         notes
    `B`         ../alice/notes
    `e`         txt
    `E`         .txt
    `z`         ../alice/notes.txt/
    `Z`         ../alice/notes.txt

Normalized path `p` is constructed using the following rules:

 - On Windows, all `/` separators are converted to `\\`.
 - Consecutive path separators are collapsed into one.
 - Non-root trailing path separator is removed.
 - Unnecessary current directory `.` components are removed.
 - Parent directory `..` components are resolved where possible.
 - Initial `..` components in an absolute path are dropped.
 - Initial `..` components in a relative path are kept.
 - Empty path is resolved to `.` (current directory).

    INPUT      OUTPUT        INPUT      OUTPUT
    -----------------        -----------------
    (empty)    .             /          /
    .          .             /.         /
    ..         ..            /..        /
    a/         a             /a/        /a
    a//        a             /a//       /a
    a/.        a             /a/.       /a
    a/..       .             /a/..      /
    ./a        a             /./a       /a
    ../a       ../a          /../a      /a
    a//b       a/b           /a//b      /a/b
    a/./b      a/b           /a/./b     /a/b
    a/../b     b             /a/../b    /b

Canonical path `P` works similarly to `p` but has some differences:

 - Evaluation will fail for a non-existent path.
 - Result will always be an absolute path.
 - If path is a symbolic link, it will be resolved.
 
Parent directory `d` might give a different result than `D` which removes last name of a path.
Similarly, file name `f` might not be the same as last name `F` which is a complement of `D`.
 
    INPUT      {d}      {D}        {f}        {F}     
    --------------------------------------------------
    /          /        /          (empty)    (empty) 
    /a         /        /          a          a       
    a/b        a        a          b          b       
    a          .        (empty)    a          a       
    ..         ../..    (empty)    (empty)    ..      
    .          ./..     (empty)    (empty)    .       
    (empty)    ..       (empty)    (empty)    (empty) 

========================================
 Substring filters
========================================

    FILTER    DESCRIPTION
    ------------------------------------------------------
    `nA-B`      Substring from index `A` to `B`.
              Indices start from 1 and are both inclusive.
    `nA-`       Substring from index `A` to end.
    `nA`        Character at index `A`.
              Equivalent to `nA-A`.
    `N`         Same as `n` but with backward indexing.

Examples:

    INPUT    FILTER    OUTPUT
    -------------------------
    abcde    `n2-3`      bc
    abcde    `N2-3`      cd
    abcde    `n2-`       bcde
    abcde    `N2-`       abcd
    abcde    `n2`        b
    abcde    `N2`        d

========================================
 Replace filters
========================================

    FILTER     DESCRIPTION
    ------------------------------------------------------------------------
    `r:X:Y`      Replace first occurrence of `X` with `Y`.
               Any other character than `:` can be also used as a delimiter.
    `r:X`        Remove first occurrence of `X`.
               Equivalent to `r:X:`.
    `R`          Same as `r` but replaces/removes all occurrences.
    `?D`         Replace empty value with `D`.

Examples:

    INPUT      FILTER       OUTPUT
    ------------------------------
    ab_ab      `r:ab:xy`      xy_ab
    ab_ab      `R:ab:xy`      xy_xy
    ab_ab      `r:ab`         _ab
    ab_ab      `R:ab`         _
    abc        `?def`         abc
    (empty)    `?def`         def

========================================
 Regex filters
========================================

    FILTER     DESCRIPTION
    --------------------------------------------------------------------------
    `=E`         Match of regular expression `E`.
    `s:X:Y`      Replace first match of regular expression `X` with `Y`.
               `Y` can reference capture groups from `X` using `$1`, `$2`, ...
               Any other character than `:` can be also used as a delimiter.
    `s:X`        Remove first match of regular expression `X`.
               Equivalent to `s:X:`.
    `S`          Same as `s` but replaces/removes all matches.
    `1`, `2`, ...  Capture group of an external regular expression.

Examples:

    INPUT    FILTER             OUTPUT
    ----------------------------------
    12_34    `=\\d+`               12
    12_34    `s:\\d+:x`            x_34
    12_34    `S:\\d+:x`            x_x
    12_34    `s:(\\d)(\\d):$2$1`    21_34
    12_34    `S:(\\d)(\\d):$2$1`    21_43

Use `-e, --regex` / `-E, --regex-filename` option to define an external regular expression.
Option `-e, --regex` matches regex against each input value.
Option `-E, --regex-filename` matches regex against 'filename component' of each input value.

    $> echo 'a/b.c' | rew -e '([a-z])' '{1}' # Will print 'a'
    $> echo 'a/b.c' | rew -E '([a-z])' '{1}' # Will print 'b'

========================================
 Format filters
========================================

    FILTER    DESCRIPTION
    -----------------------------------------------------------------------
    `t`         Trim white-spaces from both sides.
    `u`         Convert to uppercase.
    `l`         Convert to lowercase.
    `a`         Convert non-ASCII characters to ASCII.
    `A`         Remove non-ASCII characters.
    `<<M`       Left pad with mask `M`.
    `<N:M`      Left pad with `N` times repeated mask `M`.
              Any other non-digit than `:` can be also used as a delimiter.
    `>>M`       Right pad with mask `M`.
    `>N:M`      Right pad with `N` times repeated mask `M`.
              Any other non-digit than `:` can be also used as a delimiter.

Examples:

    INPUT       FILTER     OUTPUT
    ----------------------------------------------------
    ..a..b..    `t`          a..b  (dots are white-spaces)
    aBčĎ        `u`          ABČĎ
    aBčĎ        `l`          abčď
    aBčĎ        `a`          aBcD
    aBčĎ        `A`          aB
    abc         `<<123456`   123abc
    abc         `<3:XY`      XYXabc
    abc         `>>123456`   abc456
    abc         `>3:XY`      abcYXY

========================================
 Generators
========================================

    FILTER    DESCRIPTION
    -----------------------------------------------------------------------
    `*N:V`      Repeat `N` times `V`.
              Any other non-digit than `:` can be also used as a delimiter.
    `c`         Local counter
    `C`         Global counter
    `uA-B`      Random number from interval [`A`, `B`]
    `uA-`       Random number from interval [`A`, `2^64`)
    `u`         Random number from interval [`0`, `2^64`)
    `U`         Random UUID

Examples:

    FILTER    OUTPUT
    -------------------------------------------------------
    `*3:ab`     ababab
    `c`         (see below)
    `C`         (see below)
    `u0-99`     (random number between 0-99)
    `u`         5eefc76d-0ca1-4631-8fd0-62eeb401c432 (random)

Global counter `C` is a number incremented for every input value.
Local counter `c` is a number incremented per parent directory (assuming input value is a path).
Both counters start at 1 and are incremented by 1.

    INPUT    GLOBAL    LOCAL
    ------------------------
    a/x      1         1
    a/y      2         2
    b/x      3         1
    b/y      4         2

Use `-c, --local-counter` option to change local counter configuration.
Use `-C, --global-counter` option to change global counter configuration.

    $> rew -c0   '{c}' # Start from 0, increment by 1
    $> rew -c2:3 '{c}' # Start from 2, increment by 3
"};

pub fn write_pattern_help<O: Write + WriteColor>(output: &mut O) -> Result<()> {
    highlight_help(output, PATTERN_HELP)
}

pub fn write_filters_help<O: Write + WriteColor>(output: &mut O) -> Result<()> {
    highlight_help(output, FILTERS_HELP)
}

#[cfg(test)]
mod tests {
    use common::testing::ColoredOuput;
    use ntest::*;

    #[test]
    fn write_pattern_help() {
        use super::*;

        let mut ouput = ColoredOuput::new();
        write_pattern_help(&mut ouput).unwrap();
        assert_false!(ouput.chunks().is_empty());
    }

    #[test]
    fn write_filters_help() {
        use super::*;

        let mut ouput = ColoredOuput::new();
        write_filters_help(&mut ouput).unwrap();
        assert_false!(ouput.chunks().is_empty());
    }
}
