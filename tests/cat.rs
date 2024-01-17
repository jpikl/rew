#[path = "utils.rs"]
mod utils;

use rstest::rstest;
use utils::test_command;
use utils::test_command_failure;

#[test]
fn cat_max_line() {
    // Fits exactly the internal buffer including line terminator
    test_command("cat", &["-l", "--max-line=8"], "0123456\n", "0123456\n");
    // Line terminator does not fit into internal buffer
    test_command_failure("cat", &["-l", "--max-line=8"], "01234567");
}

#[rstest]
#[case(&[], "", "")]
#[case(&[], "a\nbc", "a\nbc")]
#[case(&["--buff=line"], "a", "a")]
#[case(&["--buff=line"], "a\nbc", "a\nbc")]
#[case(&[], &[0x00, 0x9f, 0x92, 0x96], &[0x00, 0x9f, 0x92, 0x96])]
#[case(&["-l"], "", "")]
#[case(&["-l"], "a\nbc", "a\nbc\n")]
#[case(&["-l"], "a\r\nbc", "a\nbc\n")]
#[case(&["-l"], "a\r\nbc", "a\nbc\n")]
#[case(&["-l", "--buff=line"], "a\r\nbc", "a\nbc\n")]
#[case(&["-l0"], "a\0bc", "a\0bc\0")]
#[case(&["-l0", "--buff=line"], "a\0bc", "a\0bc\0")]
#[case(&["-l"], &[0x00, 0x9f, 0x92, 0x96], &[0x00, 0x9f, 0x92, 0x96, 0x0a])]
fn cat(#[case] args: &[&str], #[case] input: impl AsRef<[u8]>, #[case] output: impl AsRef<[u8]>) {
    test_command("cat", args, input, output);
}
