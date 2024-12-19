use std::process::Command;
use std::io;
use std::io::{Write, Cursor};
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
use std::collections::VecDeque;

use cpal::traits::{DeviceTrait, HostTrait};

use rustpotter::{Rustpotter, RustpotterConfig, SampleFormat};

use tray_item::{IconSource, TrayItem};

use vosk::{Model, Recognizer, DecodingState};

// Messages to be sent to the speech thread
enum SpeakMessage<'a> {
    Say(&'a str),
}

// Messages to be sent from the tray icon to the main program
enum TrayMessage {
    Close,
}

// State of the overall program
enum State {
    Waiting,
    Listening,
    CommandRunning,
}

fn main() {
    // state stuff
    let mut state = State::Waiting;

    // initialize rustpotter for wakeword detection
    let mut rp = rustpotter_init(SampleFormat::I16, 16000, "./resources/Yo_Zinnia2.rpw").unwrap();

    // set up a buffer for feeding samples to Rustpotter
    let mut samples_buffer: VecDeque<i16> = VecDeque::new();
    let rp_buffer_size = rp.get_samples_per_frame();

    // set up vosk for speech recognition (it's short enough that it didn't get its own function)
    let vosk_model = Model::new("resources/vosk-model-small-en-us-0.15").unwrap();
    let mut recog = Recognizer::new(&vosk_model, 16000.0).unwrap();

    // make a channel for sending messages to be spoken to the talk thread
    let (send, recv) = mpsc::channel::<SpeakMessage>();
    let in_send = send.clone(); // make another input for the in_stream to use

    // input device stuff
    let host = cpal::default_host();
    let in_device = host.default_input_device().expect("no input device available");
    println!("Input device: {}", in_device.name().unwrap());
    let supported_configs_range = in_device.supported_input_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.filter(|x| x.sample_format() == cpal::SampleFormat::I16 && x.channels() == 2)
        .next()
        .expect("no supported config?!")
        .with_sample_rate(cpal::SampleRate(16000));
    println!("input config: {:#?}", supported_config);
    let in_stream = in_device.build_input_stream(
        &supported_config.into(),
        move |data: & [i16], _: &cpal::InputCallbackInfo| {
            // react to stream events and read or write stream data here.
            match state {
                State::Waiting => {
                    let mut data_vec = data.to_vec().into();
                    samples_buffer.append(&mut data_vec);
                    while samples_buffer.len() >= rp_buffer_size {
                        //println!("Used up some of the buffer :) Remaining buffer: {}", samples_buffer.len()-rp_buffer_size);
                        let detection = rp.process_samples(samples_buffer.drain(..rp_buffer_size).collect());
                        if let Some(detection) = detection {
                            println!("Detected: {:?}", detection);
                            let _ = in_send.send(SpeakMessage::Say("Zinnia here!"));
                            state = State::Listening;
                        }
                    }
                },
                State::Listening => {
                    let decoding_state = recog.accept_waveform(data).unwrap();
                    if decoding_state == DecodingState::Finalized {
                        let vosk::CompleteResult::Single(single_result) = recog.final_result() else { todo!() };
                        println!("Heard: \"{}\"", single_result.text);
                        //let result = recog.final_result();
                        //println!("{:#?}", result);
                        recog.reset();
                        //state = State::Waiting;
                    }
                    if decoding_state == DecodingState::Failed {
                        eprintln!("Something broke with decoding the audio in Vosk");
                    }
                },
                State::CommandRunning => {}, // doesn't do anything, just waits
            }
        },
        move |err| {
            // react to errors here.
            eprintln!("Error with audio input stream: {}", err);
        },
        None // None=blocking, Some(Duration)=timeout
    );
    // This is supposed to be here but it actually makes it not work for some reason so uhh yeah
    //match in_stream {
    //    Ok(is) => {
    //        match is.play() { // it's possible the stream won't start automatically so this makes sure it does
    //            Ok(_) => {},
    //            Err(e) => {eprintln!("Error starting audio stream: {}", e);}
    //        }
    //    },
    //    Err(e) => { eprintln!("Error making audio input stream: {}", e); }
    //}
    
    
    // make a thread to handle talking, and give it the receiver end of the channel
    let talk_thread = thread::spawn(move || {
        for message in recv {
            let SpeakMessage::Say(thing) = message;
            match say(thing.to_string()) {
                Ok(_) => {},
                Err(e) => {eprintln!("Error with speech synthesis: {}", e);}
            }
        }
    });

    //let _ = send.send(SpeakMessage::Say("Zinnia here!"));
    //let _ = send.send(SpeakMessage::Say("There are very few good reasons to skin a cat, but according to popular idioms there are quite a few methods to do so if you find you must."));

    // get the icon to use in the tray
    let img_decoder = png::Decoder::new(Cursor::new(include_bytes!("../resources/1f444.png")));
    let (img_info, mut img_reader) = img_decoder.read_info().unwrap();
    let mut img_buf = vec![0; img_info.buffer_size()];
    img_reader.next_frame(&mut img_buf).unwrap();
    let icon = IconSource::Data {
        data: img_buf,
        height: 72,
        width: 72,
    };
    // set up the tray menu
    let mut tray = TrayItem::new("Zinnia", icon).unwrap();
    tray.add_label("ZINNIA").unwrap();
    let (tray_tx, tray_rx) = mpsc::sync_channel::<TrayMessage>(2);
    let tray_quit_tx = tray_tx.clone();
    let id_menu = tray.inner_mut()
        .add_menu_item_with_id("Close Zinnia", move || {
            tray_quit_tx.send(TrayMessage::Close).unwrap();
        }).unwrap();

    println!("TEST: Made it to the tray message watch loop");
    // watch for tray messages
    loop {
        match tray_rx.recv() {
            Ok(TrayMessage::Close) => {
                break;
            }
            Err(e) => {
                eprintln!("Error with the tray menu: {}", e);
            }
        }
    }
    
    // gotta drop these two first so all the inputs to the speaking channel are closed
    drop(send);
    drop(in_stream);
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

// initializes Rustpotter using settings passed in, plus some baked in ones that I don't expect to change'
fn rustpotter_init(format : SampleFormat, sample_rate : u16, wwpath : &str) -> Result<Rustpotter, &'static str> {
    let mut rp_config = RustpotterConfig::default();
    rp_config.detector.threshold = 0.42;
    rp_config.detector.avg_threshold = 0.23;
    rp_config.filters = rustpotter::FiltersConfig {
        gain_normalizer: rustpotter::GainNormalizationConfig {
            enabled: true,
            gain_ref: None,
            min_gain: 0.1,
            max_gain: 1.0,
        },
        band_pass: rustpotter::BandPassConfig {
            enabled: false,
            low_cutoff: 80.0,
            high_cutoff: 400.0,
        },
    };
    rp_config.fmt = rustpotter::AudioFmt {
        sample_rate: sample_rate as usize,
        sample_format: format,
        channels: 2,
        endianness: rustpotter::Endianness::Little,
    };
    println!("config: {:#?}", rp_config);
    let mut rp = match Rustpotter::new(&rp_config) {
        Ok(x) => {x},
        Err(_) => {return Err("Failed to initialize Rustpotter")},
    };
    // load the wakeword file
    match rp.add_wakeword_from_file("Yo Zinnia", wwpath) {
        Ok(_) => {},
        Err(_) => {return Err("Failed to add wakeword from file")},
    };
    return Ok(rp);
}
