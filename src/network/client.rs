use crate::network::fingerprint::{DeviceProfile, Fingerprint};
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;

pub struct FetchResult {
    pub status: u16,
    pub final_url: String,
    pub html: String,
    pub is_captcha_detected: bool,
}

pub struct NetworkClient {
    client: reqwest::Client,
    pub profile: DeviceProfile,
    pub fingerprint: Fingerprint,
    #[allow(dead_code)]
    pub cookie_jar: Arc<reqwest::cookie::Jar>,
}

impl NetworkClient {
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        Self::with_profile(DeviceProfile::ChromeWindows)
    }

    pub fn with_profile(profile: DeviceProfile) -> Result<Self> {
        Self::with_builder_config(profile, None, Duration::from_secs(30), 10, None)
    }

    pub fn with_builder_config(
        profile: DeviceProfile,
        proxy_url: Option<&str>,
        timeout: Duration,
        max_redirects: usize,
        custom_ua: Option<&str>,
    ) -> Result<Self> {
        let cookie_jar = Arc::new(reqwest::cookie::Jar::default());
        let fingerprint = Fingerprint::for_profile(profile);
        let mut headers = fingerprint.build_headers();

        if let Some(ua) = custom_ua {
            headers.insert(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_str(ua)
                    .context("Invalid custom User-Agent header")?,
            );
        }

        let mut builder = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_provider(cookie_jar.clone())
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::limited(max_redirects))
            .gzip(true)
            .brotli(true)
            .deflate(true);

        if let Some(proxy_str) = proxy_url {
            let proxy = reqwest::Proxy::all(proxy_str)
                .context(format!("Invalid proxy URL: {}", proxy_str))?;
            builder = builder.proxy(proxy);
        }

        // StealthGuard: Pre-warm legitimate session and consent cookies
        if let Ok(google_url) = "https://www.google.com".parse::<reqwest::Url>() {
            cookie_jar.add_cookie_str("SOCS=CAESHAgBEhJnd3NfMjAyNDA5MDUtMF9SQzIaAmVuIAEaBgiA_L20Bg; Path=/; Domain=.google.com; Secure", &google_url);
            cookie_jar.add_cookie_str(
                "CONSENT=PENDING+987; Path=/; Domain=.google.com; Secure",
                &google_url,
            );
            cookie_jar.add_cookie_str(
                "AEC=AZ6Zc-Wz2R61_67w88hJ; Path=/; Domain=.google.com; Secure",
                &google_url,
            );
        }
        if let Ok(ddg_url) = "https://duckduckgo.com".parse::<reqwest::Url>() {
            cookie_jar.add_cookie_str("5=0; Path=/; Domain=.duckduckgo.com; Secure", &ddg_url);
            cookie_jar.add_cookie_str("l=en-us; Path=/; Domain=.duckduckgo.com; Secure", &ddg_url);
        }

        let client = builder.build().context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            profile,
            fingerprint,
            cookie_jar,
        })
    }

    pub fn set_profile(&mut self, profile: DeviceProfile) -> Result<()> {
        let new_client = Self::with_profile(profile)?;
        self.client = new_client.client;
        self.profile = profile;
        self.fingerprint = new_client.fingerprint;
        Ok(())
    }

    pub async fn fetch(&self, url: &str) -> Result<FetchResult> {
        if url.starts_with("file://") {
            let file_path = url
                .trim_start_matches("file:///")
                .trim_start_matches("file://");
            let clean_path = file_path.replace('/', "\\");
            let html = std::fs::read_to_string(&clean_path)
                .or_else(|_| std::fs::read_to_string(file_path))
                .context(format!("Failed to read local file: {}", url))?;
            return Ok(FetchResult {
                status: 200,
                final_url: url.to_string(),
                html,
                is_captcha_detected: false,
            });
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context(format!("Failed to send request to {}", url))?;

        let status = response.status().as_u16();
        let final_url = response.url().to_string();
        let html = response
            .text()
            .await
            .context("Failed to decode response body")?;

        let is_captcha_detected = html.contains("sorry/index?continue=")
            || html.contains("Our systems have detected unusual traffic")
            || html.contains("id=\"captcha-form\"")
            || (html.contains("challenges.cloudflare.com")
                && html.contains("cf-turnstile-wrapper"))
            || html.contains("hcaptcha-box");

        Ok(FetchResult {
            status,
            final_url,
            html,
            is_captcha_detected,
        })
    }
}
