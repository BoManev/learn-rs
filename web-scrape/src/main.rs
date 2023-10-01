use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    web_scrape::scrape_entry("Atlanta").await?;
    Ok(())
}