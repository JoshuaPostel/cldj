use std::error::Error;
use std::fs::File;
use std::io::{Read, Write, BufWriter};
use std::str;

use byteorder::{LittleEndian, WriteBytesExt};

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

    fn write<W: Write>(self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        writer.write(self.riff.as_bytes())?;
        writer.write_u32::<LittleEndian>(self.file_size)?;
        writer.write(self.four_cc.as_bytes())?;
        Ok(())
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

    fn write<W: Write>(self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        writer.write(self.fmt.as_bytes())?;
        writer.write_u32::<LittleEndian>(self.header_size)?;
        writer.write_u16::<LittleEndian>(self.format)?;
        writer.write_u16::<LittleEndian>(self.nchannels)?;
        writer.write_u32::<LittleEndian>(self.sample_rate)?;
        writer.write_u32::<LittleEndian>(self.byte_rate)?;
        writer.write_u16::<LittleEndian>(self.block_align)?;
        writer.write_u16::<LittleEndian>(self.bits_per_sample)?;
        Ok(())
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

    fn write<W: Write>(self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        writer.write(self.data.as_bytes())?;
        writer.write_u32::<LittleEndian>(self.size)?;
        Ok(())
    }
}

pub struct WAV {
    pub riff_header: RIFFHeader,
    pub fmt_header: FMTHeader,
    pub data_header: DataHeader,
    pub signal: Vec<i16>, //TODO make genaric
}

impl WAV {

    pub fn from_file(
        filename: &str,
    ) -> Result<WAV, Box<dyn Error>> {
        let mut f = File::open(filename)?;

        let mut buf = [0u8; 12];
        f.read(&mut buf)?;
        let riff_header = RIFFHeader::new(&buf)?;

        let mut buf = [0u8; 24];
        f.read(&mut buf)?;
        let fmt_header = FMTHeader::new(&buf)?;

        let mut buf = [0u8; 8];
        f.read(&mut buf)?;
        let data_header = DataHeader::new(&buf)?;

        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let signal: Vec<i16> = buf
            .chunks(2)
            .map(|x| i16::from_le_bytes([x[0], x[1]]))
            .collect();

        let wav = WAV {
            riff_header,
            fmt_header,
            data_header,
            signal,
        };

        Ok(wav)
    }

    pub fn write(self, filename: &str) -> Result<(), Box<dyn Error>> {
        let f = File::create(filename)?;
        let mut writer = BufWriter::new(f);
        self.riff_header.write(&mut writer)?;
        self.fmt_header.write(&mut writer)?;
        self.data_header.write(&mut writer)?;
        for x in self.signal {
            writer.write_i16::<LittleEndian>(x)?
        }

        Ok(())
    }
}

#[cfg(test)]
mod there_and_back_again {
    use super::WAV;
    use std::fs::{File, remove_file};
    use std::io::Read;

    #[test]
    fn lossless_read_write_1khz_file() {
        let wav = WAV::from_file("data/1kHz_44100Hz_16bit_05sec.wav").unwrap();
        wav.write("data/copy_1kHz.wav").unwrap();

        let mut input_file = File::open("data/1kHz_44100Hz_16bit_05sec.wav").unwrap();
        let mut input = Vec::new();
        input_file.read_to_end(&mut input).unwrap();

        let mut output_file = File::open("data/copy_1kHz.wav").unwrap();
        let mut output = Vec::new();
        output_file.read_to_end(&mut output).unwrap();
        assert_eq!(input, output);

        remove_file("data/copy_1kHz.wav").unwrap();
    }
}
