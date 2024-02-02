use crate::command_test;

command_test!("x", {
    empty_none: [ cmd "" should "" => "" ],
    empty_many: [ cmd "" should "a\nbc\ndef" => "\n\n\n" ],
    constant_none: [ cmd "xyz" should "" => "" ],
    constant_many: [ cmd "xyz" should "a\nbc\ndef" => "xyz\nxyz\nxyz\n" ],
    empty_expr_none: [ cmd "{}" should "" => "" ],
    empty_expr_many: [ cmd "{}" should "a\nbc\ndef" => "a\nbc\ndef\n" ],
    simple_none: [ cmd "pre_{}_mid_{}_post" should "" => "" ],
    simple_many: [ cmd "pre_{}_mid_{}_post" should "a\nbc\ndef" => "pre_a_mid_a_post\npre_bc_mid_bc_post\npre_def_mid_def_post\n" ],
});
