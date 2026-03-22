use std::io;

use bytes::Bytes;
use futures_util::StreamExt;
use sdkwork_api_provider_core::ProviderStreamOutput;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub enum OpenAiSseEvent {
    Json(Value),
    Done,
}

pub fn transform_openai_sse_stream<State, F>(
    response: ProviderStreamOutput,
    state: State,
    mapper: F,
) -> ProviderStreamOutput
where
    State: Send + 'static,
    F: FnMut(&mut State, OpenAiSseEvent) -> Vec<String> + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<Result<Bytes, io::Error>>(16);

    tokio::spawn(async move {
        let mut stream = response.into_body_stream();
        let mut mapper = mapper;
        let mut state = state;
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    buffer.push_str(&String::from_utf8_lossy(&chunk).replace("\r\n", "\n"));

                    while let Some(event) = take_next_sse_event(&mut buffer) {
                        let Some(parsed) = parse_openai_sse_event(&event) else {
                            continue;
                        };
                        if emit_frames(&tx, mapper(&mut state, parsed)).await.is_err() {
                            return;
                        }
                    }
                }
                Err(error) => {
                    let _ = tx.send(Err(error)).await;
                    return;
                }
            }
        }
    });

    ProviderStreamOutput::new("text/event-stream", ReceiverStream::new(rx))
}

pub fn sse_data_frame(value: &Value) -> String {
    format!("data: {}\n\n", value)
}

pub fn sse_named_event_frame(event: &str, value: &Value) -> String {
    format!("event: {event}\ndata: {}\n\n", value)
}

async fn emit_frames(
    tx: &mpsc::Sender<Result<Bytes, io::Error>>,
    frames: Vec<String>,
) -> Result<(), ()> {
    for frame in frames {
        if tx.send(Ok(Bytes::from(frame))).await.is_err() {
            return Err(());
        }
    }
    Ok(())
}

fn take_next_sse_event(buffer: &mut String) -> Option<String> {
    let delimiter_index = buffer.find("\n\n")?;
    let event = buffer[..delimiter_index].to_owned();
    let remainder = buffer[delimiter_index + 2..].to_owned();
    *buffer = remainder;
    Some(event)
}

fn parse_openai_sse_event(event: &str) -> Option<OpenAiSseEvent> {
    let data = event
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim_start)
        .collect::<Vec<_>>()
        .join("\n");

    if data.is_empty() {
        return None;
    }

    if data == "[DONE]" {
        return Some(OpenAiSseEvent::Done);
    }

    serde_json::from_str::<Value>(&data)
        .ok()
        .map(OpenAiSseEvent::Json)
}
