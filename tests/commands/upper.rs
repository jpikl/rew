use crate::command_test;

command_test!("upper", {
    ascii: [ cmd assert "abCD" => "ABCD" ],
    non_ascii: [ cmd assert "ábČD" => "ÁBČD" ],
    buf_size: [ cmd "--buf-size=8" assert "aáaáaáaá" => "AÁAÁAÁAÁ" ],
});
