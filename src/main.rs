use std::{error::Error};

use cldj::input::wav;
use cldj::display;


fn main() -> Result<(), Box<dyn Error>> {
    let (_, format_header, _, mut signal) = wav::read_file("data/100Hz_44100Hz_16bit_05sec.wav")?;
    //let (_, format_header, _, mut signal) = wav::read_file("data/1kHz_44100Hz_16bit_05sec.wav");

    let sample_rate = format_header.sample_rate as usize;
    let fourier_output_length = sample_rate / 10;
//    println!("sample rate: {}", sample_rate);

    //100Hz frequency
    //44100Hz sampling frequency
    //44100Hz / 100Hz = 441 oversampling factor
    //5 seconds
    //100 * 441 * 5 = 220500 samples
//    println!("n samples: {}", signal.len());

    let head = signal.drain(..fourier_output_length).collect::<Vec<i16>>();
//    let result = fourier_transform(head);
//
//    //2 ** 12 = 4096
//    //44100Hz / 4096 = 10.7666 
//    //100 / 10.7666 = 9.28798
//    // => spike at 9.28798
//    //
//    // 44100Hz sampling rate / 4410 vector length => ith position = i * 10Hz frequency
//    // therefore we expect a spike at the 10th index
//
//
//    let mut foo: Vec<f64> = Vec::new();
//    //println!("{:#?}", result);
//    for x in result {
//        foo.push(x.norm());
//        //println!("{}", x.norm())
//    }
//    let half = foo.drain(..150).collect::<Vec<f64>>();
//    for (i, x) in half.iter().enumerate() {
//        println!("{}: {}", i, x);
//    }
    //display::run(foo)?;
    display::run(head)?;
    Ok(())
}

