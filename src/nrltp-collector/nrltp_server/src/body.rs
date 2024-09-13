use super::{
    NrltpDatagram, NrltpHunk
};
use std::time::{
    SystemTime, UNIX_EPOCH
};

//TODO: implement timestamp hunk type
#[derive(Debug)]
pub enum HunkBody {
    ClientId(ClientIdHunk),
    IntMetric(IntMetricHunk),
    FloatMetric(FloatMetricHunk),
    Empty,
}

#[derive(Debug)]
pub struct ClientIdHunk {
    client_id: String
}

impl ClientIdHunk {
    pub fn new(datagram: &mut NrltpDatagram, hunk: &NrltpHunk) -> Option<Self> {
        let n = hunk.body_size() as usize;
        let mut vec = Vec::with_capacity(n);
        for _ in 0..n {
            if let Some(b) = datagram.read() {
                vec.push(b);
            }
            else {
                println!("Parse ClientID Error");
                return None
            }
        }
        // Since it is a printable-ASCII string, it's a valid UTF-8 string as well.
        let client_id = String::from_utf8_lossy(&vec).into_owned();
        println!("Parse ClientID OK = {}", client_id);
        Some(
            Self {
                client_id
            }
        )
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum MetricType {
    Gauge = 0,
    Count = 1,
    Unknown
}

impl From<u8> for MetricType {
    fn from(metric_type: u8) -> Self {
        match metric_type {
            0 => return MetricType::Gauge,
            1 => return MetricType::Count,
            _ => return MetricType::Unknown,
        };
    }
}

#[derive(Debug)]
pub struct IntMetricHunk {
    metric_type: MetricType,
    metric_name: String,
    timestamp: SystemTime,
    /// Vector of tuples: (Metric Value, Time Offset in millis)
    metrics: Vec<(i32, u16)>,
    /// Used by the Iterator
    metrics_index: usize,
}

impl Default for IntMetricHunk {
    fn default() -> Self {
        Self {
            metric_type: MetricType::Unknown,
            metric_name: Default::default(),
            timestamp: SystemTime::now(),
            metrics: Default::default(),
            metrics_index: 0
        }
    }
}

impl IntMetricHunk {
    pub fn new(datagram: &mut NrltpDatagram, hunk: &NrltpHunk) -> Option<Self> {
        let mut hunk_body = IntMetricHunk::default();
        if hunk_body.parse_header(datagram) {
            let sz = hunk.body_size() as usize;
            let num_metrics = (sz - hunk_body.metric_name.len() - 1) / 6;
            println!("Number of int metrics = {}", num_metrics);
            if hunk_body.parse_metrics(datagram, hunk, num_metrics) {
                Some(hunk_body)
            }
            else {
                None
            }
        }
        else {
            None
        }
    }

    pub fn metric_name(&self) -> &str {
        &self.metric_name
    }

    pub fn metric_type(&self) -> MetricType {
        self.metric_type
    }

    pub fn timestamp(&self) -> u64 {
        let since_the_epoch = self.timestamp
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_secs()
    }

    fn parse_header(&mut self, datagram: &mut NrltpDatagram) -> bool {
        if let Some(b) = datagram.read() {
            let metric_type = (b >> 5) & 0b111;
            let metric_name_size = (b & 0b11111) + 1;
            self.metric_type = metric_type.into();
            let n = metric_name_size as usize;
            let mut vec = Vec::with_capacity(n);
            for _ in 0..n {
                if let Some(b) = datagram.read() {
                    vec.push(b);
                }
                else {
                    println!("Parse Int Metric Name Error");
                    return false
                }
            }
            self.metric_name = String::from_utf8_lossy(&vec).into_owned();
            println!("Parse Metric Name OK = {}", self.metric_name);
            true
        }
        else {
            println!("Parse Int Metric Header Error");
            false
        }
    }

    fn parse_metrics(&mut self, datagram: &mut NrltpDatagram, hunk: &NrltpHunk, num_metrics: usize) -> bool {
        for _ in 0..num_metrics {
            let metric_value = if let Ok(metric_value) = hunk.endianness().read_i32(datagram) {
                metric_value
            }
            else {
                println!("Parse int metric Error read value");
                return false
            };

            let time_offset = if let Ok(time_offset) = hunk.endianness().read_u16(datagram) {
                time_offset
            }
            else {
                println!("Parse int metric Error read time offset");
                return false
            };
            println!("Read metric value = {:x} with time offset = {}", metric_value, time_offset);
            self.metrics.push((metric_value, time_offset));
        }
        true
    }
}

impl Iterator for IntMetricHunk {
    type Item = (i32, u16);

    fn next(&mut self) -> Option<Self::Item> {
        if self.metrics_index < self.metrics.len() {
            let val = self.metrics[self.metrics_index];
            self.metrics_index += 1;
            Some(val)
        }
        else {
            None
        }
    }
}

#[derive(Debug)]
pub struct FloatMetricHunk {
    //TODO
}

impl FloatMetricHunk {
    pub fn new(_datagram: &mut NrltpDatagram, _hunk: &NrltpHunk) -> Option<Self> {
        None
    }
}
