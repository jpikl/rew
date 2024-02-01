use crate::command_test;

command_test!("lower", {
    ascii: [ cmd should "abCD" => "abcd" ],
    non_ascii: [ cmd should "ábČD" => "ábčd" ],
});
