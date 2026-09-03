<<<<<<< HEAD
use anyhow::anyhow;
use gpui::http_client::{HttpClient, HttpResponse};
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;
=======
use crate::WebDispatcher;
use anyhow::{Context as _, anyhow};
use futures::{
    AsyncRead, AsyncReadExt as _, FutureExt as _, SinkExt as _, TryStreamExt as _,
    channel::{mpsc, oneshot},
};
use http_client::{AsyncBody, HttpClient, RedirectPolicy};
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
>>>>>>> ae625934ba7c510bdf18099911e025fc9bee4e57
use wasm_bindgen::JsCast as _;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = "fetch")]
    fn global_fetch(input: &web_sys::Request) -> Result<js_sys::Promise, JsValue>;
}

pub struct FetchHttpClient;

impl Default for FetchHttpClient {
    fn default() -> Self {
        Self
    }
}

#[cfg(feature = "multithreaded")]
impl FetchHttpClient {
    pub unsafe fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "multithreaded"))]
impl FetchHttpClient {
    pub fn new() -> Self {
        Self
    }
}

/// Wraps a `!Send` future to satisfy the `Send` bound on `BoxFuture`.
struct AssertSend<F>(F);

unsafe impl<F> Send for AssertSend<F> {}

impl<F: Future> Future for AssertSend<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let inner = unsafe { self.map_unchecked_mut(|this| &mut this.0) };
        inner.poll(cx)
    }
}

impl HttpClient for FetchHttpClient {
    fn get(
        &self,
        url: &str,
        follow_redirects: bool,
    ) -> futures::future::BoxFuture<'static, anyhow::Result<HttpResponse>> {
        let url = url.to_string();
        Box::pin(AssertSend(async move {
            let init = web_sys::RequestInit::new();
            init.set_method("GET");

            if !follow_redirects {
                init.set_redirect(web_sys::RequestRedirect::Manual);
            }

            let request = web_sys::Request::new_with_str_and_init(&url, &init)
                .map_err(|error| anyhow!("failed to create fetch Request: {error:?}"))?;

            let promise = global_fetch(&request)
                .map_err(|error| anyhow!("fetch threw an error: {error:?}"))?;
            let response_value = wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .map_err(|error| anyhow!("fetch failed: {error:?}"))?;

            let web_response: web_sys::Response = response_value
                .dyn_into()
                .map_err(|error| anyhow!("fetch result is not a Response: {error:?}"))?;

            let status_code = http::StatusCode::from_u16(web_response.status())
                .map_err(|_| anyhow!("invalid status code"))?;

            let body_promise = web_response
                .array_buffer()
                .map_err(|error| anyhow!("failed to initiate response body read: {error:?}"))?;
            let body_value = wasm_bindgen_futures::JsFuture::from(body_promise)
                .await
                .map_err(|error| anyhow!("failed to read response body: {error:?}"))?;
            let array_buffer: js_sys::ArrayBuffer = body_value
                .dyn_into()
                .map_err(|error| anyhow!("response body is not an ArrayBuffer: {error:?}"))?;
            let body = js_sys::Uint8Array::new(&array_buffer).to_vec();

<<<<<<< HEAD
            Ok(HttpResponse {
                status: status_code,
                body,
            })
        }))
=======
    let status = web_response.status();
    let mut builder = http_client::http::Response::builder().status(status);

    // `Headers` is a JS iterable yielding `[name, value]` pairs.
    // `js_sys::Array::from` calls `Array.from()` which accepts any iterable.
    let header_pairs = js_sys::Array::from(&web_response.headers());
    for index in 0..header_pairs.length() {
        match header_pairs.get(index).dyn_into::<js_sys::Array>() {
            Ok(pair) => match (pair.get(0).as_string(), pair.get(1).as_string()) {
                (Some(name), Some(value)) => {
                    builder = builder.header(name, value);
                }
                (name, value) => {
                    log::warn!(
                        "skipping response header at index {index}: \
                                     name={name:?}, value={value:?}"
                    );
                }
            },
            Err(entry) => {
                log::warn!("skipping non-array header entry at index {index}: {entry:?}");
            }
        }
    }

    let body = match web_response.body() {
        Some(stream) => {
            let reader = stream
                .get_reader()
                .dyn_into::<web_sys::ReadableStreamDefaultReader>()
                .map_err(|error| {
                    anyhow!("response body reader has an unexpected type: {error:?}")
                })?;
            AsyncBody::from_reader(ReadableStreamBody::new(reader))
        }
        None => AsyncBody::empty(),
    };

    builder.body(body).map_err(|error| anyhow!(error))
}

// Request bodies are buffered into memory because streaming uploads require
// half-duplex Fetch support that browsers largely don't ship yet.
async fn read_body_to_bytes(mut body: AsyncBody) -> anyhow::Result<Option<Vec<u8>>> {
    let mut buffer = Vec::new();
    body.read_to_end(&mut buffer).await?;
    if buffer.is_empty() {
        Ok(None)
    } else {
        Ok(Some(buffer))
>>>>>>> ae625934ba7c510bdf18099911e025fc9bee4e57
    }
}

const RESPONSE_BODY_CHANNEL_CAPACITY: usize = 8;

struct ReadableStreamBody {
    chunks: futures::stream::IntoAsyncRead<mpsc::Receiver<io::Result<Vec<u8>>>>,
    // Dropping this sender resolves the pump's cancellation future, which
    // cancels the browser-side `ReadableStream`.
    _cancellation: oneshot::Sender<()>,
}

impl ReadableStreamBody {
    fn new(reader: web_sys::ReadableStreamDefaultReader) -> Self {
        let (chunks_sender, chunks_receiver) = mpsc::channel(RESPONSE_BODY_CHANNEL_CAPACITY);
        let (cancellation, cancellation_receiver) = oneshot::channel();
        wasm_bindgen_futures::spawn_local(pump_response_body(
            reader,
            chunks_sender,
            cancellation_receiver,
        ));
        Self {
            chunks: chunks_receiver.into_async_read(),
            _cancellation: cancellation,
        }
    }
}

impl AsyncRead for ReadableStreamBody {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.chunks).poll_read(cx, buffer)
    }
}

async fn pump_response_body(
    reader: web_sys::ReadableStreamDefaultReader,
    mut chunks: mpsc::Sender<io::Result<Vec<u8>>>,
    cancellation: oneshot::Receiver<()>,
) {
    let cancellation = cancellation.fuse();
    futures::pin_mut!(cancellation);

    loop {
        let read = wasm_bindgen_futures::JsFuture::from(reader.read()).fuse();
        futures::pin_mut!(read);
        let result = futures::select_biased! {
            _ = cancellation => {
                cancel_reader(&reader).await;
                return;
            }
            result = read => result,
        };

        let chunk = result
            .map_err(|error| io::Error::other(format!("response stream failed: {error:?}")))
            .and_then(response_chunk);
        match chunk {
            Ok(Some(chunk)) => {
                if chunks.send(Ok(chunk)).await.is_err() {
                    cancel_reader(&reader).await;
                    return;
                }
            }
            Ok(None) => return,
            Err(error) => {
                if chunks.send(Err(error)).await.is_err() {
                    log::debug!("response body receiver was dropped after a stream error");
                }
                return;
            }
        }
    }
}

fn response_chunk(result: JsValue) -> io::Result<Option<Vec<u8>>> {
    // `ReadableStreamReadResult` is a dictionary type, so there is no runtime
    // class to check against; `unchecked_into` is the only available cast.
    let result: web_sys::ReadableStreamReadResult = result.unchecked_into();
    if result.get_done().unwrap_or(false) {
        return Ok(None);
    }

    result
        .get_value()
        .dyn_into::<js_sys::Uint8Array>()
        .map(|bytes| Some(bytes.to_vec()))
        .map_err(|value| {
            io::Error::other(format!(
                "response stream yielded a non-byte chunk: {value:?}"
            ))
        })
}

async fn cancel_reader(reader: &web_sys::ReadableStreamDefaultReader) {
    if let Err(error) = wasm_bindgen_futures::JsFuture::from(reader.cancel()).await {
        log::debug!("failed to cancel response body reader: {error:?}");
    }
}
