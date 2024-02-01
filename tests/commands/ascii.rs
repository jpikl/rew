use crate::command_test;

command_test!("ascii", {
    replace_none: [ cmd  should "abCD" => "abCD" ],
    replace_some: [ cmd should "ábČD" => "abCD" ],
    delete_none: [ cmd "-d" should "abCD" => "abCD" ],
    delete_some: [ cmd "-d" should "ábČD" => "bD" ],
});
