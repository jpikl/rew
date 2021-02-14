use indoc::indoc;

pub const PATTERN: &str = indoc! {r#"
# Syntax

  `abc`         Constant  
  `{}`          Empty expression     
  `{x}`         Expression with a filter    
  `{x|y|z}`     Expression with multiple filters  
  `a{}b{x|y}c`  Example of a more complex pattern

# Rules

  Constants are directly copied to output.
  Expression is replaced by input value
  Filters are consecutively applied on input value.
"#};

pub const FILTERS: &str = indoc! {r#"
# Paths

  `d`  Parent directory             `D`  Remove last name
  `f`  File name                    `F`  Last name
  `b`  Base name                    `B`  Remove extension
  `e`  Extension                    `E`  Extension with dot

  `w`  Working directory  
  `a`  Absolute path                `A`  Relative path
  `p`  Normalized path              `P`  Canonical path

  `z`  Ensure trailing separator    `Z`  Remove trailing separator


# Substrings

  `nA-B`  From `A` to `B`           (`n` = normal indexing) 
  `nA+L`  From `A` with length `L`  (`N` = backward indexing) 
  `nA-`   From `A` to end         (`A`, `B` = inclusive 1-based indices)
  `nA`    Character at `A`


# Replacement

  `r:X:Y`  Replace `X` with `Y`      (`r` = first occurence)
  `r:X`    Remove `X`              (`R` = all occurences)
  `?D`     Replace empty with `D`  (`:` = any separator)


# Regular expressions

  `=E`     Match of regex `E`           (`:` = any separator)
  `s:X:Y`  Replace match of `X` with `Y`  (`s` = first occurence)  
  `s:X`    Remove match of `X`          (`S` = all occurences)

   `@:X1:Y1:...:Xn:Yn:D`  Output `Yi` for first match of `Xi`, `D` for no match

   `0`, `1`, `2`, ...     Capture group of an external regex
   `$0`, `$1`, `$2`, ...  Capture group inside `s/S/@`

# Formatting

  `t`     Trim
  `v`     To lowercase    `^`  To uppercase
  `i`     To ASCII        `I`  Remove non-ASCII chars

  `<<M`   Left pad with `M`            (`>>` or `>` to right pad)
  `<N:M`  Left pad `N` times with `M`    (`:` = any separator)


# Generators

  `*N:V`  Repeat `N` times `V`  (`:` = any separator)

  `c`     Local counter           `C`  Global counter
  `u`     Random 64-bit number    `U`  Random UUID

  `uA-B`  `u` where `A <= u <= B`
  `uA-`   `u` where `A <= u`
"#};

#[cfg(test)]
mod tests {
    use super::*;
    use common::help::highlight;
    use common::testing::ColoredOuput;
    use ntest::*;

    #[test]
    fn pattern() {
        test_highlight(PATTERN);
    }

    #[test]
    fn filters() {
        test_highlight(FILTERS);
    }

    fn test_highlight(text: &str) {
        let mut output = ColoredOuput::new();
        highlight(&mut output, text).unwrap();
        assert_false!(output.chunks().is_empty());
    }
}
