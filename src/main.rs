use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample,
};
use raylib::prelude::*;
use wavers::{read, AudioConversion};

//  TODO:- implement fft from spec
//  TODO:- give final output some flair

const SCREEN_HEIGHT: i32 = 900;
const FPS: u32 = 60;
// const FONT_PATH: &str = "assets/fonts/FiraCode-SemiBold.ttf";
// const FONT_PATH: &str = "assets/fonts/SF-Pro.ttf";
const FONT_PATH: &str = "assets/fonts/SFMono-Medium.otf";
const FILE_PATH: &str = "assets/audio/song.wav";
const FONT_SIZE: f32 = 16.0;
const FONT_OFFSET: f32 = 1.0;
const OFFSET: i32 = 12;
const TIME_BAR_HEIGHT: i32 = 6;
const SAMPLE_RATE: usize = 44100;
const DATA_SIZE: usize = SAMPLE_RATE / FPS as usize;
const SCREEN_WIDTH: i32 = (DATA_SIZE * 2) as i32;

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Music Visualizer")
        .msaa_4x()
        .vsync()
        .build();

    rl.set_target_fps(FPS);
    let font = rl.load_font(&thread, FONT_PATH).unwrap();

    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find output device");

    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::I8 => {
            run::<i8>(&device, &config.into(), font, rl, thread)?
        }
        cpal::SampleFormat::I16 => run::<i16>(
            &device,
            &config.into(),
            font,
            rl,
            thread,
        )?,
        cpal::SampleFormat::I32 => run::<i32>(
            &device,
            &config.into(),
            font,
            rl,
            thread,
        )?,
        cpal::SampleFormat::I64 => run::<i64>(
            &device,
            &config.into(),
            font,
            rl,
            thread,
        )?,
        cpal::SampleFormat::U8 => {
            run::<u8>(&device, &config.into(), font, rl, thread)?
        }
        cpal::SampleFormat::U16 => run::<u16>(
            &device,
            &config.into(),
            font,
            rl,
            thread,
        )?,
        cpal::SampleFormat::U32 => run::<u32>(
            &device,
            &config.into(),
            font,
            rl,
            thread,
        )?,
        cpal::SampleFormat::U64 => run::<u64>(
            &device,
            &config.into(),
            font,
            rl,
            thread,
        )?,
        cpal::SampleFormat::F32 => run::<f32>(
            &device,
            &config.into(),
            font,
            rl,
            thread,
        )?,
        cpal::SampleFormat::F64 => run::<f64>(
            &device,
            &config.into(),
            font,
            rl,
            thread,
        )?,
        sample_format => {
            panic!("Unsupported sample format '{sample_format}'")
        }
    };

    Ok(())
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    font: Font,
    mut rl: RaylibHandle,
    thread: RaylibThread,
) -> color_eyre::eyre::Result<()>
where
    T: SizedSample + FromSample<f32>,
{
    let input_file_path = std::path::Path::new(FILE_PATH);
    let file = read(input_file_path, None)?;
    let file_size = file.len();
    let mut data = Box::new(file).leak().iter();

    // let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let total_time =
        file_size as f32 / (SAMPLE_RATE * channels) as f32;
    dbg!(total_time);

    let buffer = Arc::new(Mutex::new([0.0f32; DATA_SIZE]));
    let buf = buffer.clone();

    let mut next_value = move || data.next().unwrap().as_f32();

    let err_fn =
        |err| eprintln!("an error occured on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |output_buffer: &mut [T],
              _: &cpal::OutputCallbackInfo| {
            write_data(
                output_buffer,
                channels,
                &mut next_value,
                buffer.clone(),
            )
        },
        err_fn,
        None,
    )?;

    stream.play()?;
    let instant = Instant::now();

    while !rl.window_should_close() {
        let mut buffer = [0.0; DATA_SIZE];
        buf.lock()
            .unwrap()
            .swap_with_slice(buffer.as_mut_slice());

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::DARKGRAY);
        let offset =
            FONT_SIZE as i32 + 3 * OFFSET + TIME_BAR_HEIGHT;

        // let line_height = SCREEN_HEIGHT / 2 - offset;
        // d.draw_line(
        //     0,
        //     line_height,
        //     SCREEN_WIDTH,
        //     line_height,
        //     Color::GOLD,
        // );
        draw_timebar(
            &mut d,
            &font,
            instant.elapsed(),
            total_time,
        );
        draw_recs(
            &mut d,
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT - offset,
        );
    }

    Ok(())
}

fn draw_timebar(
    d: &mut RaylibDrawHandle,
    monospace_font: &Font,
    music_time_played: Duration,
    total_music_length: f32,
) {
    let music_time_played = music_time_played.as_secs_f32();

    d.draw_text_ex(
        monospace_font,
        formatted_text(
            music_time_played,
            total_music_length,
            FILE_PATH.split('/').last().unwrap(),
        )
        .as_str(),
        Vector2::new(
            0.0 + OFFSET as f32,
            (SCREEN_HEIGHT - OFFSET) as f32 - FONT_SIZE,
        ),
        FONT_SIZE,
        FONT_OFFSET,
        Color::LIGHTGRAY,
    );
    d.draw_rectangle(
        OFFSET,
        SCREEN_HEIGHT - 2 * OFFSET - FONT_SIZE as i32,
        SCREEN_WIDTH - 2 * OFFSET,
        TIME_BAR_HEIGHT,
        Color::RAYWHITE,
    );

    d.draw_rectangle(
        // x
        OFFSET,
        // y
        SCREEN_HEIGHT - 2 * OFFSET - FONT_SIZE as i32,
        // width
        ((SCREEN_WIDTH as f32 - 2.0 * OFFSET as f32)
            * music_time_played
            / total_music_length) as i32,
        // height
        TIME_BAR_HEIGHT,
        Color::ORANGE,
    );
}

fn draw_recs(
    d: &mut RaylibDrawHandle,
    data: [f32; DATA_SIZE],
    screen_width: i32,
    screen_height: i32,
) {
    let no_of_samples = DATA_SIZE;
    let width = screen_width / (no_of_samples) as i32;
    for (i, value) in data.iter().enumerate() {
        let height = i32::abs(
            (value * screen_height as f32 / 2.0) as i32 * 2,
        );
        let x = width * i as i32;
        let y = if *value < 0.0 {
            (screen_height / 2) - height
        } else {
            screen_height / 2
        };
        d.draw_rectangle(x, y, width, height, Color::GOLD);
    }
}

fn write_data<T>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> f32,
    buf: Arc<Mutex<[f32; DATA_SIZE]>>,
) where
    T: SizedSample + FromSample<f32>,
{
    let mut values = [0.0; DATA_SIZE];
    let mut c = 0;
    for frame in output.chunks_mut(channels) {
        for sample in frame.iter_mut() {
            let next_sample = next_sample();
            if c < values.len() {
                values[c] = next_sample;
                c += 1;
            }
            let value: T = T::from_sample(next_sample);
            *sample = value;
        }

        if c >= DATA_SIZE {
            buf.lock().unwrap().swap_with_slice(&mut values);
            c = 0;
        }
    }
}

fn formatted_text(
    music_time_played: f32,
    total_music_length: f32,
    music_name: &str,
) -> String {
    let (m_played, s_played) = s_to_m_and_s(music_time_played);
    let (m_total, s_total) = s_to_m_and_s(total_music_length);
    format!(
        "{:02.0}:{:02.0}/{:02.0}:{:02.0} | Playing - {:>10}",
        m_played, s_played, m_total, s_total, music_name
    )
}

fn s_to_m_and_s(s: f32) -> (f32, f32) {
    (s / 60.0, s % 60.0)
}

// fn playback(
//     mut d: RaylibDrawHandle,
//     data: [f32; DATA_SIZE],
// ) {

//         // Pause/Resume the song
//         if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
//             if sink.is_paused() {
//                 sink.play();
//                 now = Instant::now();
//                 // .checked_add(timestamp)
//                 // .unwrap();
//             } else {
//                 sink.pause();
//                 timestamp += now.elapsed();
//                 music_time_played = timestamp.as_secs_f32();
//             }
//         }

//         // Restart the song
//         if rl.is_key_pressed(KeyboardKey::KEY_R) {
//             sink.stop();
//             let mut f = file.try_clone().unwrap();
//             f.seek(std::io::SeekFrom::Start(0)).unwrap();
//             sink.append(
//                 rodio::Decoder::new(BufReader::new(f)).unwrap(),
//             );
//             sink.play();
//             now = Instant::now();
//             timestamp = Duration::ZERO;
//             sink.set_volume(music_volume);
//         }

//         // Increase the volume
//         if rl.is_key_pressed(KeyboardKey::KEY_UP) {
//             music_volume = f32::min(music_volume + 0.05, 1.0);
//             sink.set_volume(music_volume);
//         }

//         // Decrease the volume
//         if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
//             music_volume = f32::max(music_volume - 0.05, 0.0);
//             sink.set_volume(music_volume);
//         }

//         if !sink.is_paused() {
//             music_time_played = now.elapsed().as_secs_f32()
//                 + timestamp.as_secs_f32();
//         }

// }
