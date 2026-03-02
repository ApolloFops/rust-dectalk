#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::CStr;
use std::fmt;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// This may be a hacky workaround but IDK enough about rust to know if it is
// It does work though
#[link(name = "dectalk")]
unsafe extern "C" {}

#[derive(Debug, PartialEq)]
pub enum DtError {
    NoError,
    Error,
    BadDeviceID,
    NotEnabled,
    Allocated,
    InvalidHandle,
    NoDriver,
    NoMemory,
    NotSupported,
    BadErrorNumber,
    InvalidFlag,
    InvalidParameter,
    HandleBusy,
    InvalidAlias,
}

fn parse_result(v: MMRESULT) -> Result<DtError, DtError> {
    match v {
        MMSYSERR_NOERROR => Ok(DtError::NoError),
        MMSYSERR_ERROR => Err(DtError::Error),
        MMSYSERR_BADDEVICEID => Err(DtError::BadDeviceID),
        MMSYSERR_NOTENABLED => Err(DtError::NotEnabled),
        MMSYSERR_ALLOCATED => Err(DtError::Allocated),
        MMSYSERR_INVALHANDLE => Err(DtError::InvalidHandle),
        MMSYSERR_NODRIVER => Err(DtError::NoDriver),
        MMSYSERR_NOMEM => Err(DtError::NoMemory),
        MMSYSERR_NOTSUPPORTED => Err(DtError::NotSupported),
        MMSYSERR_BADERRNUM => Err(DtError::BadErrorNumber),
        MMSYSERR_INVALFLAG => Err(DtError::InvalidFlag),
        MMSYSERR_INVALPARAM => Err(DtError::InvalidParameter),
        MMSYSERR_HANDLEBUSY => Err(DtError::HandleBusy),
        MMSYSERR_INVALIDALIAS => Err(DtError::InvalidAlias),
        // Handle all valid cases
        _ => Err(DtError::BadErrorNumber),
    }
}

pub fn text_to_speech_version() -> u32 {
    let x = unsafe { TextToSpeechVersion(std::ptr::null_mut()) };
    return x;
}

pub fn text_to_speech_startup(
    tts_handle: *mut LPTTS_HANDLE_T,
    device_number: UINT,
    device_options: DWORD,
    callback_routine: Option<unsafe extern "C" fn(i64, i64, i64, u32)>,
    callback_parameter: LONG,
) -> Result<DtError, DtError> {
    dbg!(callback_parameter);

    unsafe {
        let status = TextToSpeechStartup(
            tts_handle,
            device_number,
            device_options,
            callback_routine,
            callback_parameter,
        );

        return parse_result(status);
    }
}

pub fn text_to_speech_shutdown(tts_handle: LPTTS_HANDLE_T) -> Result<DtError, DtError> {
    unsafe {
        let status = TextToSpeechShutdown(tts_handle);

        return parse_result(status);
    }
}

pub fn text_to_speech_speak(
    tts_handle: LPTTS_HANDLE_T,
    text: &str,
    flags: DWORD,
) -> Result<DtError, DtError> {
    unsafe {
        let status = TextToSpeechSpeak(
            tts_handle,
            String::from(text).as_mut_ptr() as *mut i8,
            flags,
        );

        return parse_result(status);
    }
}

pub fn text_to_speech_open_wave_out_file(
    tts_handle: LPTTS_HANDLE_T,
    file: &Path,
    audio_format: DWORD,
) -> Result<DtError, DtError> {
    unsafe {
        let mut filepath: String = String::from(
            std::path::absolute(file)
                .expect("Failed to find absolute path to file")
                .to_str()
                .expect("File path is not a valid string"),
        );

        let status =
            TextToSpeechOpenWaveOutFile(tts_handle, filepath.as_mut_ptr() as *mut i8, audio_format);

        return parse_result(status);
    }
}

pub fn text_to_speech_close_wave_out_file(tts_handle: LPTTS_HANDLE_T) -> Result<DtError, DtError> {
    unsafe {
        let status = TextToSpeechCloseWaveOutFile(tts_handle);

        return parse_result(status);
    }
}

pub fn text_to_speech_open_in_memory(
    tts_handle: LPTTS_HANDLE_T,
    audio_format: DWORD,
) -> Result<DtError, DtError> {
    unsafe {
        let status = TextToSpeechOpenInMemory(tts_handle, audio_format);

        return parse_result(status);
    }
}

pub fn text_to_speech_close_in_memory(tts_handle: LPTTS_HANDLE_T) -> Result<DtError, DtError> {
    unsafe {
        let status = TextToSpeechCloseInMemory(tts_handle);

        return parse_result(status);
    }
}

pub fn text_to_speech_add_buffer(
    tts_handle: LPTTS_HANDLE_T,
    buffer: *mut TTS_BUFFER_T,
) -> Result<DtError, DtError> {
    unsafe {
        let status = TextToSpeechAddBuffer(tts_handle, buffer);

        return parse_result(status);
    }
}

pub struct TTSHandle {
    tts_handle_ptr: LPTTS_HANDLE_T,
}

impl TTSHandle {
    pub fn new() -> Self {
        Self {
            tts_handle_ptr: std::ptr::null_mut(),
        }
    }

    pub fn startup(
        &mut self,
        device_number: UINT,
        device_options: DWORD,
    ) -> Result<DtError, DtError> {
        return text_to_speech_startup(
            &mut self.tts_handle_ptr,
            device_number,
            device_options,
            Some(dt_callback),
            self as *mut Self as *mut usize as LONG,
        );
    }

    pub fn shutdown(&self) -> Result<DtError, DtError> {
        return text_to_speech_shutdown(self.tts_handle_ptr);
    }

    pub fn speak(&self, text: &str, flags: DWORD) -> Result<DtError, DtError> {
        return text_to_speech_speak(self.tts_handle_ptr, text, flags);
    }

    pub fn open_wav_out_file(&self, file: &Path, audio_format: DWORD) -> Result<DtError, DtError> {
        return text_to_speech_open_wave_out_file(self.tts_handle_ptr, file, audio_format);
    }

    pub fn close_wav_out_file(&self) -> Result<DtError, DtError> {
        return text_to_speech_close_wave_out_file(self.tts_handle_ptr);
    }

    pub fn open_in_memory(&self, audio_format: DWORD) -> Result<DtError, DtError> {
        return text_to_speech_open_in_memory(self.tts_handle_ptr, audio_format);
    }

    pub fn close_in_memory(&self) -> Result<DtError, DtError> {
        return text_to_speech_close_in_memory(self.tts_handle_ptr);
    }

    pub fn add_buffer(&self, buffer: *mut TTS_BUFFER_T) -> Result<DtError, DtError> {
        return text_to_speech_add_buffer(self.tts_handle_ptr, buffer);
    }
}

impl fmt::Debug for TTSHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TTSHandle")
            .field("tts_handle_ptr", &self.tts_handle_ptr)
            .finish()
    }
}

extern "C" fn dt_callback(wparam: i64, lparam: i64, user_defined: i64, message: u32) {
    println!("DtCallback called");
    println!(
        "\tWPARAM: {}\n\tLPARAM: {}\n\tUser defined: {}\n\tMessage: {}",
        wparam, lparam, user_defined, message
    );

    // Get the tts handle struct from the pointer
    let tts_handle: *mut TTSHandle = user_defined as *mut TTSHandle;

    if (message == TTS_MSG_BUFFER) {
        let buffer: *mut TTS_BUFFER_T = lparam as *mut TTS_BUFFER_T;

        unsafe {
            dbg!((*buffer));
            println!("{:?}", CStr::from_ptr((*buffer).lpData));

            // Requeue the buffer
            (*tts_handle)
                .add_buffer(buffer)
                .expect("Failed to reuse buffer");
        }
    }
}
