use dectalk;

use std::env;
use std::ffi::CString;

use dectalk::TTS_FORCE;
use dectalk::WAVE_FORMAT_1M16;

fn main() {
    println!("DECTalk Version: {}", dectalk::text_to_speech_version());

    let args: Vec<String> = env::args().collect();

    let mut tts_handle: dectalk::TTSHandle = dectalk::TTSHandle::new();

    tts_handle.startup(0, 0).expect("Failed to start DECTalk");

    dbg!(&tts_handle);

    tts_handle
        .open_in_memory(WAVE_FORMAT_1M16)
        .expect("Failed to open in memory");

    let mut data_string;
    unsafe {
        data_string = CString::from_vec_with_nul_unchecked(vec![0; 4096]);
    }
    let buffer_length = data_string.count_bytes() as u32;

    // TODO: Sort out keeping this alive and then dropping it when done
    let mut index_vec: Vec<dectalk::TTS_INDEX_T> = Vec::with_capacity(128 as usize);

    let mut buffer: dectalk::TTS_BUFFER_T = dectalk::TTS_BUFFER_T {
        lpData: data_string.into_raw(),
        dwMaximumBufferLength: buffer_length,
        lpPhonemeArray: std::ptr::null_mut(),
        lpIndexArray: index_vec.as_mut_ptr(),
        dwMaximumNumberOfPhonemeChanges: 0,
        dwMaximumNumberOfIndexMarks: 128,
        dwBufferLength: 0,
        dwNumberOfPhonemeChanges: 0,
        dwNumberOfIndexMarks: 0,
        dwReserved: 0,
    };

    tts_handle
        .add_buffer(&mut buffer)
        .expect("Failed to add buffer");

    tts_handle
        .speak("[:index mark 1]Testing dectalk[:index mark 2]", TTS_FORCE)
        .expect("Failed to queue speech");

    while (true) {}

    tts_handle
        .close_in_memory()
        .expect("Failed to close in memory");

    tts_handle.shutdown().expect("Failed to shut down DECTalk");
}
