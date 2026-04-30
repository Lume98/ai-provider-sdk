#[path = "common/mod.rs"]
mod shared;

use bytes::Bytes;
use vendor_ai_sdk::FileCreateParams;

#[tokio::main]
async fn main() -> Result<(), vendor_ai_sdk::Error> {
    let client = shared::client();

    let file = client
        .files
        .create(FileCreateParams::from_bytes(
            "train.jsonl",
            Bytes::from_static(br#"{\"messages\": [{\"role\": \"user\", \"content\": \"hello\"}]}
"#),
            "fine-tune",
        ))
        .await?;

    println!("created file id: {}", file.id);

    Ok(())
}
