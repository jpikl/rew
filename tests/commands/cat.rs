use crate::command_test;

command_test!("cat", {
    defalt_empty: [ cmd assert "" => "" ],
    defalt_items: [ cmd assert "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij" ],
    defalt_non_utf8: [ cmd assert &[0x00, 0x9f, 0x92, 0x96] => vec![0x00, 0x9f, 0x92, 0x96] ],
    // seq 1 10000 | md5sum -t
    default_big: [ sh "seq 1 10000 | %cmd% | md5sum -t" assert "" => "72d4ff27a28afbc066d5804999d5a504  -\n" ],
    bytes_empty: [ cmd "-b" assert "" => "" ],
    bytes_items: [ cmd "-b" assert "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij" ],
    bytes_buf_mode: [ cmd "-b" "--buf-mode=line" assert "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij" ],
    bytes_buf_size: [ cmd "-b" "--buf-size=8" assert "123456789" => "123456789" ],
    bytes_non_utf8: [ cmd "-b" assert &[0x00, 0x9f, 0x92, 0x96] => vec![0x00, 0x9f, 0x92, 0x96] ],
    // seq 1 10000 | md5sum -t
    bytes_big: [ sh "seq 1 10000 | %cmd% -b | md5sum -t" assert "" => "72d4ff27a28afbc066d5804999d5a504  -\n" ],
    chars_empty: [ cmd "-c" assert "" => "" ],
    chars_items: [ cmd "-c" assert "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij" ],
    chars_buf_mode: [ cmd "-c" "--buf-mode=line" assert "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij" ],
    chars_buf_size: [ cmd "-c" "--buf-size=8" assert "123456789" => "123456789" ],
    chars_non_utf8: [ cmd "-c" assert &[0x00, 0x9f, 0x92, 0x96] => vec![0x00, 0x9f, 0x92, 0x96] ],
    chars_buf_over: [ cmd "-c" "--buf-size=1" assert "á" => err "could not fetch utf-8 character longer than '1' bytes\n" ],
    // seq 1 10000 | md5sum -t
    chars_big: [ sh "seq 1 10000 | %cmd% -c | md5sum -t" assert "" => "72d4ff27a28afbc066d5804999d5a504  -\n" ],
    lines_empty: [ cmd "-l" assert "" => "" ],
    lines_items: [ cmd "-l" assert "a\nbc\r\ndef\0ghij" => "a\nbc\ndef\0ghij\n" ],
    lines_buf_mode: [ cmd "-l" "--buf-mode=line" assert "a\nbc\r\ndef\0ghij" => "a\nbc\ndef\0ghij\n" ],
    lines_buf_under: [ cmd "-l" "--buf-size=8" assert "123456\n789" => "123456\n789\n" ],
    lines_buf_exact: [ cmd "-l" "--buf-size=8" assert "1234567\n89" => "1234567\n89\n" ],
    lines_buf_over: [ cmd "-l" "--buf-size=8" assert "12345678\n9" => err "could not fetch line longer than '8' bytes\n" ],
    lines_cr_buf_under: [ cmd "-l" "--buf-size=8" assert "12345\r\n6789" => "12345\n6789\n" ],
    lines_cr_buf_exact: [ cmd "-l" "--buf-size=8" assert "123456\r\n789" => "123456\n789\n" ],
    lines_cr_buf_over: [ cmd "-l" "--buf-size=8" assert "1234567\r\n89" => err "could not fetch line longer than '8' bytes\n" ],
    lines_non_utf8: [ cmd "-l" assert &[0x00, 0x9f, 0x92, 0x96] => vec![0x00, 0x9f, 0x92, 0x96, 0x0a] ],
    // seq 1 10000 | md5sum -t
    lines_big: [ sh "seq 1 10000 | %cmd% -l | md5sum -t" assert "" => "72d4ff27a28afbc066d5804999d5a504  -\n" ],
    records_empty: [ cmd "-l0" assert "" => "" ],
    records_items: [ cmd "-l0" assert "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij\0" ],
    records_buf_mode: [ cmd "-l0" "--buf-mode=line" assert "a\nbc\r\ndef\0ghij" => "a\nbc\r\ndef\0ghij\0" ],
    records_buf_under: [ cmd "-l0" "--buf-size=8" assert "123456\x00789" => "123456\x00789\0" ],
    records_buf_exact: [ cmd "-l0" "--buf-size=8" assert "1234567\089" => "1234567\089\0" ],
    records_buf_over: [ cmd "-l0" "--buf-size=8" assert "12345678\09" => err "could not fetch line longer than '8' bytes\n" ],
    records_non_utf8: [ cmd "-l0" assert &[0x00, 0x9f, 0x92, 0x96] => vec![0x00, 0x9f, 0x92, 0x96, 0x00] ],
    // seq 1 10000 | tr '\n' '\0' | md5sum -t
    records_big: [ sh "seq 1 10000 | tr '\\n' '\\0' | %cmd% -l0 | md5sum -t" assert "" => "05cb1e39ef75550ae349fb6f08cd6000  -\n" ],
});
