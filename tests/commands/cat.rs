use crate::command_test;

command_test!("cat", {
    defalt_empty: [ cmd should "" => "" ],
    defalt_items: [ cmd should "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij" ],
    defalt_non_utf8: [ cmd should &[0x00, 0x9f, 0x92, 0x96] => vec![0x00, 0x9f, 0x92, 0x96] ],
    // seq 1 10000 | md5sum
    default_big: [ sh "seq 1 10000 | %cmd% | md5sum" should "" => "72d4ff27a28afbc066d5804999d5a504  -\n" ],
    blocks_empty: [ cmd "-b" should "" => "" ],
    blocks_items: [ cmd "-b" should "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij" ],
    blocks_buf_mode: [ cmd "-b" "--buf-mode=line" should "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij" ],
    blocks_buf_size: [ cmd "-b" "--buf-size=8" should "123456789" => "123456789" ],
    blocks_non_utf8: [ cmd "-b" should &[0x00, 0x9f, 0x92, 0x96] => vec![0x00, 0x9f, 0x92, 0x96] ],
    // seq 1 10000 | md5sum
    blocks_big: [ sh "seq 1 10000 | %cmd% -b | md5sum" should "" => "72d4ff27a28afbc066d5804999d5a504  -\n" ],
    lines_empty: [ cmd "-l" should "" => "" ],
    lines_items: [ cmd "-l" should "a\nbc\r\ndef\0ghij" => "a\nbc\ndef\0ghij\n" ],
    lines_buf_mode: [ cmd "-l" "--buf-mode=line" should "a\nbc\r\ndef\0ghij" => "a\nbc\ndef\0ghij\n" ],
    lines_buf_under: [ cmd "-l" "--buf-size=8" should "123456\n789" => "123456\n789\n" ],
    lines_buf_exact: [ cmd "-l" "--buf-size=8" should "1234567\n89" => "1234567\n89\n" ],
    lines_buf_over: [ cmd "-l" "--buf-size=8" should "12345678\n9" => err "error: cannot fetch line longer than '8' bytes\n" ],
    lines_cr_buf_under: [ cmd "-l" "--buf-size=8" should "12345\r\n6789" => "12345\n6789\n" ],
    lines_cr_buf_exact: [ cmd "-l" "--buf-size=8" should "123456\r\n789" => "123456\n789\n" ],
    lines_cr_buf_over: [ cmd "-l" "--buf-size=8" should "1234567\r\n89" => err "error: cannot fetch line longer than '8' bytes\n" ],
    lines_non_utf8: [ cmd "-l" should &[0x00, 0x9f, 0x92, 0x96] => vec![0x00, 0x9f, 0x92, 0x96, 0x0a] ],
    // seq 1 10000 | md5sum
    lines_big: [ sh "seq 1 10000 | %cmd% -l | md5sum" should "" => "72d4ff27a28afbc066d5804999d5a504  -\n" ],
    records_empty: [ cmd "-l0" should "" => "" ],
    records_items: [ cmd "-l0" should "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij\0" ],
    records_buf_mode: [ cmd "-l0" "--buf-mode=line" should "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij\0" ],
    records_buf_under: [ cmd "-l0" "--buf-size=8" should "123456\x00789" => "123456\x00789\0" ],
    records_buf_exact: [ cmd "-l0" "--buf-size=8" should "1234567\089" => "1234567\089\0" ],
    records_buf_over: [ cmd "-l0" "--buf-size=8" should "12345678\09" => err "error: cannot fetch line longer than '8' bytes\n" ],
    records_non_utf8: [ cmd "-l0" should &[0x00, 0x9f, 0x92, 0x96] => vec![0x00, 0x9f, 0x92, 0x96, 0x00] ],
    // seq 1 10000 | tr '\n' '\0' | md5sum
    records_big: [ sh "seq 1 10000 | tr '\\n' '\\0' | %cmd% -l0 | md5sum" should "" => "05cb1e39ef75550ae349fb6f08cd6000  -\n" ],
});
