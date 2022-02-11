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

mod error;
mod event_source;
mod reqwest_ext;
pub mod retry;

pub use error::{CannotCloneRequestError, Error};
pub use event_source::{Event, EventSource};
pub use reqwest_ext::RequestBuilderExt;
