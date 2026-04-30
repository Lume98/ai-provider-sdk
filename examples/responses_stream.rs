#[path = "common/mod.rs"]
mod shared;

use futures::StreamExt;
use vendor_ai_sdk::ResponseCreateParams;

#[tokio::main]
async fn main() -> Result<(), vendor_ai_sdk::Error> {
    let client = shared::client();

    let mut stream = client
        .responses
        .stream(&ResponseCreateParams::new(
            "gpt-5.3-codex",
            "1+1",
        ))
        .await?;

    while let Some(event) = stream.next().await {
        let event = event?;
        println!("{} {:?}", event.event_type, event.data);
    }

    Ok(())
}
