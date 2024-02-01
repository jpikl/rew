use crate::command_test;

command_test!("upper", {
    ascii: [ cmd should "abCD" => "ABCD" ],
    non_ascii: [ cmd should "ábČD" => "ÁBČD" ],
});
