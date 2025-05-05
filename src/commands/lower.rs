use crate::cli::Args;
use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::ErrorKind;
use crate::cli::HELP;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::NULL;
use crate::run::Context;
use bstr::ByteSlice;

pub const LOWER: Command = CommandBuilder::new()
    .name("lower")
    .description("Convert characters to lowercase.")
    .options(&[HELP.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .run(run)
    .done();

fn run(args: Args) -> Result<(), ErrorKind<'static>> {
    let context = Context::new(&args);

    let mut reader = context.char_chunk_reader();
    let mut writer = context.writer();
    let mut buffer = context.uninit_buf();

    while let Some(chunk) = reader.read_chunk()? {
        if chunk.is_ascii() {
            chunk.make_ascii_lowercase();
            writer.write(chunk)?;
        } else {
            buffer.clear();
            chunk.to_lowercase_into(&mut buffer);
            writer.write(&buffer)?;
        }
    }

    Ok(())
}
