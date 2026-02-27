mod lib;
use lib as dectalk;

use std::path::Path;

use dectalk::LPTTS_HANDLE_T;
use dectalk::TTS_FORCE;
use dectalk::WAVE_FORMAT_1M16;

fn main() {
    let mut tts_handle_ptr: LPTTS_HANDLE_T = std::ptr::null_mut();

    println!("DECTalk Version: {}", dectalk::text_to_speech_version());

    dectalk::text_to_speech_startup(&mut tts_handle_ptr, 0, 0, Some(dt_callback), 0)
        .expect("Failed to start DECTalk");

    dectalk::text_to_speech_open_wave_out_file(
        tts_handle_ptr,
        Path::new("test.wav"),
        WAVE_FORMAT_1M16,
    )
    .expect("Failed to open output file");

    dectalk::text_to_speech_speak(tts_handle_ptr, "dectalk from rust!", TTS_FORCE)
        .expect("Failed to queue speech");

    dectalk::text_to_speech_close_wave_out_file(tts_handle_ptr)
        .expect("Failed to close output file");

    dectalk::text_to_speech_shutdown(tts_handle_ptr).expect("Failed to shut down DECTalk");
}

extern "C" fn dt_callback(wparam: i64, lparam: i64, user_defined: u32, message: u32) {
    println!("DtCallback called");
    println!(
        "\tWPARAM: {}\n\tLPARAM: {}\n\tUser defined: {}\n\tMessage: {}",
        wparam, lparam, user_defined, message
    );
}
