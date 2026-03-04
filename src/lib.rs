pub mod ffi;

use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Notify;

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

fn parse_result(v: ffi::MMRESULT) -> Result<DtError, DtError> {
    match v {
        ffi::MMSYSERR_NOERROR => Ok(DtError::NoError),
        ffi::MMSYSERR_ERROR => Err(DtError::Error),
        ffi::MMSYSERR_BADDEVICEID => Err(DtError::BadDeviceID),
        ffi::MMSYSERR_NOTENABLED => Err(DtError::NotEnabled),
        ffi::MMSYSERR_ALLOCATED => Err(DtError::Allocated),
        ffi::MMSYSERR_INVALHANDLE => Err(DtError::InvalidHandle),
        ffi::MMSYSERR_NODRIVER => Err(DtError::NoDriver),
        ffi::MMSYSERR_NOMEM => Err(DtError::NoMemory),
        ffi::MMSYSERR_NOTSUPPORTED => Err(DtError::NotSupported),
        ffi::MMSYSERR_BADERRNUM => Err(DtError::BadErrorNumber),
        ffi::MMSYSERR_INVALFLAG => Err(DtError::InvalidFlag),
        ffi::MMSYSERR_INVALPARAM => Err(DtError::InvalidParameter),
        ffi::MMSYSERR_HANDLEBUSY => Err(DtError::HandleBusy),
        ffi::MMSYSERR_INVALIDALIAS => Err(DtError::InvalidAlias),
        // Handle all valid cases
        _ => Err(DtError::BadErrorNumber),
    }
}

// ----- Wrapper functions -----
pub fn text_to_speech_version() -> u32 {
    let x = unsafe { ffi::TextToSpeechVersion(std::ptr::null_mut()) };
    return x;
}

pub fn text_to_speech_startup(
    tts_handle: *mut ffi::LPTTS_HANDLE_T,
    device_number: ffi::UINT,
    device_options: ffi::DWORD,
    callback_routine: Option<unsafe extern "C" fn(i64, i64, i64, u32)>,
    callback_parameter: ffi::LONG,
) -> Result<DtError, DtError> {
    dbg!(callback_parameter);

    unsafe {
        let status = ffi::TextToSpeechStartup(
            tts_handle,
            device_number,
            device_options,
            callback_routine,
            callback_parameter,
        );

        return parse_result(status);
    }
}

pub fn text_to_speech_shutdown(tts_handle: ffi::LPTTS_HANDLE_T) -> Result<DtError, DtError> {
    unsafe {
        let status = ffi::TextToSpeechShutdown(tts_handle);

        return parse_result(status);
    }
}

pub fn text_to_speech_speak(
    tts_handle: ffi::LPTTS_HANDLE_T,
    text: String,
    flags: ffi::DWORD,
) -> Result<DtError, DtError> {
    unsafe {
        let status = ffi::TextToSpeechSpeak(tts_handle, text.as_ptr() as *mut i8, flags);

        return parse_result(status);
    }
}

pub fn text_to_speech_open_wave_out_file(
    tts_handle: ffi::LPTTS_HANDLE_T,
    file: &Path,
    audio_format: ffi::DWORD,
) -> Result<DtError, DtError> {
    unsafe {
        let mut filepath: String = String::from(
            std::path::absolute(file)
                .expect("Failed to find absolute path to file")
                .to_str()
                .expect("File path is not a valid string"),
        );

        let status = ffi::TextToSpeechOpenWaveOutFile(
            tts_handle,
            filepath.as_mut_ptr() as *mut i8,
            audio_format,
        );

        return parse_result(status);
    }
}

pub fn text_to_speech_close_wave_out_file(
    tts_handle: ffi::LPTTS_HANDLE_T,
) -> Result<DtError, DtError> {
    unsafe {
        let status = ffi::TextToSpeechCloseWaveOutFile(tts_handle);

        return parse_result(status);
    }
}

pub fn text_to_speech_open_in_memory(
    tts_handle: ffi::LPTTS_HANDLE_T,
    audio_format: ffi::DWORD,
) -> Result<DtError, DtError> {
    unsafe {
        let status = ffi::TextToSpeechOpenInMemory(tts_handle, audio_format);

        return parse_result(status);
    }
}

pub fn text_to_speech_close_in_memory(tts_handle: ffi::LPTTS_HANDLE_T) -> Result<DtError, DtError> {
    unsafe {
        let status = ffi::TextToSpeechCloseInMemory(tts_handle);

        return parse_result(status);
    }
}

pub fn text_to_speech_add_buffer(
    tts_handle: ffi::LPTTS_HANDLE_T,
    buffer: *mut ffi::TTS_BUFFER_T,
) -> Result<DtError, DtError> {
    unsafe {
        let status = ffi::TextToSpeechAddBuffer(tts_handle, buffer);

        return parse_result(status);
    }
}

// ----- TTSOutputBuffer -----
pub struct TTSOutputBuffer {
    pub output_data: Vec<u8>,
    index_mark: ffi::DWORD,
    pub ready: bool,
    notify_ready: Arc<Notify>,
}

impl TTSOutputBuffer {
    pub fn new(index_mark: ffi::DWORD) -> Self {
        Self {
            output_data: Vec::new(),
            index_mark: index_mark,
            ready: false,
            notify_ready: Arc::new(Notify::new()),
        }
    }

    pub fn notify_when_ready(&self) -> Arc<Notify> {
        return self.notify_ready.clone();
    }

    pub fn mark_ready(&mut self) {
        self.ready = true;
        // Notify that this buffer is ready
        self.notify_ready.notify_one();
    }
}

impl fmt::Debug for TTSOutputBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TTSOutputBuffer")
            .field("index_mark", &self.index_mark)
            .field("ready", &self.ready)
            .finish()
    }
}

// ----- TTSHandle -----
#[derive(Debug)]
pub struct TTSHandle {
    tts_handle_ptr: ffi::LPTTS_HANDLE_T,
    buffers: Vec<*mut ffi::TTS_BUFFER_T>,
    pub output_buffers: HashMap<ffi::DWORD, TTSOutputBuffer>,
    pub last_buffer_modified: ffi::DWORD,
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
        device_number: ffi::UINT,
        device_options: ffi::DWORD,
    ) -> Result<DtError, DtError> {
        return text_to_speech_startup(
            &mut self.tts_handle_ptr,
            device_number,
            device_options,
            Some(dt_callback),
            self as *mut Self as *mut usize as ffi::LONG,
        );
    }

    pub fn shutdown(&self) -> Result<DtError, DtError> {
        return text_to_speech_shutdown(self.tts_handle_ptr);
    }

    pub fn speak(
        &mut self,
        text: &str,
        flags: ffi::DWORD,
    ) -> Result<&mut TTSOutputBuffer, DtError> {
        // Find the first integer key not in the hashmap and use that as our index mark
        let unused_key = (1..).find(|i| !self.output_buffers.contains_key(i));

        let index_mark: ffi::DWORD;
        match unused_key {
            Some(n) => index_mark = n,
            // If we can't find an index mark to use, throw an error
            None => return Err(DtError::Error),
        }

        // Create an output buffer and add it to the map
        let output_buffer = TTSOutputBuffer::new(index_mark);
        let output_buffer_reference = self
            .output_buffers
            .entry(index_mark)
            .or_insert(output_buffer);

        let status = text_to_speech_speak(
            self.tts_handle_ptr,
            format!("[:index mark {}]{}", index_mark, text),
            flags,
        );

        match status {
            Ok(_) => return Ok(output_buffer_reference),
            Err(e) => return Err(e),
        }
    }

    pub fn open_wav_out_file(
        &self,
        file: &Path,
        audio_format: ffi::DWORD,
    ) -> Result<DtError, DtError> {
        return text_to_speech_open_wave_out_file(self.tts_handle_ptr, file, audio_format);
    }

    pub fn close_wav_out_file(&self) -> Result<DtError, DtError> {
        return text_to_speech_close_wave_out_file(self.tts_handle_ptr);
    }

    pub fn open_in_memory(&self, audio_format: ffi::DWORD) -> Result<DtError, DtError> {
        return text_to_speech_open_in_memory(self.tts_handle_ptr, audio_format);
    }

    pub fn close_in_memory(&self) -> Result<DtError, DtError> {
        return text_to_speech_close_in_memory(self.tts_handle_ptr);
    }

    pub fn add_buffer(&mut self, buffer: *mut ffi::TTS_BUFFER_T) -> Result<DtError, DtError> {
        return text_to_speech_add_buffer(self.tts_handle_ptr, buffer);
    }

    pub fn create_buffer(
        &mut self,
        data_buffer_size: usize,
        index_buffer_size: usize,
    ) -> Result<DtError, DtError> {
        let mut data = vec![0 as ffi::LPSTR; data_buffer_size];
        let data_ptr = data.as_mut_ptr();
        std::mem::forget(data);

        // TODO: Sort out keeping this alive and then dropping it when done
        let mut index_vec: Vec<ffi::TTS_INDEX_T> = Vec::with_capacity(index_buffer_size);
        let index_vec_ptr = index_vec.as_mut_ptr();
        std::mem::forget(index_vec);

        let buffer = Box::new(ffi::TTS_BUFFER_T {
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
    // println!("DtCallback called");
    // println!(
    //     "\tWPARAM: {}\n\tLPARAM: {}\n\tUser defined: {}\n\tMessage: {}",
    //     wparam, lparam, user_defined, message
    // );

    // Get the tts handle struct from the pointer
    let tts_handle: *mut TTSHandle = user_defined as *mut TTSHandle;

    if (message == ffi::TTS_MSG_BUFFER) {
        let buffer: *mut ffi::TTS_BUFFER_T = lparam as *mut ffi::TTS_BUFFER_T;

        unsafe {
            // Get the data array
            let data_array = std::slice::from_raw_parts(
                (*buffer).lpData as *const u8,
                (*buffer).dwBufferLength as usize,
            );

            // Get the index array
            let index_array = std::slice::from_raw_parts(
                (*buffer).lpIndexArray,
                (*buffer).dwMaximumNumberOfIndexMarks as usize,
            );

            // Print out the index array
            // for (i, mark) in index_array
            //     .iter()
            //     .filter(|m| m.dwIndexValue != 0)
            //     .enumerate()
            // {
            //     println!(
            //         "Index {}: sample={} value={}",
            //         i, mark.dwIndexSampleNumber, mark.dwIndexValue
            //     );
            // }

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
                    // Get the previous buffer and mark it as ready
                    let last_buffer = (*tts_handle).output_buffers.get_mut(&last_buffer_index);
                    match last_buffer {
                        Some(v) => {
                            v.mark_ready();
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
