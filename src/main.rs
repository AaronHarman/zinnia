use std::process::Command;
use std::io;
use std::io::Write;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
use std::collections::VecDeque;

use cpal::Data;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use rustpotter::{Rustpotter, RustpotterConfig};


fn main() {
    // initialize rustpotter for wakeword detection
    let mut rp_config = RustpotterConfig::default();
    rp_config.detector.threshold = 0.20;
    rp_config.detector.avg_threshold = 0.15;
    rp_config.filters = rustpotter::FiltersConfig {
        gain_normalizer: rustpotter::GainNormalizationConfig {
            enabled: true,
            gain_ref: None,
            min_gain: 0.5,
            max_gain: 1.0,
        },
        band_pass: rustpotter::BandPassConfig {
            enabled: true,
            low_cutoff: 80.0,
            high_cutoff: 400.0,
        },
    };
    println!("config: {:#?}", rp_config);
    let mut rp = Rustpotter::new(&rp_config).unwrap();
    // load the wakeword file
    rp.add_wakeword_from_file("Yo Zinnia", "./resources/Yo_Zinnia2.rpw").unwrap();
    let mut samples_buffer: VecDeque<f32> = VecDeque::new();
    let rp_buffer_size = rp.get_samples_per_frame();

    // input device stuff
    let host = cpal::default_host();
    let in_device = host.default_input_device().expect("no input device available");
    println!("Input device: {}", in_device.name().unwrap());
    let mut supported_configs_range = in_device.supported_input_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.filter(|x| x.sample_format() == cpal::SampleFormat::F32)
        .next()
        .expect("no supported config?!")
        .with_sample_rate(cpal::SampleRate(16000));
    println!("input config: {:#?}", supported_config);
    let in_stream = in_device.build_input_stream(
        &supported_config.into(),
        move |data: & [f32], _: &cpal::InputCallbackInfo| {
            // react to stream events and read or write stream data here.
            //println!("Input Length: {}", data.len());
            //println!("{:#?}", data);
            let mut data_vec = data.to_vec().into();
            samples_buffer.append(&mut data_vec);
            while false && samples_buffer.len() >= rp_buffer_size {
                //println!("Used up some of the buffer :) Remaining buffer: {}", samples_buffer.len()-rp_buffer_size);
                let detection = rp.process_samples(samples_buffer.drain(..rp_buffer_size).collect());
                if let Some(detection) = detection {
                    println!("Detected: {:?}", detection);
                }
            }
        },
        move |err| {
            // react to errors here.
            eprintln!("Error with audio input stream: {}", err);
        },
        None // None=blocking, Some(Duration)=timeout
    );
    
    // make a channel for sending messages to be spoken to the talk thread
    let (send, recv) = mpsc::channel::<&str>();

    // make a thread to handle talking, and give it the receiver end of the channel
    let talk_thread = thread::spawn(move || {
        for thing in recv {
            match say(thing.to_string()) {
                Ok(_) => {},
                Err(e) => {eprintln!("Error with speech synthesis: {}", e);}
            }
        }
    });

    let _ = send.send("Zinnia here!");
    let _ = send.send("There are very few good reasons to skin a cat, but according to popular idioms there are quite a few methods to do so if you find you must.");

    
    drop(send);
    talk_thread.join().expect("Error joining the talk thread");
    while true {}
    
}

// use Piper (for speech synthesis) and aplay (audio output) to output speech
fn say(text : String) -> io::Result<u8> {
    // open piper, and pass it some settings
    let piper = Command::new("piper/piper")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("--model")
        .arg("piper/libritts_r/en_US-libritts_r-medium.onnx")
        .arg("--speaker")
        .arg("45")
        .arg("--length_scale")
        .arg("1.2")
        .arg("--output-raw")
        .spawn()?;

    // send the message to piper
    piper.stdin.as_ref().unwrap().write_all(text.as_bytes())?;
    // wait for piper to finish synthesizing and grab the audio output
    let stream = piper.wait_with_output()?;

    
    // open aplay, and set it to the correct audio format
    let mut child = Command::new("aplay")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["-r","22050","-f","S16_LE","-t","raw"])
        .spawn()?;

    // send the audio to aplay to play
    child.stdin.as_ref().unwrap().write_all(stream.stdout.as_slice())?;
    // wait for aplay to finish
    child.wait()?;

    return Ok(0);
}
