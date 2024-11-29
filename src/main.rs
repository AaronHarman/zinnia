use std::process::Command;
use std::io;
use std::io::Write;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;

use cpal::Data;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};


fn main() {

    let host = cpal::default_host();
    let device = host.default_input_device().expect("no input device available");
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let stream = device.build_input_stream(
        &supported_config.into(),
        move |data: & [f32], _: &cpal::InputCallbackInfo| {
            // react to stream events and read or write stream data here.
            println!("Length: {}", data.len());
        },
        move |err| {
            // react to errors here.
        },
        None // None=blocking, Some(Duration)=timeout
    );

    // make a channel for sending messages to be spoken to the talk thread
    let (send, recv) = mpsc::channel::<&str>();

    // make a thread to handle talking, and give it the receiver end of the channel
    let talk_thread = thread::spawn(move || {
        for thing in recv {
            say(thing.to_string());
        }
    });

    send.send("Zinnia here!");
    send.send("There are very few good reasons to skin a cat, but according to popular idioms there are quite a few methods to do so if you find you must.");

    drop(send);
    talk_thread.join().expect("Error joining the talk thread");
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
