use crate::command_test;

command_test!("join", {
    empty_0: [ cmd ":" should "" => "\n" ],
    empty_1: [ cmd ":" should "\n" => "\n" ],
    empty_2: [ cmd ":" should "\n\n" => ":\n" ],
    many: [ cmd ":" should "a\nbc\ndef" => "a:bc:def\n" ],
    many_nl: [ cmd ":" should "a\nbc\ndef\n" => "a:bc:def\n" ],
    buf_under: [ cmd ":" "--buf-size=8" should "aaaaaa\nb" => "aaaaaa:b\n" ],
    buf_exact: [ cmd ":" "--buf-size=8" should "aaaaaaa\nb" => "aaaaaaa:b\n" ],
    buf_over: [ cmd ":" "--buf-size=8" should "aaaaaaaa\nb" => "aaaaaaaa:b\n" ],
    buf_over_2: [ cmd ":" "--buf-size=8" should "aaaaaaaaa\nb" => "aaaaaaaaa:b\n" ],
    trail_empty_0: [ cmd "-t" ":" should "" => ":\n" ],
    trail_empty_1: [ cmd "-t" ":" should "\n" => ":\n" ],
    trail_empty_2: [ cmd "-t" ":" should "\n\n" => "::\n" ],
    trail_many: [ cmd "-t" ":" should "a\nbc\ndef" => "a:bc:def:\n" ],
    trail_many_nl: [ cmd "-t" ":" should "a\nbc\ndef\n" => "a:bc:def:\n" ],
    trail_buf_under: [ cmd "-t" ":" "--buf-size=8" should "aaaaaa\nb" => "aaaaaa:b:\n" ],
    trail_buf_exact: [ cmd "-t" ":" "--buf-size=8" should "aaaaaaa\nb" => "aaaaaaa:b:\n" ],
    trail_buf_over: [ cmd "-t" ":" "--buf-size=8" should "aaaaaaaa\nb" => "aaaaaaaa:b:\n" ],
    trail_buf_over_2: [ cmd "-t" ":" "--buf-size=8" should "aaaaaaaaa\nb" => "aaaaaaaaa:b:\n" ],
});
