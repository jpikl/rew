#[path = "utils.rs"]
mod utils;

use rstest::rstest;
use utils::test_command;

#[rstest]
#[case(&[], "aB\nCD\nef\n", "AB\nCD\nEF\n")]
#[case(&[], "aB\nCD\něf\n", "AB\nCD\nĚF\n")]
fn upper(#[case] args: &[&str], #[case] input: &str, #[case] output: &str) {
    test_command("upper", args, input, output);
}
