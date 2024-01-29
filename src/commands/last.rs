use crate::args::GlobalArgs;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use crate::io::BufSizeConfig;
use crate::io::LineConfig;
use crate::io::Writer;
use anyhow::Result;
use memchr::memchr;
use std::collections::LinkedList;
use std::io::stdin;
use std::io::Read;
use std::mem::replace;

pub const META: Meta = command_meta! {
    name: "last",
    group: Group::Filters,
    args: Args,
    run: run,
};

/// Output last N input lines.
#[derive(clap::Args)]
struct Args {
    /// Number of lines to print.
    #[arg(default_value_t = 1)]
    count: usize,
}

fn run(global_args: &GlobalArgs, args: &Args) -> Result<()> {
    if args.count == 0 {
        return Ok(());
    }

    let mut reader = stdin().lock();
    let mut buffers = LinkedList::new();
    let mut last_buf = LineBuf::new(global_args.buf_size());
    let mut total_lines = 0;
    let separator = global_args.line_separator().as_byte();

    loop {
        let len = reader.read(last_buf.available())?;
        if len == 0 {
            if !last_buf.is_empty() {
                buffers.push_back(last_buf);
            }
            break;
        }

        total_lines += last_buf.advance(len, separator);

        if last_buf.is_full() {
            let new_buf = LineBuf::new(global_args.buf_size());
            buffers.push_back(replace(&mut last_buf, new_buf));

            if let Some(first_buf) = buffers.front() {
                if total_lines - first_buf.lines > args.count {
                    total_lines -= first_buf.lines;
                    buffers.pop_front();
                }
            }
        }
    }

    let last_line_terminated = buffers
        .back()
        .map(|buf| buf.is_terminated_with(separator))
        .unwrap_or_default();

    if !last_line_terminated {
        total_lines += 1;
    }

    let mut writer = Writer::from_stdout(global_args);

    for buf in buffers {
        let mut data = buf.used();

        if total_lines > args.count {
            while let Some(pos) = memchr(separator, data) {
                data = &data[(pos + 1)..];
                total_lines -= 1;

                if total_lines == args.count {
                    writer.write_block(data)?;
                    break;
                }
            }
        } else {
            writer.write_block(data)?;
        }
    }

    Ok(())
}

struct LineBuf {
    data: Vec<u8>,
    end: usize,
    lines: usize,
}

impl LineBuf {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
            end: 0,
            lines: 0,
        }
    }

    fn used(&self) -> &[u8] {
        &self.data[..self.end]
    }

    fn available(&mut self) -> &mut [u8] {
        &mut self.data[self.end..]
    }

    fn advance(&mut self, len: usize, separator: u8) -> usize {
        let next_end = self.end + len;
        let mut area = &self.data[self.end..next_end];
        let mut lines = 0;

        while let Some(pos) = memchr(separator, area) {
            lines += 1;
            area = &area[(pos + 1)..];
        }

        self.lines += lines;
        self.end = next_end;
        lines
    }

    fn is_empty(&self) -> bool {
        self.end == 0
    }

    fn is_full(&self) -> bool {
        self.end == self.data.len()
    }

    fn is_terminated_with(&self, separator: u8) -> bool {
        self.end > 0 && self.data[self.end - 1] == separator
    }
}
