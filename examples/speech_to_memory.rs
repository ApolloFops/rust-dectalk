use dectalk;

use std::env;
use std::fs::OpenOptions;
use std::io::Write;

use dectalk::TTS_FORCE;
use dectalk::WAVE_FORMAT_1M16;

#[tokio::main]
async fn main() {
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
        .speak("Testing dectalk speech-to-memory from rust!", TTS_FORCE)
        .expect("Failed to queue speech");

    tts_handle
        .speak("Speaking a second time", TTS_FORCE)
        .expect("Failed to queue speech");

    tts_handle
        .speak("Speaking once again, now three!", TTS_FORCE)
        .expect("Failed to queue speech");

    // while (true) {}

    tts_handle
        .close_in_memory()
        .expect("Failed to close in memory");

    tts_handle.shutdown().expect("Failed to shut down DECTalk");

    dbg!(&tts_handle.output_buffers);

    // Write data to a file
    for (key, value) in tts_handle.output_buffers {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(format!("output{}.wav", key))
            .expect("Failed to open file");

        file.write_all(value.output_data.as_slice())
            .expect("Failed to write data to output file");
    }
}
