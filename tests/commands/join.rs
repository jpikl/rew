use crate::command_test;

command_test!("join", {
    empty_0: [ cmd ":" assert "" => "\n" ],
    empty_1: [ cmd ":" assert "\n" => "\n" ],
    empty_2: [ cmd ":" assert "\n\n" => ":\n" ],
    many: [ cmd ":" assert "a\nbc\ndef" => "a:bc:def\n" ],
    many_nl: [ cmd ":" assert "a\nbc\ndef\n" => "a:bc:def\n" ],
    buf_under: [ cmd ":" "--buf-size=8" assert "aaaaaa\nb" => "aaaaaa:b\n" ],
    buf_exact: [ cmd ":" "--buf-size=8" assert "aaaaaaa\nb" => "aaaaaaa:b\n" ],
    buf_over: [ cmd ":" "--buf-size=8" assert "aaaaaaaa\nb" => "aaaaaaaa:b\n" ],
    buf_over_2: [ cmd ":" "--buf-size=8" assert "aaaaaaaaa\nb" => "aaaaaaaaa:b\n" ],
    trail_empty_0: [ cmd "-t" ":" assert "" => ":\n" ],
    trail_empty_1: [ cmd "-t" ":" assert "\n" => ":\n" ],
    trail_empty_2: [ cmd "-t" ":" assert "\n\n" => "::\n" ],
    trail_many: [ cmd "-t" ":" assert "a\nbc\ndef" => "a:bc:def:\n" ],
    trail_many_nl: [ cmd "-t" ":" assert "a\nbc\ndef\n" => "a:bc:def:\n" ],
    trail_buf_under: [ cmd "-t" ":" "--buf-size=8" assert "aaaaaa\nb" => "aaaaaa:b:\n" ],
    trail_buf_exact: [ cmd "-t" ":" "--buf-size=8" assert "aaaaaaa\nb" => "aaaaaaa:b:\n" ],
    trail_buf_over: [ cmd "-t" ":" "--buf-size=8" assert "aaaaaaaa\nb" => "aaaaaaaa:b:\n" ],
    trail_buf_over_2: [ cmd "-t" ":" "--buf-size=8" assert "aaaaaaaaa\nb" => "aaaaaaaaa:b:\n" ],
});
