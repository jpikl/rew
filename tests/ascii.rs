#[path = "utils.rs"]
mod utils;

use rstest::rstest;
use utils::test_command;

#[rstest]
#[case(&[], "áB\nčĎ\neF\n", "aB\ncD\neF\n")]
#[case(&["-d"], "áB\nčĎ\neF\n", "B\n\neF\n")]
fn ascii(#[case] args: &[&str], #[case] input: &str, #[case] output: &str) {
    test_command("ascii", args, input, output);
}
