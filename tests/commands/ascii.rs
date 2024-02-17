use crate::command_test;

command_test!("ascii", {
    replace_none: [ cmd  assert "abCD" => "abCD" ],
    replace_some: [ cmd assert "ábČD" => "abCD" ],
    delete_none: [ cmd "-d" assert "abCD" => "abCD" ],
    delete_some: [ cmd "-d" assert "ábČD" => "bD" ],
});
