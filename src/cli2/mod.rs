mod build;
mod model;
mod parse;

use crate::define_ids;
pub use build::*;
pub use model::*;
pub use parse::*;

define_ids!(Id::zero(), 1, {
    CMD_ID,
    FLAG_ID,
    OPT_ID,
    POS_ID,
});

const FLAG: TypedArg<bool> = FlagBuilder::new()
    .id(FLAG_ID)
    .short('f')
    .long("flag")
    .description(&["Desc"])
    .done();

const OPT: TypedArg<i32> = OptBuilder::new()
    .id(OPT_ID)
    .short('o')
    .long("opt")
    .description(&["Desc"])
    .done();

const POS: TypedArg<String> = PosBuilder::new()
    .id(POS_ID)
    .name("pos")
    .description(&["Desc"])
    .required()
    .done();

const CMD: Command = CommandBuilder::new()
    .id(CMD_ID)
    .name("cmd")
    .description(&["Desc"])
    .args(&[&FLAG, &OPT, &POS])
    .done();

pub fn main() -> Result<(), ParseError> {
    let mut parser = CMD.parse(RawArgs::from_env());

    let mut flag = FLAG.placeholder();
    let mut opt = OPT.placeholder();
    let mut pos = POS.placeholder();

    parser.process(&mut [&mut flag, &mut opt, &mut pos])?;

    let flag = flag.value()?;
    let opt = opt.value()?;
    let pos = pos.value()?;

    println!("flag: {flag}");
    println!("opt: {opt}");
    println!("pos: {pos}");

    Ok(())
}
