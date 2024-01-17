#[path = "utils.rs"]
mod utils;

use rstest::rstest;
use utils::test_command;

#[rstest]
#[case(&[], "a\nbc\n", "a\n")]
#[case(&["0"], "a\nbc\n", "")]
#[case(&["1"], "a\nbc\n", "a\n")]
#[case(&["2"], "a\nbc\n", "a\nbc\n")]
#[case(&["3"], "a\nbc\n", "a\nbc\n")]
fn first(#[case] args: &[&str], #[case] input: &str, #[case] output: &str) {
    test_command("first", args, input, output);
}
