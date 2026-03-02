use dectalk;

use std::env;
use std::path::Path;

use dectalk::TTS_FORCE;
use dectalk::WAVE_FORMAT_1M16;

fn main() {
    println!("DECTalk Version: {}", dectalk::text_to_speech_version());

    let args: Vec<String> = env::args().collect();

    let mut tts_handle: dectalk::TTSHandle = dectalk::TTSHandle::new();

    tts_handle
        .startup(0, 0, Some(dt_callback))
        .expect("Failed to start DECTalk");

    tts_handle
        .open_wav_out_file(Path::new(&args[1]), WAVE_FORMAT_1M16)
        .expect("Failed to open output file");

    tts_handle
        .speak(&args[2], TTS_FORCE)
        .expect("Failed to queue speech");

    tts_handle
        .close_wav_out_file()
        .expect("Failed to close output file");

    tts_handle.shutdown().expect("Failed to shut down DECTalk");
}

extern "C" fn dt_callback(wparam: i64, lparam: i64, user_defined: u32, message: u32) {
    println!("DtCallback called");
    println!(
        "\tWPARAM: {}\n\tLPARAM: {}\n\tUser defined: {}\n\tMessage: {}",
        wparam, lparam, user_defined, message
    );
}
