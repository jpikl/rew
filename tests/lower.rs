#[path = "utils.rs"]
mod utils;

use rstest::rstest;
use utils::test_command;

#[rstest]
#[case(&[], "aB\nCD\nef\n", "ab\ncd\nef\n")]
#[case(&[], "aB\nČD\nef\n", "ab\nčd\nef\n")]
fn lower(#[case] args: &[&str], #[case] input: &str, #[case] output: &str) {
    test_command("lower", args, input, output);
}
