use crate::command_test;

command_test!("last", {
    default: [ cmd assert "a\nbc\ndef\n" => "def\n" ],
    count_0: [ cmd "0" assert "a\nbc\ndef\n" => "" ],
    count_0_unt: [ cmd "0" assert "a\nbc\ndef" => "" ],
    count_1: [ cmd "1" assert "a\nbc\ndef\n" => "def\n" ],
    count_1_unt: [ cmd "1" assert "a\nbc\ndef" => "def" ],
    count_2: [ cmd "2" assert "a\nbc\ndef\n" => "bc\ndef\n" ],
    count_2_unt: [ cmd "2" assert "a\nbc\ndef" => "bc\ndef" ],
    count_3: [ cmd "3" assert "a\nbc\ndef\n" => "a\nbc\ndef\n" ],
    count_3_unt: [ cmd "3" assert "a\nbc\ndef" => "a\nbc\ndef" ],
    count_4: [ cmd "4" assert "a\nbc\ndef\n" => "a\nbc\ndef\n" ],
    count_4_unt: [ cmd "4" assert "a\nbc\ndef" => "a\nbc\ndef" ],
    buf_under_2: [ cmd "--buf-size=8" assert "aaaaa\nb\n" => "b\n" ],
    buf_under: [ cmd "--buf-size=8" assert "aaaaaa\nb\n" => "b\n" ],
    buf_exact: [ cmd "--buf-size=8" assert "aaaaaaa\nb\n" => "b\n" ],
    buf_over: [ cmd "--buf-size=8" assert "aaaaaaaa\nb\n" => "b\n" ],
    buf_over_2: [ cmd "--buf-size=8" assert "aaaaaaaaa\nb\n" => "b\n" ],
    // seq 1 20000 | tail -n10000 | md5sum -t
    many: [ sh "seq 1 20000 | %cmd% 10000 | md5sum -t" assert "" => "8857ef28723cc4788a8ca7456214fc0c  -\n" ],
});
