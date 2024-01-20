use crate::command::Meta;

mod ascii;
mod cat;
mod first;
mod r#loop;
mod lower;
mod seq;
mod stream;
mod trim;
mod upper;

pub fn get_meta() -> Vec<&'static Meta> {
    vec![
        &ascii::META,
        &cat::META,
        &first::META,
        &r#loop::META,
        &lower::META,
        &seq::META,
        &stream::META,
        &trim::META,
        &upper::META,
    ]
}
