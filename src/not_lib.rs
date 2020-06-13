use std::convert::TryInto;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::mem;
use std::str;

use std::f64::consts::PI;

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Terminal,
};

use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    ignore_exit_key: Arc<AtomicBool>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            //tick_rate: Duration::from_millis(250),
            tick_rate: Duration::from_millis(100),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                        if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                            return;
                        }
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || loop {
                tx.send(Event::Tick).unwrap();
                thread::sleep(config.tick_rate);
            })
        };
        Events {
            rx,
            ignore_exit_key,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed);
    }
}

#[derive(Debug)]
struct RIFFHeader {
    riff: String,
    file_size: u32,
    four_cc: String,
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
struct FMTHeader {
    fmt: String,
    header_size: u32,
    format: u16,
    nchannels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
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
struct DataHeader {
    data: String,
    size: u32,
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

//TODO calculate and return complex part too
fn calculate_kth_nth(x_n: &i16, n: usize, n_samples: usize, k: usize) -> f64 {
    let x_n = *x_n as f64;
    let n = n as f64;
    let n_samples = n_samples as f64;
    let k = k as f64;
    let inner = 2.0 * PI * k * n / n_samples;
    //println!("inner: {}", inner.cos());
    //println!("x_n: {}", x_n);
    x_n * inner.cos()
}

fn calculate_kth(k: usize, samples: &Vec<i16>) -> f64 {
    let mut x_k: f64 = 0.0;
    let n_samples = samples.len();
    for (n, x_n) in samples.iter().enumerate() {
        let tmp = calculate_kth_nth(x_n, n, n_samples, k);
        //println!("tmp: {}", tmp);
        x_k += tmp;
    }
    x_k / n_samples as f64
}

//TODO write a few tests for this
pub fn finite_fourier_transform(samples: Vec<i16>) -> Vec<f64> {
    let mut transformed_samples: Vec<f64> = Vec::new();
    let n_samples = samples.len();
    for k in 0..n_samples {
        let x_k = calculate_kth(k, &samples);
        transformed_samples.push(x_k);
    }
    transformed_samples
}

#[cfg(test)]
mod fft_test {
    use super::finite_fourier_transform;

    #[test]
    fn impulse_at_origin() {
        let zeros: Vec<i16> = vec![1, 0, 0, 0, 0, 0, 0, 0];
        let expected: Vec<f64> = vec![0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125];
        let result = finite_fourier_transform(zeros);
        assert_eq!(expected, result);
    }

    #[test]
    fn impulse_at_one() {
        let zeros: Vec<i16> = vec![0, 1, 0, 0, 0, 0, 0, 0];
        let expected: Vec<f64> = vec![0.125, 0.088, 0.000, -0.088, -0.125, -0.088, 0.000, 0.088];
        let mut result = finite_fourier_transform(zeros);
        result = result
            .iter()
            .map(|x| (x * 1000.0).round() / 1000.0)
            .collect();
        assert_eq!(expected, result);
    }
}

struct App {
    signal: Vec<(f64, f64)>,
    data: Vec<(f64, f64)>,
    window: [f64; 2],
}

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let signal_head = &self.signal[..2];
        let signal_tail = &self.signal[(self.signal.len()-2)..];
        let data_head = &self.data[..2];
        let data_tail = &self.data[(self.data.len()-2)..];
        write!(
            f,
            "App:\n  signal_head: {:?} ... {:?},\n  data_head: {:?} ... {:?},\n  window: {:?}",
            signal_head, signal_tail, data_head, data_tail, self.window
        )
    }
}

impl App {
    fn new(mut data: Vec<i16>) -> App {
        let mut signal: Vec<(f64, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f64, *x as f64))
            .collect();
        let data = signal.drain(..200).collect::<Vec<(f64, f64)>>();
        App {
            signal,
            data,
            window: [0.0, 20.0],
        }
    }

    fn update(&mut self) {
        for _ in 0..5 {
            self.data.remove(0);
        }
        self.data.extend(self.signal.drain(..5));
        self.window[0] += 5.0;
        self.window[1] += 5.0;
    }
}

fn read_file() -> (RIFFHeader, FMTHeader, DataHeader, Vec<i16>) {
    //let mut f = File::open(file).unwrap();
    let mut f = File::open("1kHz_44100Hz_16bit_05sec.wav").unwrap();

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

    (riff_h, fmt_h, data_h, signal)
}
