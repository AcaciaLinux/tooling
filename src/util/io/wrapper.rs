pub struct DynReadWrapper<'a> {
    inner: &'a mut dyn std::io::Read,
}

impl<'a> DynReadWrapper<'a> {
    pub fn new(inner: &'a mut dyn std::io::Read) -> Self {
        Self { inner }
    }
}

impl std::io::Read for DynReadWrapper<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

pub struct DynWriteWrapper<'a> {
    inner: &'a mut dyn std::io::Write,
}

impl<'a> DynWriteWrapper<'a> {
    pub fn new(inner: &'a mut dyn std::io::Write) -> Self {
        Self { inner }
    }
}

impl std::io::Write for DynWriteWrapper<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
