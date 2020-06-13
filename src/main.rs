use std::{error::Error};

use cldj::input::wav;
use cldj::display;

fn main() -> Result<(), Box<dyn Error>> {
    let (_, _, _, signal) = wav::read_file("data/1kHz_44100Hz_16bit_05sec.wav");
    display::run(signal)?;
    Ok(())
}

