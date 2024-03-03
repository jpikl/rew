use crate::command_test;

command_test!("lower", {
    ascii: [ cmd assert "abCD" => "abcd" ],
    non_ascii: [ cmd assert "ábČD" => "ábčd" ],
    buf_size: [ cmd "--buf-size=8" assert "AÁAÁAÁAÁ" => "aáaáaáaá" ],
});
