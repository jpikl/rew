use crate::command::Meta;

mod ascii;
mod cat;
mod first;
mod last;
mod r#loop;
mod lower;
mod seq;
mod skip;
mod stream;
mod trim;
mod upper;

pub fn get_meta() -> Vec<&'static Meta> {
    vec![
        &ascii::META,
        &cat::META,
        &first::META,
        &last::META,
        &r#loop::META,
        &lower::META,
        &seq::META,
        &skip::META,
        &stream::META,
        &trim::META,
        &upper::META,
    ]
}
