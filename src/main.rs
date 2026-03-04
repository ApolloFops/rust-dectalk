use dectalk;

use std::env;
use std::path::Path;

fn main() {
    println!("DECTalk Version: {}", dectalk::text_to_speech_version());

    let args: Vec<String> = env::args().collect();

    let mut tts_handle: dectalk::TTSHandle = dectalk::TTSHandle::new();

    tts_handle.startup(0, 0).expect("Failed to start DECTalk");

    tts_handle
        .open_wav_out_file(Path::new(&args[1]), dectalk::DtTTSFormat::WaveFormat1M16)
        .expect("Failed to open output file");

    tts_handle
        .speak(&args[2], dectalk::DtTTSFlags::Force)
        .expect("Failed to queue speech");

    tts_handle
        .close_wav_out_file()
        .expect("Failed to close output file");

    tts_handle.shutdown().expect("Failed to shut down DECTalk");
}
