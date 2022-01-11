use futures::stream::StreamExt;
use reqwest::Client;
use reqwest_eventsource::RequestBuilderExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut stream = client
        .get("http://localhost:7020/notifications")
        .eventsource()?;

    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => println!("received: {:?}: {}", event.event, event.data),
            Err(e) => eprintln!("error occured: {}", e),
        }
    }

    Ok(())
}
