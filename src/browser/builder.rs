use crate::browser::engine::BrowserEngine;
use crate::browser::tab::BrowserTab;
use crate::network::client::NetworkClient;
use crate::network::fingerprint::DeviceProfile;
use anyhow::Result;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct BrowserBuilder {
    pub profile: DeviceProfile,
    pub proxy_url: Option<String>,
    pub timeout: Duration,
    pub custom_user_agent: Option<String>,
    pub max_redirects: usize,
}

impl Default for BrowserBuilder {
    fn default() -> Self {
        Self {
            profile: DeviceProfile::ChromeWindows,
            proxy_url: None,
            timeout: Duration::from_secs(30),
            custom_user_agent: None,
            max_redirects: 10,
        }
    }
}

impl BrowserBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn profile(mut self, profile: DeviceProfile) -> Self {
        self.profile = profile;
        self
    }

    pub fn proxy<S: Into<String>>(mut self, proxy_url: S) -> Self {
        self.proxy_url = Some(proxy_url.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn custom_user_agent<S: Into<String>>(mut self, ua: S) -> Self {
        self.custom_user_agent = Some(ua.into());
        self
    }

    pub fn max_redirects(mut self, redirects: usize) -> Self {
        self.max_redirects = redirects;
        self
    }

    pub fn build(self) -> Result<BrowserTab> {
        let network = NetworkClient::with_builder_config(
            self.profile,
            self.proxy_url.as_deref(),
            self.timeout,
            self.max_redirects,
            self.custom_user_agent.as_deref(),
        )?;
        BrowserTab::from_network(network)
    }

    pub fn build_engine(self) -> Result<BrowserEngine> {
        BrowserEngine::with_builder(self)
    }
}
