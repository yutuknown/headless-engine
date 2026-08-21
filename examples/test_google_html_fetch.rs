use anyhow::Result;
use headless_engine::network::client::NetworkClient;
use headless_engine::network::fingerprint::DeviceProfile;

#[tokio::main]
async fn main() -> Result<()> {
    let client = NetworkClient::with_profile(DeviceProfile::ChromeWindows)?;

    let urls = [
        "https://www.google.com/search?q=Rust+programming+language&udm=14&hl=en",
        "https://www.google.com/search?q=Rust+programming+language&gbv=1&hl=en",
        "https://www.google.com/search?q=Rust+programming+language&hl=en",
    ];

    for url in urls {
        println!("--------------------------------------------------");
        println!("Fetching: {}", url);
        let res = client.fetch(url).await?;
        println!("Status: {}", res.status);
        println!("Bytes:  {}", res.html.len());
        println!("Contains search results container: {}", res.html.contains("id=\"search\"") || res.html.contains("class=\"g\"") || res.html.contains("<h3") || res.html.contains("id=\"rso\""));
        println!("Contains fallback/sorry:           {}", res.html.contains("having trouble accessing") || res.html.contains("sorry/index"));
    }

    Ok(())
}
