use crate::command_test;

command_test!("first", {
    default: [ cmd assert "a\nbc\ndef\n" => "a\n" ],
    count_0: [ cmd "0" assert "a\nbc\ndef\n" => "" ],
    count_1: [ cmd "1" assert "a\nbc\ndef\n" => "a\n" ],
    count_2: [ cmd "2" assert "a\nbc\ndef\n" => "a\nbc\n" ],
    count_3: [ cmd "3" assert "a\nbc\ndef\n" => "a\nbc\ndef\n" ],
    count_4: [ cmd "4" assert "a\nbc\ndef\n" => "a\nbc\ndef\n" ],
    buf_under: [ cmd "--buf-size=8" assert "aaaaaa\nb\n" => "aaaaaa\n" ],
    buf_exact: [ cmd "--buf-size=8" assert "aaaaaaa\nb\n" => "aaaaaaa\n" ],
    buf_over: [ cmd "--buf-size=8" assert "aaaaaaaa\nb\n" => "aaaaaaaa\n" ],
    buf_over_2: [ cmd "--buf-size=8" assert "aaaaaaaaa\nb\n" => "aaaaaaaaa\n" ],
    // seq 1 10000 | head -n9999 | md5sum -t
    many: [ sh "seq 1 10000 | %cmd% 9999 | md5sum -t" assert "" => "05fda6bec6aabc94d0fc54380ace8412  -\n" ],
});
