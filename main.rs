use std::{
    f32::consts::PI,
    fs::File,
    io::{BufReader, Cursor},
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::sleep,
    time::Duration,
};

use anyhow::{self};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample,
};
use wavers::Sample as WSample;
use wavers::{read, AudioConversion};

// const FILE_PATH: &str = "05 Set Fire to the Rain.wav";
// const FILE_PATH: &str = "two_channel.wav";
const FILE_PATH: &str = "output.wav";

fn main() -> anyhow::Result<()> {

    Ok(())
}
