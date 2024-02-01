use crate::command_test;

command_test!("first", {
    default: [ cmd should "a\nbc\ndef\n" => "a\n" ],
    count_0: [ cmd "0" should "a\nbc\ndef\n" => "" ],
    count_1: [ cmd "1" should "a\nbc\ndef\n" => "a\n" ],
    count_2: [ cmd "2" should "a\nbc\ndef\n" => "a\nbc\n" ],
    count_3: [ cmd "3" should "a\nbc\ndef\n" => "a\nbc\ndef\n" ],
    count_4: [ cmd "4" should "a\nbc\ndef\n" => "a\nbc\ndef\n" ],
    buf_under: [ cmd "--buf-size=8" should "aaaaaa\nb\n" => "aaaaaa\n" ],
    buf_exact: [ cmd "--buf-size=8" should "aaaaaaa\nb\n" => "aaaaaaa\n" ],
    buf_over: [ cmd "--buf-size=8" should "aaaaaaaa\nb\n" => "aaaaaaaa\n" ],
    buf_over_2: [ cmd "--buf-size=8" should "aaaaaaaaa\nb\n" => "aaaaaaaaa\n" ],
    // seq 1 10000 | head -n9999 | md5sum
    many: [ sh "seq 1 10000 | %cmd% 9999 | md5sum" should "" => "05fda6bec6aabc94d0fc54380ace8412  -\n" ],
});
