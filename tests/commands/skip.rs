use crate::command_test;

command_test!("skip", {
    count_0: [ cmd "0" should "a\nbc\ndef\n" => "a\nbc\ndef\n" ],
    count_1: [ cmd "1" should "a\nbc\ndef\n" => "bc\ndef\n" ],
    count_2: [ cmd "2" should "a\nbc\ndef\n" => "def\n" ],
    count_3: [ cmd "3" should "a\nbc\ndef\n" => "" ],
    count_4: [ cmd "4" should "a\nbc\ndef\n" => "" ],
    buf_under_2: [ cmd "1" "--buf-size=8" should "aaaaa\nb\n" => "b\n" ],
    buf_under: [ cmd "1" "--buf-size=8" should "aaaaaa\nb\n" => "b\n" ],
    buf_exact: [ cmd "1" "--buf-size=8" should "aaaaaaa\nb\n" => "b\n" ],
    buf_over: [ cmd "1" "--buf-size=8" should "aaaaaaaa\nb\n" => "b\n" ],
    buf_over_2: [ cmd "1" "--buf-size=8" should "aaaaaaaaa\nb\n" => "b\n" ],
    // seq 1 20000 | tail -n-10000 | md5sum
    many: [ sh "seq 1 20000 | %cmd% 10000 | md5sum" should "" => "8857ef28723cc4788a8ca7456214fc0c  -\n" ],
    many_buf_line: [ sh "seq 1 20000 | %cmd% --buf-mode=line 10000 | md5sum" should "" => "8857ef28723cc4788a8ca7456214fc0c  -\n" ],
});
