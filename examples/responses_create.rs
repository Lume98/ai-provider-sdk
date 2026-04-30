#[path = "common/mod.rs"]
mod shared;

use vendor_ai_sdk::{ResponseCreateParams, ResponseOutputContent, ResponseOutputItem};

#[tokio::main]
async fn main() -> Result<(), vendor_ai_sdk::Error> {
    let client = shared::client();

    let response = client
        .responses
        .create(&ResponseCreateParams::new(
            "gpt-4.1-mini",
            "写一句简短的 Rust 学习建议",
        ))
        .await?;

    for item in &response.output {
        if let ResponseOutputItem::Message { content, .. } = item {
            for part in content {
                if let ResponseOutputContent::OutputText { text, .. } = part {
                    println!("{text}");
                }
            }
        }
    }

    Ok(())
}
