use crate::command::CommandMeta;

mod ascii;
mod cat;
mod first;
mod trim;

pub fn get_commands() -> Vec<&'static CommandMeta> {
    vec![&ascii::META, &cat::META, &first::META, &trim::META]
}
