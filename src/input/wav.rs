use std::fs::File;
use std::io::Read;
use std::str;
use std::error::Error;

#[derive(Debug)]
pub struct RIFFHeader {
    pub riff: String,
    pub file_size: u32,
    pub four_cc: String,
}

impl RIFFHeader {
    fn new(bytes: &[u8; 12]) -> Result<RIFFHeader, String> {
        let riff = match str::from_utf8(&bytes[0..4]) {
            Ok("RIFF") => "RIFF".to_string(),
            Ok(&_) => return Err("first four bytes are not RIFF".to_string()),
            Err(e) => return Err(e.to_string()),
        };
        let file_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let four_cc = match str::from_utf8(&bytes[8..12]) {
            Ok(a) => a.to_string(),
            Err(e) => return Err(e.to_string()),
        };
        let header = RIFFHeader {
            riff: riff,
            file_size: file_size,
            four_cc: four_cc,
        };
        Ok(header)
    }
}

#[derive(Debug)]
pub struct FMTHeader {
    pub fmt: String,
    pub header_size: u32,
    pub format: u16,
    pub nchannels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
}

impl FMTHeader {
    fn new(bytes: &[u8; 24]) -> Result<FMTHeader, String> {
        let fmt = match str::from_utf8(&bytes[0..4]) {
            Ok("fmt ") => "fmt ".to_string(),
            Ok(&_) => return Err("header does not start with FMT".to_string()),
            Err(e) => return Err(e.to_string()),
        };
        let header_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let format = u16::from_le_bytes([bytes[8], bytes[9]]);
        let nchannels = u16::from_le_bytes([bytes[10], bytes[11]]);
        let sample_rate = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let byte_rate = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let block_align = u16::from_le_bytes([bytes[20], bytes[21]]);
        let bits_per_sample = u16::from_le_bytes([bytes[22], bytes[23]]);
        let header = FMTHeader {
            fmt: fmt,
            header_size: header_size,
            format: format,
            nchannels: nchannels,
            sample_rate: sample_rate,
            byte_rate: byte_rate,
            block_align: block_align,
            bits_per_sample: bits_per_sample,
        };
        Ok(header)
    }
}

#[derive(Debug)]
pub struct DataHeader {
    pub data: String,
    pub size: u32,
}

impl DataHeader {
    fn new(bytes: &[u8; 8]) -> Result<DataHeader, String> {
        let data = match str::from_utf8(&bytes[0..4]) {
            Ok("data") => "data".to_string(),
            Ok(&_) => return Err("header does not start with data".to_string()),
            Err(e) => return Err(e.to_string()),
        };
        let size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let header = DataHeader {
            data: data,
            size: size,
        };
        Ok(header)
    }
}

pub fn read_file(filename: &str) -> Result<(RIFFHeader, FMTHeader, DataHeader, Vec<i16>), Box<dyn Error>> {
    //let mut f = File::open(file).unwrap();
    let mut f = File::open(filename).unwrap();

    let mut buf = [0u8; 12];
    f.read(&mut buf).unwrap();
    let riff_h = RIFFHeader::new(&buf).unwrap();

    let mut buf = [0u8; 24];
    f.read(&mut buf).unwrap();
    let fmt_h = FMTHeader::new(&buf).unwrap();

    let mut buf = [0u8; 8];
    f.read(&mut buf).unwrap();
    let data_h = DataHeader::new(&buf).unwrap();

    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    let signal: Vec<i16> = buf
        .chunks(2)
        .map(|x| i16::from_le_bytes([x[0], x[1]]))
        .collect();

    Ok((riff_h, fmt_h, data_h, signal))
}
