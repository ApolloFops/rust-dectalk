#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// This may be a hacky workaround but IDK enough about rust to know if it is
// It does work though
#[link(name = "dectalk")]
unsafe extern "C" {}

// ----- DtError -----
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

// ----- Wrapper functions -----
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
    text: String,
    flags: DWORD,
) -> Result<DtError, DtError> {
    unsafe {
        let status = TextToSpeechSpeak(tts_handle, text.as_ptr() as *mut i8, flags);

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

// ----- TTSOutputBuffer -----
pub struct TTSOutputBuffer {
    pub output_data: Vec<u8>,
    index_mark: DWORD,
    pub done: bool,
}

impl TTSOutputBuffer {
    pub fn new(index_mark: DWORD) -> Self {
        Self {
            output_data: Vec::new(),
            index_mark: index_mark,
            done: false,
        }
    }
}

impl fmt::Debug for TTSOutputBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TTSOutputBuffer")
            .field("index_mark", &self.index_mark)
            .finish()
    }
}

// ----- TTSHandle -----
#[derive(Debug)]
pub struct TTSHandle {
    tts_handle_ptr: LPTTS_HANDLE_T,
    buffers: Vec<*mut TTS_BUFFER_T>,
    pub output_buffers: HashMap<DWORD, TTSOutputBuffer>,
    pub last_buffer_modified: DWORD,
}

impl TTSHandle {
    pub fn new() -> Self {
        Self {
            tts_handle_ptr: std::ptr::null_mut(),
            buffers: Vec::new(),
            output_buffers: HashMap::new(),
            last_buffer_modified: 0,
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

    pub fn speak(&mut self, text: &str, flags: DWORD) -> Result<DtError, DtError> {
        // Find the first integer key not in the hashmap and use that as our index mark
        let unused_key = (1..).find(|i| !self.output_buffers.contains_key(i));

        let index_mark: DWORD;
        match unused_key {
            Some(n) => index_mark = n,
            // If we can't find an index mark to use, throw an error
            None => return Err(DtError::Error),
        }

        // Create an output buffer and add it to the map
        let output_buffer = TTSOutputBuffer::new(index_mark);
        self.output_buffers.insert(index_mark, output_buffer);

        return text_to_speech_speak(
            self.tts_handle_ptr,
            format!("[:index mark {}]{}", index_mark, text),
            flags,
        );
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

    pub fn add_buffer(&mut self, buffer: *mut TTS_BUFFER_T) -> Result<DtError, DtError> {
        return text_to_speech_add_buffer(self.tts_handle_ptr, buffer);
    }

    pub fn create_buffer(
        &mut self,
        data_buffer_size: usize,
        index_buffer_size: usize,
    ) -> Result<DtError, DtError> {
        let mut data = vec![0 as LPSTR; data_buffer_size];
        let data_ptr = data.as_mut_ptr();
        std::mem::forget(data);

        // TODO: Sort out keeping this alive and then dropping it when done
        let mut index_vec: Vec<TTS_INDEX_T> = Vec::with_capacity(index_buffer_size);
        let index_vec_ptr = index_vec.as_mut_ptr();
        std::mem::forget(index_vec);

        let buffer = Box::new(TTS_BUFFER_T {
            lpData: data_ptr as *mut i8,
            dwMaximumBufferLength: data_buffer_size as u32,
            lpPhonemeArray: std::ptr::null_mut(),
            lpIndexArray: index_vec_ptr,
            dwMaximumNumberOfPhonemeChanges: 0,
            dwMaximumNumberOfIndexMarks: index_buffer_size as u32,
            dwBufferLength: 0,
            dwNumberOfPhonemeChanges: 0,
            dwNumberOfIndexMarks: 0,
            dwReserved: 0,
        });

        let buffer_ptr = Box::into_raw(buffer);

        let status = self.add_buffer(unsafe { &mut *buffer_ptr });

        self.buffers.push(buffer_ptr);

        return status;
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
            // Get the data array
            let data_array = std::slice::from_raw_parts(
                (*buffer).lpData as *const u8,
                (*buffer).dwBufferLength as usize,
            );

            // Get the index array and print it out
            let index_array = std::slice::from_raw_parts(
                (*buffer).lpIndexArray,
                (*buffer).dwMaximumNumberOfIndexMarks as usize,
            );
            for (i, mark) in index_array
                .iter()
                .filter(|m| m.dwIndexValue != 0)
                .enumerate()
            {
                println!(
                    "Index {}: sample={} value={}",
                    i, mark.dwIndexSampleNumber, mark.dwIndexValue
                );
            }

            // Append data to the output buffer
            for (i, mark) in index_array
                .iter()
                .filter(|m| m.dwIndexValue != 0)
                .enumerate()
            {
                // If this buffer is different from the last one we wrote to, mark the last one as
                // done
                // TODO: I'm sure there's a better way to do this whole thing
                let last_buffer_index = (*tts_handle).last_buffer_modified;

                if mark.dwIndexValue != last_buffer_index {
                    // Get the previous buffer and mark it as done
                    let last_buffer = (*tts_handle).output_buffers.get_mut(&last_buffer_index);
                    match last_buffer {
                        Some(v) => {
                            v.done = true;
                            println!("Done with buffer");
                        }
                        None => eprintln!("Previous index tag not in buffer cache"),
                    }

                    // Set this buffer as the last buffer modified
                    (*tts_handle).last_buffer_modified = mark.dwIndexValue;
                }

                // Find the buffer to write to
                let buffer = (*tts_handle).output_buffers.get_mut(&mark.dwIndexValue);

                match buffer {
                    // If we found a buffer, append the sample data to it
                    // TODO: Check for and handle the case where we have multiple indices in one
                    // message
                    Some(v) => v.output_data.extend_from_slice(data_array),
                    None => eprintln!("Index tag not in buffer cache"),
                }
            }

            // Requeue the buffer
            (*tts_handle)
                .add_buffer(buffer)
                .expect("Failed to reuse buffer");
        }
    }
}
