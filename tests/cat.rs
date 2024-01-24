#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn cat() {
    let tc = Tc::cmd("cat");

    let empty = tc.clone().stdin("");
    empty.clone().ok("");
    empty.clone().arg("-0").ok("");
    empty.clone().arg("-l").ok("");
    empty.clone().arg("-l0").ok("");
    empty.clone().arg("--buff=line").ok("");
    empty.clone().arg("--buff=line").arg("-0").ok("");
    empty.clone().arg("--buff=line").arg("-l").ok("");
    empty.clone().arg("--buff=line").arg("-l0").ok("");

    let lines = tc.clone().stdin("a\nbc\r\ndef\0ghij");
    lines.clone().ok("a\nbc\r\ndef\0ghij");
    lines.clone().arg("-0").ok("a\nbc\r\ndef\0ghij");
    lines.clone().arg("-l").ok("a\nbc\ndef\0ghij\n");
    lines.clone().arg("-l0").ok("a\nbc\r\ndef\0ghij\0");
    lines.clone().arg("--buff=line").ok("a\nbc\r\ndef\0ghij");
    lines.clone().arg("--buff=line").arg("-0").ok("a\nbc\r\ndef\0ghij");
    lines.clone().arg("--buff=line").arg("-l").ok("a\nbc\ndef\0ghij\n");
    lines.clone().arg("--buff=line").arg("-l0").ok("a\nbc\r\ndef\0ghij\0");

    let non_utf8 = tc.clone().stdin([0x00, 0x9f, 0x92, 0x96]);
    non_utf8.clone().ok([0x00, 0x9f, 0x92, 0x96]);
    non_utf8.clone().arg("-0").ok([0x00, 0x9f, 0x92, 0x96]);
    non_utf8.clone().arg("-l").ok([0x00, 0x9f, 0x92, 0x96, 0x0a]);
    non_utf8.clone().arg("-l0").ok([0x00, 0x9f, 0x92, 0x96, 0x00]);
    non_utf8.clone().arg("--buff=line").ok([0x00, 0x9f, 0x92, 0x96]);
    non_utf8.clone().arg("--buff=line").arg("-0").ok([0x00, 0x9f, 0x92, 0x96]);
    non_utf8.clone().arg("--buff=line").arg("-l").ok([0x00, 0x9f, 0x92, 0x96, 0x0a]);
    non_utf8.clone().arg("--buff=line").arg("-l0").ok([0x00, 0x9f, 0x92, 0x96, 0x00]);

    let max_line = tc.clone().arg("--max-line=8").arg("-l");
    max_line.clone().stdin("0123456\n").ok("0123456\n");
    max_line
        .clone()
        .stdin("01234567")
        .err("error: cannot process input line bigger than '8' bytes\n");

    // Same hash as `seq 1 10000 | md5sum`
    Tc::shell("seq 1 10000 | %bin% cat | md5sum").ok("72d4ff27a28afbc066d5804999d5a504  -\n");
    Tc::shell("seq 1 10000 | %bin% cat -0 | md5sum").ok("72d4ff27a28afbc066d5804999d5a504  -\n");
    Tc::shell("seq 1 10000 | %bin% cat -l | md5sum").ok("72d4ff27a28afbc066d5804999d5a504  -\n");

    Tc::shell("seq 1 10000 | %bin% cat -l0")
        .err("error: cannot process input line bigger than '32768' bytes\n");

    // Same hash as `{ seq 1 10000; printf '\0'; } | md5sum`
    Tc::shell("seq 1 10000 | %bin% cat -l0 --max-line=65536 | md5sum")
        .ok("b57df9dc6e3f5501464d52e2f67cce33  -\n"); // Adds NUL at the end
}
