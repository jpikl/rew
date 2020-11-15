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

    $> rew --explain 'file_{c|<00}.{e}'

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
    ----------------------------
    `a`         Absolute path
    `A`         Canonical path
    `p`         Parent path
    `f`         File name
    `b`         Base name
    `e`         Extension
    `E`         Extension with dot

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
    `A`         /home/alice/notes.txt
    `p`         ../alice
    `f`         notes.txt
    `b`         notes
    `e`         txt
    `E`         .txt

========================================
 Substring filters
========================================

    FILTER    DESCRIPTION
    ------------------------------------------------------
    `nA-B`      Substring from index `A` to `B`.
              Indices start from 1 and are both inclusive.
    `nA-`       Substring from index `A` to end.
    `n-B`       Substring from start to index `B`.
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
    abcde    `n-2`       ab
    abcde    `N-2`       de
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
    `mE`         Match of regular expression `E`.
    `s:X:Y`      Replace first match of regular expression `X` with `Y`.
               `Y` can reference capture groups from `X` using `$1`, `$2`, ...
               Any other character than `:` can be also used as a delimiter.
    `s:X`        Remove first match of regular expression `X`.
               Equivalent to `s:X:`.
    `S`          Same as `s` but replaces/removes all matches.
    

Examples:

    INPUT    FILTER                     OUTPUT
    ------------------------------------------
    12_34    `m[0-9]+`                    12
    12_34    `s:[0-9]+:x`                 x_34
    12_34    `S:[0-9]+:x`                 x_x
    12_34    `s:([0-9])([0-9]):$2$1`      21_34
    12_34    `S:([0-9])([0-9]):$2$1`      21_43

========================================
 Format filters
========================================

    FILTER    DESCRIPTION
    ------------------------------------------------
    `t`         Trim white-spaces from both sides.
    `u`         Convert to uppercase.
    `l`         Convert to lowercase.
    `a`         Convert non-ASCII characters to ASCII.
    `A`         Remove non-ASCII characters.
    `<M`        Left pad with mask `M`.
    `>M`        Right pad with mask `M`.
    
Examples:
    
    INPUT       FILTER      OUTPUT
    ------------------------------
    ..a..b..    `t`           a..b    (dots are white-spaces)
    aBčĎ        `u`           ABČĎ
    aBčĎ        `l`           abčď
    aBčĎ        `a`           aBcD
    aBčĎ        `A`           aB
    abc         `<12345`      12abc
    abc         `>12345`      abc45

========================================
 Generators
========================================

    FILTER    DESCRIPTION
    ---------------------------------
    `*N:V`      Repeat `N` times `V`.
    `c`         Local counter
    `C`         Global counter
    `u`         Randomly generated UUID 
    
Examples:
    
    FILTER    OUTPUT
    -------------------------------------------------------
    `*3:ab`     ababab
    `c`         (see below)
    `C`         (see below)
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
    use super::*;
    use common::testing::ColoredOuput;

    #[test]
    fn writes_pattern_help() {
        let mut ouput = ColoredOuput::new();
        write_pattern_help(&mut ouput).unwrap();
        assert_eq!(ouput.chunks().is_empty(), false);
    }

    #[test]
    fn writes_filters_help() {
        let mut ouput = ColoredOuput::new();
        write_filters_help(&mut ouput).unwrap();
        assert_eq!(ouput.chunks().is_empty(), false);
    }
}
