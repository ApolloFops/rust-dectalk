use dectalk;

use std::fs::OpenOptions;
use std::io::Write;

#[tokio::main]
async fn main() {
    println!("DECTalk Version: {}", dectalk::text_to_speech_version());

    let mut tts_handle: dectalk::TTSHandle = dectalk::TTSHandle::new();

    tts_handle.startup(0, 0).expect("Failed to start DECTalk");

    tts_handle
        .open_in_memory(dectalk::DtTTSFormat::WaveFormat1M16)
        .expect("Failed to open in memory");

    tts_handle
        .create_buffer(4096, 128)
        .expect("Failed to create buffer");

    write_to_file(
        0,
        tts_handle
            .speak(
                "Testing dectalk speech-to-memory from rust!",
                dectalk::DtTTSFlags::Force,
            )
            .expect("Failed to queue speech")
            .await,
    );

    println!("First buffer done");

    write_to_file(
        1,
        tts_handle
            .speak("Speaking a second time", dectalk::DtTTSFlags::Force)
            .expect("Failed to queue speech")
            .await,
    );

    println!("Second buffer done");

    write_to_file(
        2,
        tts_handle
            .speak(
                "Speaking once again, now three!",
                dectalk::DtTTSFlags::Force,
            )
            .expect("Failed to queue speech")
            .await,
    );

    println!("Third buffer done");

    tts_handle
        .close_in_memory()
        .expect("Failed to close in memory");

    tts_handle.shutdown().expect("Failed to shut down DECTalk");
}

fn write_to_file(index: u32, output_data: Vec<u8>) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("output{}.wav", index))
        .expect("Failed to open file");

    file.write_all(output_data.as_slice())
        .expect("Failed to write data to output file");
}
