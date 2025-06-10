use itoap::Integer;

pub struct Formatter {
    buffer: Vec<u8>,
}

impl Formatter {
    pub fn with_buf_size(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    pub fn format_int<T: Integer>(&mut self, value: T) -> &[u8] {
        self.buffer.clear();
        itoap::write_to_vec(&mut self.buffer, value);
        &self.buffer
    }
}
