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

        let effective_proxy = proxy_url
            .map(|s| s.to_string())
            .or_else(|| std::env::var("HTTPS_PROXY").ok())
            .or_else(|| std::env::var("https_proxy").ok())
            .or_else(|| std::env::var("HTTP_PROXY").ok())
            .or_else(|| std::env::var("http_proxy").ok())
            .or_else(|| std::env::var("ALL_PROXY").ok())
            .or_else(|| std::env::var("all_proxy").ok());

        if let Some(proxy_str) = &effective_proxy {
            if !proxy_str.is_empty() {
                let proxy = reqwest::Proxy::all(proxy_str)
                    .context(format!("Invalid proxy URL: {}", proxy_str))?;
                builder = builder.proxy(proxy);
            }
        }

        // StealthGuard: Pre-warm legitimate session and consent cookies
        if let Ok(google_url) = "https://www.google.com".parse::<reqwest::Url>() {
            cookie_jar.add_cookie_str("SOCS=CAESHAgBEhJnd3NfMjAyNDA5MDUtMF9SQzIaAmVuIAEaBgiA_L20Bg; Path=/; Domain=.google.com; Secure", &google_url);
            cookie_jar.add_cookie_str(
                "CONSENT=YES+cb.20230531-04-p0.en+FX+908; Path=/; Domain=.google.com; Secure",
                &google_url,
            );
            cookie_jar.add_cookie_str(
                "AEC=AZ6Zc-Wz2R61_67w88hJ; Path=/; Domain=.google.com; Secure",
                &google_url,
            );
            cookie_jar.add_cookie_str(
                "1P_JAR=2024-09-05-12; Path=/; Domain=.google.com; Secure",
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

        let target_url = if url.contains("google.com/search") && !url.contains("hl=") {
            if url.contains('?') {
                format!("{}&hl=en&gl=us", url)
            } else {
                format!("{}?hl=en&gl=us", url)
            }
        } else {
            url.to_string()
        };

        let response = self
            .client
            .get(&target_url)
            .send()
            .await
            .context(format!("Failed to send request to {}", url))?;

        let mut status = response.status().as_u16();
        let mut final_url = response.url().to_string();
        let bytes = response
            .bytes()
            .await
            .context("Failed to read response body bytes")?;
        let mut html = String::from_utf8_lossy(&bytes).to_string();

        // If Google serves an interstitial fallback link, follow it directly
        if html.contains("having trouble accessing Google Search") || html.contains("emsg=SG_REL") {
            if let Some(start_idx) = html.find("href=\"/search?") {
                if let Some(end_idx) = html[start_idx + 6..].find('\"') {
                    let rel_url = &html[start_idx + 6..start_idx + 6 + end_idx];
                    let clean_rel = rel_url.replace("&amp;", "&");
                    let redirect_target = format!("https://www.google.com{}", clean_rel);
                    if let Ok(next_resp) = self.client.get(&redirect_target).send().await {
                        status = next_resp.status().as_u16();
                        final_url = next_resp.url().to_string();
                        if let Ok(next_bytes) = next_resp.bytes().await {
                            html = String::from_utf8_lossy(&next_bytes).to_string();
                        }
                    }
                }
            }
        }

        let mut is_captcha_detected = html.contains("sorry/index?continue=")
            || html.contains("Our systems have detected unusual traffic")
            || html.contains("id=\"captcha-form\"")
            || (html.contains("challenges.cloudflare.com")
                && html.contains("cf-turnstile-wrapper"))
            || html.contains("hcaptcha-box");

        // If Google serves an anti-bot fallback, hydrate with live rendered DOM
        if (url.contains("google.com/search") && (html.contains("having trouble accessing Google Search") || html.contains("emsg=SG_REL"))) || is_captcha_detected {
            if let Some(rendered_html) = crate::dom::screenshot::RealBrowserScreenshot::dump_rendered_dom(url).await {
                html = rendered_html;
                status = 200;
                final_url = url.to_string();
                is_captcha_detected = false;
            }
        }

        Ok(FetchResult {
            status,
            final_url,
            html,
            is_captcha_detected,
        })
    }
}
