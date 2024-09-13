use super::{
    NrltpDatagram, HunkBody, ClientIdHunk, IntMetricHunk, FloatMetricHunk
};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Endianness {
    Little = 0,
    Big = 1,
    Unknown,
}

impl From<u8> for Endianness {
    fn from(endianness: u8) -> Self {
        match endianness {
            0 => return Endianness::Little,
            1 => return Endianness::Big,
            _ => return Endianness::Unknown,
        };
    }
}

impl Endianness {
    pub fn read_u16(&self, d: &mut NrltpDatagram) -> Result<u16, ()> {
        match self {
            Self::Little => {
                if let (Some(b0), Some(b1)) = (d.read(), d.read()) {
                    let num = (b0 as u16 | ((b1 as u16) << 8)).into();
                    Ok(num)
                }
                else {
                    Err(())
                }
            },
            Self::Big => {
                if let (Some(b1), Some(b0)) = (d.read(), d.read()) {
                    let num = (b0 as u16 | ((b1 as u16) << 8)).into();
                    Ok(num)
                }
                else {
                    Err(())
                }
            },
            _ => { Err(()) }
        }
    }

    pub fn read_i32(&self, d: &mut NrltpDatagram) -> Result<i32, ()> {
        match self {
            Self::Little => {
                if let (Some(b0), Some(b1), Some(b2), Some(b3)) = (d.read(), d.read(), d.read(), d.read()) {
                    let num = (b0 as i32 | ((b1 as i32) << 8) | ((b2 as i32) << 16) | ((b3 as i32) << 24)).into();
                    Ok(num)
                }
                else {
                    Err(())
                }
            },
            Self::Big => {
                if let (Some(b3), Some(b2), Some(b1), Some(b0)) = (d.read(), d.read(), d.read(), d.read()) {
                    let num = (b0 as i32 | ((b1 as i32) << 8) | ((b2 as i32) << 16) | ((b3 as i32) << 24)).into();
                    Ok(num)
                }
                else {
                    Err(())
                }
            },
            _ => { Err(()) }
        }
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum HunkType {
    Reserved = 0,
    ClientId = 1,
    Timestamp = 2,
    IntMetric = 3,
    FloatMetric = 4,
    Unknown,
}

impl From<u8> for HunkType {
    fn from(hunk_type: u8) -> Self {
        match hunk_type {
            0 => return HunkType::Reserved,
            1 => return HunkType::ClientId,
            2 => return HunkType::Timestamp,
            3 => return HunkType::IntMetric,
            4 => return HunkType::FloatMetric,
            _ => return HunkType::Unknown,
        };
    }
}

pub struct NrltpHunk {
    endianness: Endianness,
    header_size: u8,
    body_size: u16,
    hunk_type: HunkType,
    body: HunkBody,
}

impl Default for NrltpHunk {
    fn default() -> Self {
        Self {
            endianness: Endianness::Unknown,
            header_size: 0,
            body_size: 0,
            hunk_type: HunkType::Unknown,
            body: HunkBody::Empty,
        }
    }
}

impl NrltpHunk {
    pub fn new(datagram: &mut NrltpDatagram) -> Option<Self> {
        if Self::parse_magic(datagram) && Self::parse_version(datagram) {
            let mut hunk = Self::default();
            if hunk.parse_type(datagram) && hunk.parse_endian_and_hsize(datagram) && hunk.parse_bsize(datagram) {
                match hunk.hunk_type {
                    HunkType::ClientId => {
                        if let Some(client_id) = ClientIdHunk::new(datagram, &hunk) {
                            hunk.body = HunkBody::ClientId(client_id);
                        }
                        else {
                            return None
                        }
                    },
                    HunkType::IntMetric => {
                        if let Some(int_metric) = IntMetricHunk::new(datagram, &hunk) {
                            hunk.body = HunkBody::IntMetric(int_metric);
                        }
                        else {
                            return None
                        }
                    },
                    HunkType::FloatMetric => {
                        if let Some(float_metric) = FloatMetricHunk::new(datagram, &hunk) {
                            hunk.body = HunkBody::FloatMetric(float_metric);
                        }
                        else {
                            return None
                        }
                    },
                    //TODO: if hunk type is unknown, just skip it reading BODY_SIZE bytes.
                    _ => { return None },
                }
                Some(hunk)
            }
            else {
                None
            }
        }
        else {
            None
        }
    }

    /// Moves self
    pub fn body(self) -> HunkBody {
        self.body
    }

    pub fn body_size(&self) -> u16 {
        self.body_size
    }

    pub fn endianness(&self) -> Endianness {
        self.endianness
    }

    fn parse_magic(d: &mut NrltpDatagram) -> bool {
        if let (Some(m0), Some(m1), Some(m2)) = (d.read(), d.read(), d.read()) {
            println!("Parse Magic OK");
            m0 == 0xAB && m1 == 0xBC && m2 == 0xCD
        }
        else {
            println!("Parse Magic Error");
            false
        }
    }

    fn parse_version(d: &mut NrltpDatagram) -> bool {
        if let Some(ver) = d.read() {
            println!("Parse Version OK = {}", ver);
            ver > 0
        }
        else {
            println!("Parse Version Error");
            false
        }
    }

    fn parse_type(&mut self, d: &mut NrltpDatagram) -> bool {
        if let Some(b) = d.read() {
            self.hunk_type = b.into();
            println!("Parse type OK = {}", self.hunk_type as u8);
            true
        }
        else {
            println!("Parse type Error");
            false
        }
    }

    fn parse_endian_and_hsize(&mut self, d: &mut NrltpDatagram) -> bool {
        if let Some(b) = d.read() {
            let endianness = (b >> 6) & 0b11;
            let header_size = (b & 0b111111) + 8;
            self.endianness = endianness.into();
            self.header_size = header_size;
            println!("Parse Endian & HSize OK = {} , {}", endianness as u8, header_size);
            true
        }
        else {
            println!("Parse Endian & HSize Error");
            false
        }
    }

    fn parse_bsize(&mut self, d: &mut NrltpDatagram) -> bool {
        let sz = if let Ok(sz) = self.endianness.read_u16(d) {
            sz
        }
        else {
            println!("Parse BSize Error read");
            return false
        };

        self.body_size = sz;

        // Skip remaining header size bytes
        let remaining = self.header_size - 8;
        for _ in 0..remaining {
            if let None = d.read() {
                println!("Parse BSize Error padding");
                return false;
            }
        }

        println!("Parse BSize OK = {}", self.body_size);

        true
    }
}