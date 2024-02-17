use crate::command_test;

command_test!("trim", {
    default: [ cmd assert "  \n a \n  b  c  " => "\na\nb  c\n" ],
    start: [ cmd "-s" assert "  \n a \n  b  c  " => "\na \nb  c  \n" ],
    end: [ cmd "-e" assert "  \n a \n  b  c  " => "\n a\n  b  c\n" ],
    both: [ cmd "-se" assert "  \n a \n  b  c  " => "\na\nb  c\n" ],
});
