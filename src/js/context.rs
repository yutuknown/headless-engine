use crate::network::fingerprint::Fingerprint;
use anyhow::Result;
use boa_engine::{Context, Source};

pub struct JsRuntime {
    context: Context<'static>,
}

impl JsRuntime {
    pub fn new() -> Result<Self> {
        let default_fp =
            Fingerprint::for_profile(crate::network::fingerprint::DeviceProfile::ChromeWindows);
        Self::with_fingerprint(&default_fp)
    }

    pub fn with_fingerprint(fp: &Fingerprint) -> Result<Self> {
        let mut context = Context::default();

        let max_touch_points = if fp.is_mobile { 5 } else { 0 };
        let dpr = if fp.is_mobile { "3.0" } else { "1.0" };
        let mobile_bool = if fp.is_mobile { "true" } else { "false" };

        let init_script = format!(
            r#"
            // 1. Core Window & Global References
            globalThis.window = globalThis;
            globalThis.self = globalThis;
            globalThis.top = globalThis;
            globalThis.parent = globalThis;
            globalThis.devicePixelRatio = {dpr};

            // 2. Deep Navigator Stealth Emulation
            const pluginArray = [
                {{
                    name: "Chrome PDF Plugin",
                    filename: "internal-pdf-viewer",
                    description: "Portable Document Format",
                    length: 1,
                    0: {{
                        type: "application/x-google-chrome-pdf",
                        suffixes: "pdf",
                        description: "Portable Document Format",
                        enabledPlugin: null
                    }}
                }},
                {{
                    name: "Chrome PDF Viewer",
                    filename: "mhjfbmdgcfjbbpaeojofohoefgiehjai",
                    description: "",
                    length: 1,
                    0: {{
                        type: "application/pdf",
                        suffixes: "pdf",
                        description: "",
                        enabledPlugin: null
                    }}
                }},
                {{
                    name: "Native Client",
                    filename: "internal-nacl-plugin",
                    description: "",
                    length: 2,
                    0: {{
                        type: "application/x-nacl",
                        suffixes: "",
                        description: "Native Client Executable",
                        enabledPlugin: null
                    }},
                    1: {{
                        type: "application/x-pnacl",
                        suffixes: "",
                        description: "Portable Native Client Executable",
                        enabledPlugin: null
                    }}
                }}
            ];
            pluginArray.item = function(index) {{ return this[index] || null; }};
            pluginArray.namedItem = function(name) {{
                for (let i = 0; i < this.length; i++) {{
                    if (this[i].name === name) return this[i];
                }}
                return null;
            }};
            pluginArray.refresh = function() {{}};

            const mimeTypeArray = [
                {{
                    type: "application/pdf",
                    suffixes: "pdf",
                    description: "",
                    enabledPlugin: pluginArray[1]
                }},
                {{
                    type: "application/x-google-chrome-pdf",
                    suffixes: "pdf",
                    description: "Portable Document Format",
                    enabledPlugin: pluginArray[0]
                }},
                {{
                    type: "application/x-nacl",
                    suffixes: "",
                    description: "Native Client Executable",
                    enabledPlugin: pluginArray[2]
                }},
                {{
                    type: "application/x-pnacl",
                    suffixes: "",
                    description: "Portable Native Client Executable",
                    enabledPlugin: pluginArray[2]
                }}
            ];
            mimeTypeArray.item = function(index) {{ return this[index] || null; }};
            mimeTypeArray.namedItem = function(type) {{
                for (let i = 0; i < this.length; i++) {{
                    if (this[i].type === type) return this[i];
                }}
                return null;
            }};

            globalThis.navigator = {{
                userAgent: "{ua}",
                appVersion: "{ua}",
                platform: "{platform}",
                appName: "Netscape",
                appCodeName: "Mozilla",
                language: "en-US",
                languages: ["en-US", "en"],
                webdriver: false,
                cookieEnabled: true,
                hardwareConcurrency: 8,
                deviceMemory: 8,
                maxTouchPoints: {touch},
                vendor: "Google Inc.",
                vendorSub: "",
                product: "Gecko",
                productSub: "20030107",
                plugins: pluginArray,
                mimeTypes: mimeTypeArray,
                doNotTrack: null,
                userAgentData: {{
                    brands: [
                        {{ brand: "Not(A:Brand", version: "99" }},
                        {{ brand: "Google Chrome", version: "133" }},
                        {{ brand: "Chromium", version: "133" }}
                    ],
                    mobile: {is_mobile},
                    platform: "{platform}",
                    getHighEntropyValues: function(hints) {{
                        return Promise.resolve({{
                            architecture: "x86",
                            bitness: "64",
                            brands: [
                                {{ brand: "Not(A:Brand", version: "99" }},
                                {{ brand: "Google Chrome", version: "133" }},
                                {{ brand: "Chromium", version: "133" }}
                            ],
                            mobile: {is_mobile},
                            model: "",
                            platform: "{platform}",
                            platformVersion: "15.0.0",
                            uaFullVersion: "133.0.6943.127"
                        }});
                    }}
                }},
                permissions: {{
                    query: function(param) {{
                        return Promise.resolve({{
                            state: "default",
                            onchange: null
                        }});
                    }}
                }}
            }};

            // 3. Complete window.chrome Emulation
            globalThis.window.chrome = {{
                app: {{
                    isInstalled: false,
                    InstallState: {{
                        DISABLED: "disabled",
                        INSTALLED: "installed",
                        NOT_INSTALLED: "not_installed"
                    }},
                    RunningState: {{
                        CANNOT_RUN: "cannot_run",
                        READY_TO_RUN: "ready_to_run",
                        RUNNING: "running"
                    }},
                    getDetails: function() {{ return null; }},
                    getIsInstalled: function() {{ return false; }},
                    runningState: function() {{ return "cannot_run"; }}
                }},
                runtime: {{
                    OnInstalledReason: {{
                        CHROME_UPDATE: "chrome_update",
                        INSTALL: "install",
                        SHARED_MODULE_UPDATE: "shared_module_update",
                        UPDATE: "update"
                    }},
                    OnRestartRequiredReason: {{
                        APP_UPDATE: "app_update",
                        OS_UPDATE: "os_update",
                        PERIODIC: "periodic"
                    }},
                    PlatformArch: {{
                        ARM: "arm",
                        ARM64: "arm64",
                        MIPS: "mips",
                        MIPS64: "mips64",
                        X86_32: "x86-32",
                        X86_64: "x86-64"
                    }},
                    PlatformNaclArch: {{
                        ARM: "arm",
                        MIPS: "mips",
                        MIPS64: "mips64",
                        X86_32: "x86-32",
                        X86_64: "x86-64"
                    }},
                    PlatformOs: {{
                        ANDROID: "android",
                        CROS: "cros",
                        LINUX: "linux",
                        MAC: "mac",
                        OPENBSD: "openbsd",
                        WIN: "win"
                    }},
                    RequestUpdateCheckStatus: {{
                        NO_UPDATE: "no_update",
                        THROTTLED: "throttled",
                        UPDATE_AVAILABLE: "update_available"
                    }},
                    connect: function() {{}},
                    sendMessage: function() {{}}
                }},
                csi: function() {{
                    return {{
                        onloadT: Date.now(),
                        pageT: 142.5,
                        startE: Date.now() - 150,
                        tran: 15
                    }};
                }},
                loadTimes: function() {{
                    return {{
                        commitLoadTime: Date.now() / 1000 - 0.1,
                        connectionInfo: "h2",
                        finishDocumentLoadTime: Date.now() / 1000,
                        finishLoadTime: Date.now() / 1000,
                        firstPaintAfterLoadTime: 0,
                        firstPaintTime: Date.now() / 1000 - 0.05,
                        navigationType: "Other",
                        npnNegotiatedProtocol: "h2",
                        requestTime: Date.now() / 1000 - 0.15,
                        startLoadTime: Date.now() / 1000 - 0.15,
                        wasAlternateProtocolAvailable: false,
                        wasFetchedViaSpdy: true,
                        wasNpnNegotiated: true
                    }};
                }}
            }};

            // 4. Screen and Display Metrics
            globalThis.screen = {{
                width: {sw},
                height: {sh},
                availWidth: {sw},
                availHeight: {sh},
                colorDepth: 24,
                pixelDepth: 24,
                availLeft: 0,
                availTop: 0,
                orientation: {{
                    angle: 0,
                    type: "landscape-primary",
                    onchange: null
                }}
            }};
            globalThis.outerWidth = {sw};
            globalThis.outerHeight = {sh};
            globalThis.innerWidth = {sw};
            globalThis.innerHeight = {sh};

            // 5. Notification API
            globalThis.Notification = {{
                permission: "default",
                maxActions: 2,
                requestPermission: function() {{
                    return Promise.resolve("default");
                }}
            }};

            // 6. WebGL Vendor & Renderer Spoofing
            globalThis.WebGLRenderingContext = function() {{}};
            globalThis.WebGLRenderingContext.prototype = {{
                getParameter: function(param) {{
                    // UNMASKED_VENDOR_WEBGL
                    if (param === 37445) return "Google Inc. (NVIDIA)";
                    // UNMASKED_RENDERER_WEBGL
                    if (param === 37446) return "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, D3D11)";
                    // VENDOR
                    if (param === 7936) return "WebKit";
                    // RENDERER
                    if (param === 7937) return "WebKit WebGL";
                    return null;
                }}
            }};
            globalThis.WebGL2RenderingContext = globalThis.WebGLRenderingContext;

            // 7. Document & Location Defaults
            globalThis.document = {{
                title: "",
                cookie: "",
                referrer: "",
                readyState: "complete",
                characterSet: "UTF-8",
                compatMode: "CSS1Compat",
                location: {{
                    href: ""
                }},
                getElementById: function(id) {{ return null; }},
                getElementsByTagName: function(tag) {{ return []; }},
                querySelector: function(sel) {{ return null; }},
                querySelectorAll: function(sel) {{ return []; }},
                createElement: function(tag) {{
                    return {{
                        tagName: tag.toUpperCase(),
                        setAttribute: function() {{}},
                        getAttribute: function() {{ return null; }},
                        appendChild: function() {{}},
                        style: {{}}
                    }};
                }}
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
            dpr = dpr,
            is_mobile = mobile_bool,
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
