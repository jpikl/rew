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

Empty expression `{}` evaluates directly to input value.

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
    -------------------------------------
    `#/`          System directory separator
                `\` on Windows
                `/` everywhere else
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
    -----------------------------------
    `w`         Working directory
    `a`         Absolute path
    `A`         Relative path
    `p`         Normalized path
    `P`         Canonical path
    `d`         Parent directory
    `D`         Remove last name
    `f`         File name
    `b`         Base name
    `B`         Remove extension
    `e`         Extension
    `E`         Extension with dot
    `z`         Ensure trailing separator
    `Z`         Remove trailing separator

Path filters assume that their input value is a FS path.
To get a specific portion of a path, use one of `dD`, `fF`, `bB`, `eE` filters.

    PATTERN       OUTPUT
    -----------------------------------
    `{}`            /home/alice/notes.txt
    `{d}`, `{D}`      /home/alice
    `{f}`, `{F}`      notes.txt
    `{b}`           notes
    `{B}`           /home/alice/notes
    `{e}`           txt
    `{E}`           .txt

Parent directory `d` might give a different result than `D` which removes last name of a path.
Similarly, file name `f` might not be the same as last name `F` which is a complement of `D`.

    INPUT      {d}      {D}        {f}        {F}
    --------------------------------------------------
    /          /        /          (empty)    (empty)
    /a         /        /          a          a
    a/b        a        a          b          b
    a          .        (empty)    a          a
    .          ./..     (empty)    (empty)    .
    ..         ../..    (empty)    (empty)    ..
    (empty)    ..       (empty)    (empty)    (empty)

Extension with dot `E` can be useful when dealing with files with no extension.

    INPUT      new.{e}    new{E}
    -----------------------------
    old.txt    new.txt    new.txt
    old        new.       new

Absolute path `a` and relative path `A` are both resolved against working directory `w`.

    {w}            INPUT        {a}          {A}
    -----------------------------------------------
    /home/alice    /home/bob    /home/bob    ../bob
    /home/alice    ../bob       /home/bob    ../bob

By default, working directory `w` is set to your current working directory.
You can change that using the `-w, --working-directory` option.
`w` filter will always output an absolute path, even if you set a relative one using the `-w` option.

    $> rew -w '/home/alice' '{w}' # Absolute path
    $> rew -w '../alice'    '{w}' # Relative to your current working directory

Normalized path `p` is constructed using the following rules:

 - On Windows, all `/` separators are converted to `\\`.
 - Consecutive directory separators are collapsed into one.
 - Non-root trailing directory separator is removed.
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

Trailing separator filters `z` and `Z` can be useful when dealing with root and unnormalized paths.

    INPUT    {}b    {}/b    {z}b    {Z}/b
    -------------------------------------
    /        /b     //b     /b      /b
    a        ab     a/b     a/b     a/b
    a/       a/b    a//b    a/b     a/b

========================================
 Substring filters
========================================

    FILTER    DESCRIPTION
    ------------------------------------------------------
    `nA-B`      Substring from index `A` to `B`.
              Indices start from 1 and are both inclusive.
    `nA+B`      Substring from index `A` with length `B`.
    `nA-`       Substring from index `A` to end.
    `nA`        Character at index `A`.
              Equivalent to `nA-A`.
    `N`         Same as `n` but with backward indexing.

Examples:

    INPUT    PATTERN    OUTPUT
    --------------------------
    abcde    `{n2-3}`     bc
    abcde    `{N2-3}`     cd
    abcde    `{n2-}`      bcde
    abcde    `{N2-}`      abcd
    abcde    `{n2}`       b
    abcde    `{N2}`       d

========================================
 Replace filters
========================================

    FILTER     DESCRIPTION
    ----------------------------------------------------------------------
    `r:X:Y`      Replace first occurrence of `X` with `Y`.
               Any other character than `:` can be also used as a delimiter.
    `r:X`        Remove first occurrence of `X`.
               Equivalent to `r:X:`.
    `R`          Same as `r` but replaces/removes all occurrences.
    `?D`         Replace empty value with `D`.

Examples:

    INPUT      PATTERN      OUTPUT
    ------------------------------
    ab_ab      `{r:ab:xy}`    xy_ab
    ab_ab      `{R:ab:xy}`    xy_xy
    ab_ab      `{r:ab}`       _ab
    ab_ab      `{R:ab}`       _
    abc        `{?def}`       abc
    (empty)    `{?def}`       def

========================================
 Regex filters
========================================

    FILTER     DESCRIPTION
    ----------------------------------------------------------------------
    `=E`         Match of a regular expression `E`.
    `s:X:Y`      Replace first match of a regular expression `X` with `Y`.
               `Y` can reference capture groups from `X` using `$1`, `$2`, ...
               Any other character than `:` can be also used as a delimiter.
    `s:X`        Remove first match of a regular expression `X`.
               Equivalent to `s:X:`.
    `S`          Same as `s` but replaces/removes all matches.
    `1`, `2`, ...  Capture group of an external regular expression.

Examples:

    INPUT    PATTERN             OUTPUT
    -------------------------------------
    12_34    `{=\\d+}`                12
    12_34    `{s:\\d+:x}`             x_34
    12_34    `{S:\\d+:x}`             x_x
    12_34    `{s:(\\d)(\\d):$2$1}`    21_34
    12_34    `{S:(\\d)(\\d):$2$1}`    21_43

Use `-e, --regex` or `-E, --regex-filename` option to define an external regular expression.
Option `-e, --regex` matches regex against each input value.
Option `-E, --regex-filename` matches regex against 'filename component' of each input value.

    $> echo 'a/b.c' | rew -e '([a-z])' '{1}' # Will print 'a'
    $> echo 'a/b.c' | rew -E '([a-z])' '{1}' # Will print 'b'

========================================
 Format filters
========================================

    FILTER    DESCRIPTION
    ---------------------------------------------------------------------
    `t`         Trim white-spaces from both sides.
    `l`         Convert to lowercase.
    `L`         Convert to uppercase.
    `a`         Convert non-ASCII characters to ASCII.
    `A`         Remove non-ASCII characters.
    `<<M`       Left pad with mask `M`.
    `<N:M`      Left pad with `N` times repeated mask `M`.
              Any other non-digit than `:` can be also used as a delimiter.
    `>>M`       Right pad with mask `M`.
    `>N:M`      Right pad with `N` times repeated mask `M`.
              Any other non-digit than `:` can be also used as a delimiter.

Examples:

    INPUT       PATTERN       OUTPUT
    --------------------------------
    ..a..b..    `{t}`           a..b  (dots are white-spaces)
    aBčĎ        `{l}`           abčď
    aBčĎ        `{L}`           ABČĎ
    aBčĎ        `{a}`           aBcD
    aBčĎ        `{A}`           aB
    abc         `{<<123456}`    123abc
    abc         `{<3:XY}`       XYXabc
    abc         `{>>123456}`    abc456
    abc         `{>3:XY}`       abcYXY

========================================
 Generators
========================================

Unlike other filters, generator output is not produced from its input.
However, it is still possible (although meaningless) to pipe input into a generator.

    FILTER    DESCRIPTION
    ---------------------------------------------------------------------
    `*N:V`      Repeat `N` times `V`.
              Any other non-digit than `:` can be also used as a delimiter.
    `c`         Local counter
    `C`         Global counter
    `uA-B`      Random number from interval [`A`, `B`]
    `uA-`       Random number from interval [`A`, `2^64`)
    `u`         Random number from interval [`0`, `2^64`)
    `U`         Random UUID

Examples:

    PATTERN    OUTPUT
    --------------------------------------------------------
    `{*3:ab}`    ababab
    `{c}`        (see below)
    `{C}`        (see below)
    `{u0-99}`    (random number between 0-99)
    `{U}`        5eefc76d-0ca1-4631-8fd0-62eeb401c432 (random)

Global counter `C` is a number incremented for every input value.
Local counter `c` is a number incremented per parent directory (assuming input value is a FS path).
Both counters start at 1 and are incremented by 1.

    INPUT    GLOBAL    LOCAL
    ------------------------
    A/1      1         1
    A/2      2         2
    B/1      3         1
    B/2      4         2

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
