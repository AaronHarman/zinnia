use std::process::Command;
use std::io::Write;
use std::process::Stdio;


fn main() {
    let piper = Command::new("piper/piper")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("--model")
        .arg("piper/libritts_r/en_US-libritts_r-medium.onnx")
        .arg("--speaker")
        .arg("25")
        .arg("--output-raw")
        .spawn()
        .expect("Failed to get output from piper");

    piper.stdin.as_ref().unwrap().write_all("Take me out to the ball game".as_bytes()).expect("couldn't write to piper's stdin");

    let stream = piper.wait_with_output().expect("couldn't wait on piper");

    let mut child = Command::new("aplay")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["-r","22050","-f","S16_LE","-t","raw"])
        .spawn()
        .expect("Failed to spawn aplay");

    child.stdin.as_ref().unwrap().write_all(stream.stdout.as_slice()).expect("couldn't write to aplay's stdin");

    child.wait().expect("yikes");

}
