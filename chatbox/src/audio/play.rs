use crate::config;
use anyhow::{Context, Result};
use bytes::Bytes;
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::io::{Cursor, Read, Seek};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use log::debug;

static PLAYING_INDEX: AtomicU64 = AtomicU64::new(0);

pub fn output_devices_name() -> Vec<String> {
    let mut names = vec![];
    let host = cpal::default_host();
    match host.output_devices() {
        Ok(devices) => {
            for device in devices {
                if let Ok(name) = device.name() {
                    if name.starts_with("default") {
                        names.push(name);
                    }
                }
            }
        }
        Err(e) => println!("{:?}", e),
    }

    names
}

pub fn audio_local(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    play_audio(BufReader::new(file))
}

pub fn audio_memory(source: &Bytes) -> Result<(), Box<dyn std::error::Error>> {
    let cursor = Cursor::new(source.to_vec());
    play_audio(cursor)
}

pub fn stop_play() {
    PLAYING_INDEX.fetch_add(1, Ordering::SeqCst);
}

fn play_audio<R>(source: R) -> Result<(), Box<dyn std::error::Error>>
where
    R: Read + Seek + Send + Sync + 'static,
{
    let playing_index = PLAYING_INDEX.fetch_add(1, Ordering::SeqCst);
    let audio_config = config::audio();
    let source = Decoder::new(source)?;

    let (_stream, handle) = if audio_config.current_output_device == "default" {
        OutputStream::try_default()?
    } else {
        let host = cpal::default_host();
        let device = host
            .output_devices()?
            .find(|x| {
                x.name()
                    .map(|y| y == audio_config.current_output_device)
                    .unwrap_or(false)
            })
            .with_context(|| "failed to find output device")?;
        OutputStream::try_from_device(&device)?
    };

    let sink = Sink::try_new(&handle)?;
    sink.append(source);
    sink.set_volume(audio_config.output_volume);
    sink.play();

    while playing_index + 1 == PLAYING_INDEX.load(Ordering::SeqCst) {
        if sink.empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    sink.stop();

    debug!("audio exit...");
    Ok(())
}
