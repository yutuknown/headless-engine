use crate::network::fingerprint::Fingerprint;
use anyhow::Result;
use boa_engine::{Context, Source};

pub struct JsRuntime {
    context: Context<'static>,
}

impl JsRuntime {
    pub fn new() -> Result<Self> {
        let default_fp = Fingerprint::for_profile(crate::network::fingerprint::DeviceProfile::ChromeWindows);
        Self::with_fingerprint(&default_fp)
    }

    pub fn with_fingerprint(fp: &Fingerprint) -> Result<Self> {
        let mut context = Context::default();

        let max_touch_points = if fp.is_mobile { 5 } else { 0 };

        let init_script = format!(
            r#"
            globalThis.window = globalThis;
            globalThis.self = globalThis;
            globalThis.top = globalThis;

            globalThis.navigator = {{
                userAgent: "{ua}",
                appVersion: "{ua}",
                platform: "{platform}",
                language: "en-US",
                languages: ["en-US", "en"],
                webdriver: false,
                cookieEnabled: true,
                hardwareConcurrency: 8,
                deviceMemory: 8,
                vendor: "Google Inc.",
                vendorSub: "",
                product: "Gecko",
                productSub: "20030107",
                maxTouchPoints: {touch}
            }};

            globalThis.window.chrome = {{
                runtime: {{}},
                loadTimes: function() {{}},
                csi: function() {{}},
                app: {{}}
            }};

            globalThis.screen = {{
                width: {sw},
                height: {sh},
                availWidth: {sw},
                availHeight: {sh},
                colorDepth: 24,
                pixelDepth: 24
            }};

            globalThis.document = {{
                title: "",
                cookie: "",
                referrer: "",
                readyState: "complete",
                location: {{
                    href: ""
                }},
                getElementById: function(id) {{ return null; }},
                getElementsByTagName: function(tag) {{ return []; }},
                querySelector: function(sel) {{ return null; }},
                querySelectorAll: function(sel) {{ return []; }}
            }};

            globalThis.location = {{
                href: "",
                origin: "",
                protocol: "https:",
                host: "",
                hostname: "",
                pathname: "/",
                search: ""
            }};
            "#,
            ua = fp.user_agent,
            platform = fp.platform,
            touch = max_touch_points,
            sw = fp.screen_width,
            sh = fp.screen_height
        );

        context
            .eval(Source::from_bytes(&init_script))
            .map_err(|e| anyhow::anyhow!("Failed to initialize JS runtime: {}", e))?;

        Ok(Self { context })
    }

    pub fn update_page_state(&mut self, url: &str, title: &str) -> Result<()> {
        let escaped_url = url.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_title = title.replace('\\', "\\\\").replace('"', "\\\"");

        let update_script = format!(
            r#"
            document.title = "{}";
            document.location.href = "{}";
            location.href = "{}";
            "#,
            escaped_title, escaped_url, escaped_url
        );

        self.context
            .eval(Source::from_bytes(&update_script))
            .map_err(|e| anyhow::anyhow!("Failed to update JS page state: {}", e))?;

        Ok(())
    }

    pub fn evaluate(&mut self, code: &str) -> Result<String> {
        match self.context.eval(Source::from_bytes(code)) {
            Ok(res) => Ok(res.display().to_string()),
            Err(err) => Err(anyhow::anyhow!("JS Eval error: {}", err)),
        }
    }
}
