use crate::command_test;

command_test!("stream", {
    none: [ cmd assert "" => "" ],
    one: [ cmd "a" assert "" => "a\n" ],
    two: [ cmd "a" "bc" assert "" => "a\nbc\n" ],
    three: [ cmd "a" "bc" "def" assert "" => "a\nbc\ndef\n" ],
});
