use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, DNT,
    UPGRADE_INSECURE_REQUESTS, USER_AGENT,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeviceProfile {
    #[default]
    ChromeWindows,
    ChromeLinux,
    SafariMac,
    SafariIos,
    ChromeAndroid,
}

pub struct Fingerprint {
    pub profile: DeviceProfile,
    pub user_agent: &'static str,
    pub platform: &'static str,
    pub screen_width: u32,
    pub screen_height: u32,
    pub is_mobile: bool,
}

impl Fingerprint {
    pub fn for_profile(profile: DeviceProfile) -> Self {
        match profile {
            DeviceProfile::ChromeWindows => Self {
                profile,
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
                platform: "Win32",
                screen_width: 1920,
                screen_height: 1080,
                is_mobile: false,
            },
            DeviceProfile::ChromeLinux => Self {
                profile,
                user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
                platform: "Linux x86_64",
                screen_width: 1920,
                screen_height: 1080,
                is_mobile: false,
            },
            DeviceProfile::SafariMac => Self {
                profile,
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3 Safari/605.1.15",
                platform: "MacIntel",
                screen_width: 2560,
                screen_height: 1440,
                is_mobile: false,
            },
            DeviceProfile::SafariIos => Self {
                profile,
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 18_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3 Mobile/15E148 Safari/604.1",
                platform: "iPhone",
                screen_width: 393,
                screen_height: 852,
                is_mobile: true,
            },
            DeviceProfile::ChromeAndroid => Self {
                profile,
                user_agent: "Mozilla/5.0 (Linux; Android 14; Pixel 8 Build/UD1A.230803.022) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.6943.122 Mobile Safari/537.36",
                platform: "Linux armv8l",
                screen_width: 412,
                screen_height: 915,
                is_mobile: true,
            },
        }
    }

    pub fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_static(self.user_agent));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
            ),
        );
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert(
            ACCEPT_ENCODING,
            HeaderValue::from_static("gzip, deflate, br"),
        );
        headers.insert(UPGRADE_INSECURE_REQUESTS, HeaderValue::from_static("1"));
        headers.insert(DNT, HeaderValue::from_static("1"));

        match self.profile {
            DeviceProfile::ChromeWindows => {
                headers.insert(
                    "sec-ch-ua",
                    HeaderValue::from_static(
                        "\"Not(A:Brand\";v=\"99\", \"Google Chrome\";v=\"133\", \"Chromium\";v=\"133\"",
                    ),
                );
                headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
                headers.insert(
                    "sec-ch-ua-platform",
                    HeaderValue::from_static("\"Windows\""),
                );
                headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
                headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
                headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
                headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
            }
            DeviceProfile::ChromeLinux => {
                headers.insert(
                    "sec-ch-ua",
                    HeaderValue::from_static(
                        "\"Not(A:Brand\";v=\"99\", \"Google Chrome\";v=\"133\", \"Chromium\";v=\"133\"",
                    ),
                );
                headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
                headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Linux\""));
                headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
                headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
                headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
                headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
            }
            DeviceProfile::ChromeAndroid => {
                headers.insert(
                    "sec-ch-ua",
                    HeaderValue::from_static(
                        "\"Not(A:Brand\";v=\"99\", \"Google Chrome\";v=\"133\", \"Chromium\";v=\"133\"",
                    ),
                );
                headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?1"));
                headers.insert(
                    "sec-ch-ua-platform",
                    HeaderValue::from_static("\"Android\""),
                );
                headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
                headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
                headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
                headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
            }
            DeviceProfile::SafariMac | DeviceProfile::SafariIos => {
                // Safari does not send Sec-CH-UA headers
            }
        }

        headers
    }
}
