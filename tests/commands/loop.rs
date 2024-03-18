use crate::command_test;

command_test!("loop", {
    single_0: [ cmd "0" assert "ab" => "" ],
    single_1: [ cmd "1" assert "ab" => "ab" ],
    single_2: [ cmd "2" assert "ab" => "abab" ],
    single_3: [ cmd "3" assert "ab" => "ababab" ],
    multi_0: [ cmd "0" assert "a\nbc\n" => "" ],
    multi_1: [ cmd "1" assert "a\nbc\n" => "a\nbc\n" ],
    multi_2: [ cmd "2" assert "a\nbc\n" => "a\nbc\na\nbc\n" ],
    multi_3: [ cmd "3" assert "a\nbc\n" => "a\nbc\na\nbc\na\nbc\n" ],
    buff_over: [ cmd "8" "--buf-size=8" assert "012345" => "012345012345012345012345012345012345012345012345" ],
    // for((i=0; i<3; i++)); do seq 1 10000; done | cksum
    big_finite: [ sh "seq 1 10000 | %cmd% 3 | cksum" assert "" => "2921780161 146682\n" ],
    // for((i=0; i<10; i++)); do seq 1 10000; done | head -n40000 | cksum
    big_infinite: [ sh "seq 1 10000 | %cmd% | head -n40000 | cksum" assert "" => "745132148 195576\n" ],
});
