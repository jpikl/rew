#[path = "utils.rs"]
mod utils;

use rstest::rstest;
use utils::test_command;

#[rstest]
#[case(&[], "\n  \n a \n  b  c  \n", "\n\na\nb  c\n")]
#[case(&["-s"], "\n  \n a \n  b  c  \n", "\n\na \nb  c  \n")]
#[case(&["-e"], "\n  \n a \n  b  c  \n", "\n\n a\n  b  c\n")]
#[case(&["-se"], "\n  \n a \n  b  c  \n", "\n\na\nb  c\n")]
fn trim(#[case] args: &[&str], #[case] input: &str, #[case] output: &str) {
    test_command("trim", args, input, output);
}
