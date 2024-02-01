use crate::command_test;

command_test!("loop", {
    single_0: [ cmd "0" should "ab" => "" ],
    single_1: [ cmd "1" should "ab" => "ab" ],
    single_2: [ cmd "2" should "ab" => "abab" ],
    single_3: [ cmd "3" should "ab" => "ababab" ],
    multi_0: [ cmd "0" should "a\nbc\n" => "" ],
    multi_1: [ cmd "1" should "a\nbc\n" => "a\nbc\n" ],
    multi_2: [ cmd "2" should "a\nbc\n" => "a\nbc\na\nbc\n" ],
    multi_3: [ cmd "3" should "a\nbc\n" => "a\nbc\na\nbc\na\nbc\n" ],
    buff_over: [ cmd "8" "--buf-size=8" should "012345" => "012345012345012345012345012345012345012345012345" ],
    // for((i=0; i<3; i++)); do seq 1 10000; done | md5sum
    big_finite: [ sh "seq 1 10000 | %cmd% 3 | md5sum" should "" => "5f0f7f173c062a0d128ff75ded51b09b  -\n" ],
    // for((i=0; i<10; i++)); do seq 1 10000; done | head -n40000 | md5sum
    big_infinite: [ sh "seq 1 10000 | %cmd% | head -n40000 | md5sum" should "" => "3aeff2d35d8836dfbbdfe882848b5d30  -\n" ],
});
