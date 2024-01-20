use crate::command::CommandMeta;

mod ascii;
mod cat;
mod first;
mod r#loop;
mod lower;
mod trim;
mod upper;

pub fn get_commands() -> Vec<&'static CommandMeta> {
    vec![
        &ascii::META,
        &cat::META,
        &first::META,
        &r#loop::META,
        &lower::META,
        &trim::META,
        &upper::META,
    ]
}
