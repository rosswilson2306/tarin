use client::PsiClient;
use dotenv::dotenv;
use std::error::Error;

mod client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let psi_url = std::env::var("PSI_URL")?;
    let psi_key = std::env::var("PSI_KEY")?;

    let client = PsiClient::new(&psi_url, &psi_key);

    let _report = client.get_report("https://google.com").await?;

    Ok(())
}
