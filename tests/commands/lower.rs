use crate::command_test;

command_test!("lower", {
    ascii: [ cmd assert "abCD" => "abcd" ],
    non_ascii: [ cmd assert "ábČD" => "ábčd" ],
});
