use crate::command_test;

command_test!("stream", {
    none: [ cmd should "" => "" ],
    one: [ cmd "a" should "" => "a\n" ],
    two: [ cmd "a" "bc" should "" => "a\nbc\n" ],
    three: [ cmd "a" "bc" "def" should "" => "a\nbc\ndef\n" ],
});
