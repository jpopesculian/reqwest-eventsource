//! Provides a simple wrapper for [`reqwest`] to provide an Event Source implementation.
//! You can learn more about Server Sent Events (SSE) take a look at [the MDN
//! docs](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events)
//! This crate uses [`eventsource_stream`] to wrap the underlying Bytes stream, and retries failed
//! requests.
//!
//! # Example
//!
//! For more examples with delaying and error handling, take a look at the `examples/`
//!
//! ```ignore
//! let client = Client::new();
//! let mut stream = client
//!     .get("http://localhost:7020/notifications")
//!     .eventsource()?;
//!
//! while let Some(event) = stream.next().await {
//!     match event {
//!         Ok(event) => println!(
//!             "received: {:?}: {}",
//!             event.event,
//!             String::from_utf8_lossy(&event.data)
//!         ),
//!         Err(e) => eprintln!("error occured: {}", e),
//!     }
//! }
//! ```

pub use eventsource_stream::{Error, Event, ParseError};

use core::fmt;
use core::pin::Pin;
use eventsource_stream::Eventsource;
use futures_core::future::BoxFuture;
use futures_core::stream::{BoxStream, Stream};
use futures_core::task::{Context, Poll};
use reqwest::{RequestBuilder, Response};

type ResponseFuture = BoxFuture<'static, Result<Response, reqwest::Error>>;
type EventStream = BoxStream<'static, Result<Event, Error<reqwest::Error>>>;

/// Error raised when a [`RequestBuilder`] cannot be cloned. See [`RequestBuilder::try_clone`] for
/// more information
#[derive(Debug, Clone, Copy)]
pub struct CannotCloneRequestError;

/// Provides the [`Stream`] implementation for the [`Event`] items. This wraps the
/// [`RequestBuilder`] and retries requests when they fail.
pub struct EventsourceRequestBuilder {
    builder: RequestBuilder,
    next_response: Option<ResponseFuture>,
    cur_stream: Option<EventStream>,
}

struct EventsourceRequestBuilderProjection<'a> {
    builder: RequestBuilder,
    next_response: &'a mut Option<ResponseFuture>,
    cur_stream: &'a mut Option<EventStream>,
}

/// Provides an easy interface to build an [`EventsourceRequestBuilder`] from a [`RequestBuilder`]
pub trait RequestBuilderExt {
    fn eventsource(self) -> Result<EventsourceRequestBuilder, CannotCloneRequestError>;
}

impl EventsourceRequestBuilder {
    /// Wrap a [`RequestBuilder`]
    #[inline]
    pub fn new(builder: RequestBuilder) -> Result<Self, CannotCloneRequestError> {
        let res_future = Box::pin(
            builder
                .try_clone()
                .ok_or_else(|| CannotCloneRequestError)?
                .send(),
        );
        Ok(Self {
            builder,
            next_response: Some(res_future),
            cur_stream: None,
        })
    }

    #[inline]
    fn projection<'a>(self: Pin<&'a mut Self>) -> EventsourceRequestBuilderProjection<'a> {
        unsafe {
            let inner = self.get_unchecked_mut();
            EventsourceRequestBuilderProjection {
                builder: inner.builder.try_clone().unwrap(),
                next_response: &mut inner.next_response,
                cur_stream: &mut inner.cur_stream,
            }
        }
    }
}

impl Stream for EventsourceRequestBuilder {
    type Item = Result<Event, Error<reqwest::Error>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.projection();
        if let Some(response_future) = this.next_response {
            match response_future.as_mut().poll(cx) {
                Poll::Ready(Ok(res)) => {
                    this.next_response.take();
                    let event_stream = Box::pin(res.bytes_stream().eventsource());
                    this.cur_stream.replace(event_stream);
                }
                Poll::Ready(Err(e)) => {
                    let res_future = Box::pin(this.builder.try_clone().unwrap().send());
                    this.next_response.replace(res_future);
                    return Poll::Ready(Some(Err(Error::Transport(e))));
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
        match this.cur_stream.as_mut().unwrap().as_mut().poll_next(cx) {
            Poll::Ready(Some(Err(err))) => {
                if matches!(err, Error::Transport(_)) {
                    this.cur_stream.take();
                    let res_future = Box::pin(this.builder.try_clone().unwrap().send());
                    this.next_response.replace(res_future);
                }
                Poll::Ready(Some(Err(err)))
            }
            Poll::Ready(Some(Ok(event))) => Poll::Ready(Some(Ok(event))),
            Poll::Ready(None) => {
                this.cur_stream.take();
                let res_future = Box::pin(this.builder.try_clone().unwrap().send());
                this.next_response.replace(res_future);
                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl RequestBuilderExt for RequestBuilder {
    fn eventsource(self) -> Result<EventsourceRequestBuilder, CannotCloneRequestError> {
        EventsourceRequestBuilder::new(self)
    }
}

impl fmt::Display for CannotCloneRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("expected a cloneable request")
    }
}

impl std::error::Error for CannotCloneRequestError {}
