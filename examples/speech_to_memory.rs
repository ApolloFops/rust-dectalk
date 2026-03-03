use dectalk;

use std::env;

use dectalk::TTS_FORCE;
use dectalk::WAVE_FORMAT_1M16;

fn main() {
    println!("DECTalk Version: {}", dectalk::text_to_speech_version());

    let args: Vec<String> = env::args().collect();

    let mut tts_handle: dectalk::TTSHandle = dectalk::TTSHandle::new();

    tts_handle.startup(0, 0).expect("Failed to start DECTalk");

    tts_handle
        .open_in_memory(WAVE_FORMAT_1M16)
        .expect("Failed to open in memory");

    tts_handle
        .create_buffer(4096, 128)
        .expect("Failed to create buffer");

    tts_handle
        .speak("[:index mark 1]Testing dectalk[:index mark 2]", TTS_FORCE)
        .expect("Failed to queue speech");

    while (true) {}

    tts_handle
        .close_in_memory()
        .expect("Failed to close in memory");

    tts_handle.shutdown().expect("Failed to shut down DECTalk");
}
