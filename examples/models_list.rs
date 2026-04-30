#[path = "common/mod.rs"]
mod shared;

#[tokio::main]
async fn main() -> Result<(), vendor_ai_sdk::Error> {
    let client = shared::client();

    let page = client.models.list().await?;

    for model in page.data.iter().take(10) {
        println!("{}", model.id);
    }

    Ok(())
}
