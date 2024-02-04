use crate::command_test;

command_test!("x", {
    empty_none: [ cmd "" should "" => "" ],
    empty_many: [ cmd "" should "a\nbc" => "\n\n" ],
    constant_none: [ cmd "xyz" should "" => "" ],
    constant_many: [ cmd "xyz" should "a\nbc" => "xyz\nxyz\n" ],
    empty_expr_none: [ cmd "{}" should "" => "" ],
    empty_expr_many: [ cmd "{}" should "a\nbc" => "a\nbc\n" ],
    simple_none: [ cmd "x_{}_y_{}_z" should "" => "" ],
    simple_many: [ cmd "x_{}_y_{}_z" should "a\nbc" => "x_a_y_a_z\nx_bc_y_bc_z\n" ],
    escaped_many: [ cmd "x_\\{\\}_y" should "a\nbc" => "x_{}_y\nx_{}_y\n" ],
    escaped_alt_many: [ cmd "-e%" "x_%{%}_y" should "a\nbc" => "x_{}_y\nx_{}_y\n" ],
    internal_cmd_none: [ cmd "x_{trim -s}_y" should "" => "" ],
    internal_cmd_many: [ cmd "x_{trim -s}_y" should " a \n  bc  " => "x_a _y\nx_bc  _y\n" ],
    internal_cmd_wrapped: [ cmd "x_{rew trim -s}_y" should " a \n  bc  " => "x_a _y\nx_bc  _y\n" ],
    external_cmd_none: [ cmd "x_{tr -d 'b'}_y" should "" => "" ],
    external_cmd_many: [ cmd "x_{tr -d 'b'}_y" should "a\nbc" => "x_a_y\nx_c_y\n" ],
    pipeline_none: [ cmd "x_{trim -s | tr -d 'b'}_y" should "" => "" ],
    pipeline_many: [ cmd "x_{trim -s | tr -d 'b'}_y" should " a \n  bc  " => "x_a _y\nx_c  _y\n" ],
    complex_none: [ cmd "x_{}_{trim -s}_{tr -d 'b'}_{trim -s | tr -d 'b'}_y" should "" => "" ],
    complex_many: [ cmd "x_{}_{trim -s}_{tr -d 'b'}_{trim -s | tr -d 'b'}_y" should " a \n  bc  " => "x_ a _a _ a _a _y\nx_  bc  _bc  _  c  _c  _y\n" ],
    line_buf_none: [ cmd "--buf-mode=line" "x_{}_{trim -s}_{tr -d 'b'}_{trim -s | tr -d 'b'}_y" should "" => "" ],
    line_buf_many: [ cmd "--buf-mode=line" "x_{}_{trim -s}_{tr -d 'b'}_{trim -s | tr -d 'b'}_y" should " a \n  bc  " => "x_ a _a _ a _a _y\nx_  bc  _bc  _  c  _c  _y\n" ],
    records_none: [ cmd "-0" "x_{}_{trim -s}_{tr -d 'b'}_{trim -s | tr -d 'b'}_y" should "" => "" ],
    records_many: [ cmd "-0" "x_{}_{trim -s}_{tr -d 'b'}_{trim -s | tr -d 'b'}_y" should " a \0  bc  " => "x_ a _a _ a _a _y\0x_  bc  _bc  _  c  _c  _y\0" ],
    generator: [ cmd "x_{seq 1..2}_y" should "" => "x_1_y\nx_2_y\n" ],
    cat_and_generator_none: [ cmd "x_{}_{seq 1..2}_y" should "" => "" ],
    cat_and_generator_less: [ cmd "x_{}_{seq 1..2}_y" should "a" => "x_a_1_y\n" ],
    cat_and_generator_eq: [ cmd "x_{}_{seq 1..2}_y" should "a\nbc" => "x_a_1_y\nx_bc_2_y\n" ],
    cat_and_generator_more: [ cmd "x_{}_{seq 1..2}_y" should "a\nbc\ndef" => "x_a_1_y\nx_bc_2_y\n" ],
    // Should not get stuck by pipeline command not reading its stdin
    cat_and_generator_many: [ sh "seq 1 100000 | %cmd% 'x_{}_{seq 1..100000}_y' | wc -l" should "" => "100000\n" ],
});
