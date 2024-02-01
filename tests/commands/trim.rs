use crate::command_test;

command_test!("trim", {
    default: [ cmd should "  \n a \n  b  c  " => "\na\nb  c\n" ],
    start: [ cmd "-s" should "  \n a \n  b  c  " => "\na \nb  c  \n" ],
    end: [ cmd "-e" should "  \n a \n  b  c  " => "\n a\n  b  c\n" ],
    both: [ cmd "-se" should "  \n a \n  b  c  " => "\na\nb  c\n" ],
});
