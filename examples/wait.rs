use core::time::Duration;
use futures::stream::StreamExt;
use futures_retry::{RetryPolicy, StreamRetryExt};
use pin_utils::pin_mut;
use reqwest::Client;
use reqwest_eventsource::{Error, RequestBuilderExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let stream = client
        .get("http://localhost:7020/notifications")
        .eventsource()?
        .retry(|err| match err {
            Error::Transport(_) => {
                println!("transport error: retry in 3s");
                RetryPolicy::<()>::WaitRetry(Duration::from_secs(3))
            }
            Error::Parse(_) => {
                println!("parse error: retry immediately");
                RetryPolicy::<()>::Repeat
            }
        });
    pin_mut!(stream);
    while let Some(event) = stream.next().await {
        match event {
            Ok((event, _)) => println!(
                "received: {:?}: {}",
                event.event,
                String::from_utf8_lossy(&event.data)
            ),
            Err(_) => unreachable!(),
        }
    }

    Ok(())
}
