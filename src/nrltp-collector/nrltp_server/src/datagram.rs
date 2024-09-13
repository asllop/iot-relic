use super::NrltpHunk;

pub struct NrltpDatagram<'a> {
    buffer: &'a [u8],
    size: usize,
    index: usize,
}

impl<'a> NrltpDatagram<'a> {
    pub fn new(buffer: &'a [u8], size: usize) -> Self {
        Self {
            buffer,
            size,
            index: 0
        }
    }

    pub fn read(&mut self) -> Option<u8> {
        if self.index < self.size {
            let byte = self.buffer[self.index];
            self.index += 1;
            Some(byte)
        }
        else {
            None
        }
    }
}

impl<'a> Iterator for NrltpDatagram<'a> {
    type Item = NrltpHunk;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.size {
            NrltpHunk::new(self)
        }
        else {
            println!("Finished reading datagram");
            None
        }
    }
}