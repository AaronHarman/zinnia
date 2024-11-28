use std::process::Command;
use std::io;
use std::io::Write;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;


fn main() {
    let (send, recv) = mpsc::channel::<&str>();

    let talk_thread = thread::spawn(move || {
        for thing in recv {
            say(thing.to_string());
        }
    });

    send.send("Zinnia here!");
    send.send("There are very few good reasons to skin a cat, but according to popular idioms there are quite a few methods to do so if you find you must.");

    drop(send);
    talk_thread.join().expect("Error joining the talk thread");
    println!("test");
}

fn say(text : String) -> io::Result<u8> {
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

    piper.stdin.as_ref().unwrap().write_all(text.as_bytes())?;

    let stream = piper.wait_with_output()?;

    let mut child = Command::new("aplay")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["-r","22050","-f","S16_LE","-t","raw"])
        .spawn()?;

    child.stdin.as_ref().unwrap().write_all(stream.stdout.as_slice())?;

    child.wait()?;

    return Ok(0);
}
