use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use dectalk;

use axum::extract::{Query, State};
use axum::{routing::get, Router};
use hound::{SampleFormat, WavSpec, WavWriter};
use tokio::sync::Mutex;

/// The message output when the URL parameter is not found.
const DEFAULT_MESSAGE: &str = "Specify text to speak with a URL parameter, in the form question mark, text, equals, the text you want to speak in quotes.";

// The buffer sizes to use
const DATA_BUFFER_SIZE: usize = 4096;
const INDEX_BUFFER_SIZE: usize = 128;

/// Persistent state between different requests.
#[derive(Clone)]
struct AppState {
    dectalk: Arc<Mutex<dectalk::TTSHandle>>,
}

#[tokio::main]
async fn main() {
    println!("DECTalk Version: {}", dectalk::text_to_speech_version());

    // Create the AppState struct. This will keep track of the DECTalk handle.
    let state = AppState {
        dectalk: Arc::new(Mutex::new(dectalk::TTSHandle::new())),
    };

    // Set up DECTalk
    let mut dectalk_lock = state.dectalk.lock().await;
    dectalk_lock.startup(0, 0).expect("Failed to start DECTalk");
    dectalk_lock
        .open_in_memory(dectalk::DtTTSFormat::WaveFormat1M16)
        .expect("Failed to open DECTalk in memory");
    dectalk_lock
        .create_buffer(DATA_BUFFER_SIZE, INDEX_BUFFER_SIZE)
        .expect("Failed to create buffer");
    drop(dectalk_lock);

    // Set up the app
    let app = Router::new().route("/", get(get_tts)).with_state(state);

    // Run the HTTP server
    let address = "0.0.0.0:3000";
    println!("Running HTTP server on http://{}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_tts(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, StatusCode> {
    // Figure out what text was requested
    let default_string = &String::from(DEFAULT_MESSAGE);
    let text = params.get("text").unwrap_or(default_string);

    let mut tts_guard = state.dectalk.lock().await;

    // Queue speech and wait for it to be done
    let output_buffer_future = tts_guard
        .speak(text, dectalk::DtTTSFlags::Force)
        .expect("Failed to queue speech");

    let output_buffer = output_buffer_future.await;

    // Set up the WAV file
    let spec = WavSpec {
        channels: 1,
        sample_rate: 11025,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    let mut writer = WavWriter::new(&mut cursor, spec).unwrap();

    for chunk in output_buffer.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        writer.write_sample(sample).unwrap();
    }

    writer.finalize().unwrap();
    let wav_bytes = cursor.into_inner();

    let response = Response::builder()
        .header(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("audio/wav"),
        )
        .body(axum::body::Body::from(wav_bytes))
        .unwrap();

    return Ok((StatusCode::OK, response).into_response());
}
