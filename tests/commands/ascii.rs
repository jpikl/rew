use crate::command_test;

command_test!("ascii", {
    replace_none: [ cmd  assert "abCD" => "abCD" ],
    replace_some: [ cmd assert "ábČD" => "abCD" ],
    replace_buf_size: [ cmd "--buf-size=8" assert "aáaáaáaá" => "aaaaaaaa" ],
    delete_none: [ cmd "-d" assert "abCD" => "abCD" ],
    delete_some: [ cmd "-d" assert "ábČD" => "bD" ],
    delete_buf_size: [ cmd "-d" "--buf-size=8" assert "aáaáaáaá" => "aaaa" ],
});
