use std::io::Write;

/// A structure that implements [Write] that writes nowhere, comparable with piping to `/dev/null`
#[derive(Default)]
pub struct NullSink {}

impl Write for NullSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
