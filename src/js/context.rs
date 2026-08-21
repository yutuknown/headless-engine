use crate::network::fingerprint::{DeviceProfile, Fingerprint};
use anyhow::Result;
use boa_engine::{Context, Source};

pub struct JsRuntime {
    context: Context<'static>,
}

impl JsRuntime {
    pub fn new() -> Result<Self> {
        let default_fp = Fingerprint::for_profile(DeviceProfile::ChromeWindows);
        Self::with_fingerprint(&default_fp)
    }

    pub fn with_fingerprint(fp: &Fingerprint) -> Result<Self> {
        let mut context = Context::default();

        let max_touch_points = if fp.is_mobile { 5 } else { 0 };
        let dpr = if fp.is_mobile { "3.0" } else { "1.0" };
        let mobile_bool = if fp.is_mobile { "true" } else { "false" };

        let (gpu_vendor, gpu_renderer) = match fp.profile {
            DeviceProfile::ChromeWindows => (
                "Google Inc. (NVIDIA)",
                "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, D3D11)",
            ),
            DeviceProfile::ChromeLinux => (
                "Google Inc. (Intel)",
                "ANGLE (Intel, Mesa Intel(R) UHD Graphics 620 (KBL GT2), OpenGL 4.6)",
            ),
            DeviceProfile::SafariMac => ("Apple Inc.", "Apple M2 Pro"),
            DeviceProfile::SafariIos => ("Apple Inc.", "Apple A17 Pro GPU"),
            DeviceProfile::ChromeAndroid => ("ARM", "Mali-G715-Immortalis MC11"),
        };

        let init_script = format!(
            r###"
            // 1. Core Window & Global References
            globalThis.window = globalThis;
            globalThis.self = globalThis;
            globalThis.top = globalThis;
            globalThis.parent = globalThis;
            globalThis.devicePixelRatio = {dpr};

            // 2. Storage APIs (localStorage & sessionStorage)
            function createStorageMock() {{
                const store = new Map();
                return {{
                    getItem: function(k) {{ return store.has(String(k)) ? store.get(String(k)) : null; }},
                    setItem: function(k, v) {{ store.set(String(k), String(v)); }},
                    removeItem: function(k) {{ store.delete(String(k)); }},
                    clear: function() {{ store.clear(); }},
                    key: function(i) {{ const keys = Array.from(store.keys()); return keys[i] || null; }},
                    get length() {{ return store.size; }}
                }};
            }}
            globalThis.localStorage = createStorageMock();
            globalThis.sessionStorage = createStorageMock();
            globalThis.window.localStorage = globalThis.localStorage;
            globalThis.window.sessionStorage = globalThis.sessionStorage;

            // 3. IndexedDB API
            globalThis.indexedDB = {{
                open: function(name, ver) {{
                    return {{
                        result: {{ name: name, version: ver || 1, objectStoreNames: [] }},
                        error: null,
                        readyState: "done",
                        onsuccess: null,
                        onerror: null,
                        onupgradeneeded: null
                    }};
                }},
                databases: function() {{ return Promise.resolve([]); }},
                deleteDatabase: function() {{ return {{ onsuccess: null, onerror: null }}; }},
                cmp: function(a, b) {{ return a < b ? -1 : (a > b ? 1 : 0); }}
            }};
            globalThis.window.indexedDB = globalThis.indexedDB;

            // 4. Deep Navigator Stealth Emulation
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
                connection: {{
                    downlink: 10,
                    effectiveType: "4g",
                    rtt: 50,
                    saveData: false,
                    onchange: null
                }},
                mediaDevices: {{
                    enumerateDevices: function() {{
                        return Promise.resolve([
                            {{ deviceId: "default", kind: "audioinput", label: "Default - Microphone (Realtek Audio)", groupId: "audio-group-1" }},
                            {{ deviceId: "default", kind: "audiooutput", label: "Default - Speakers (Realtek Audio)", groupId: "audio-group-1" }},
                            {{ deviceId: "cam-01", kind: "videoinput", label: "Integrated Camera (HD Webcam)", groupId: "video-group-1" }}
                        ]);
                    }},
                    getUserMedia: function() {{ return Promise.reject(new Error("Permission denied")); }},
                    getDisplayMedia: function() {{ return Promise.reject(new Error("Permission denied")); }}
                }},
                getBattery: function() {{
                    return Promise.resolve({{
                        charging: true,
                        chargingTime: 0,
                        dischargingTime: Infinity,
                        level: 1.0,
                        onchargingchange: null,
                        onlevelchange: null
                    }});
                }},
                serviceWorker: {{
                    controller: null,
                    ready: Promise.resolve({{ active: null, scope: "/" }}),
                    register: function() {{ return Promise.resolve(); }},
                    getRegistration: function() {{ return Promise.resolve(null); }},
                    getRegistrations: function() {{ return Promise.resolve([]); }}
                }},
                credentials: {{
                    get: function() {{ return Promise.resolve(null); }},
                    create: function() {{ return Promise.resolve(null); }},
                    store: function() {{ return Promise.resolve(); }},
                    preventSilentAccess: function() {{ return Promise.resolve(); }}
                }},
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

            // 5. Complete window.chrome Emulation
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

            // 6. Screen and Display Metrics
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

            // 7. Notification API
            globalThis.Notification = {{
                permission: "default",
                maxActions: 2,
                requestPermission: function() {{
                    return Promise.resolve("default");
                }}
            }};

            // 8. WebGL Vendor & Renderer Spoofing
            globalThis.WebGLRenderingContext = function() {{}};
            globalThis.WebGLRenderingContext.prototype = {{
                getParameter: function(param) {{
                    if (param === 37445) return "{gpu_vendor}";
                    if (param === 37446) return "{gpu_renderer}";
                    if (param === 7936) return "WebKit";
                    if (param === 7937) return "WebKit WebGL";
                    return null;
                }},
                getSupportedExtensions: function() {{
                    return [
                        "ANGLE_instanced_arrays",
                        "EXT_blend_minmax",
                        "EXT_color_buffer_half_float",
                        "EXT_float_blend",
                        "EXT_frag_depth",
                        "EXT_shader_texture_lod",
                        "EXT_sRGB",
                        "EXT_texture_compression_bptc",
                        "EXT_texture_compression_rgtc",
                        "EXT_texture_filter_anisotropic",
                        "OES_element_index_uint",
                        "OES_fbo_render_mipmap",
                        "OES_standard_derivatives",
                        "OES_texture_float",
                        "OES_texture_float_linear",
                        "OES_texture_half_float",
                        "OES_texture_half_float_linear",
                        "OES_vertex_array_object",
                        "WEBGL_color_buffer_float",
                        "WEBGL_compressed_texture_s3tc",
                        "WEBGL_compressed_texture_s3tc_srgb",
                        "WEBGL_debug_renderer_info",
                        "WEBGL_debug_shaders",
                        "WEBGL_depth_texture",
                        "WEBGL_draw_buffers",
                        "WEBGL_lose_context",
                        "WEBGL_multi_draw"
                    ];
                }},
                getExtension: function(name) {{
                    if (name === "WEBGL_debug_renderer_info") {{
                        return {{
                            UNMASKED_VENDOR_WEBGL: 37445,
                            UNMASKED_RENDERER_WEBGL: 37446
                        }};
                    }}
                    return {{}};
                }}
            }};
            globalThis.WebGL2RenderingContext = globalThis.WebGLRenderingContext;

            // 9. Web Audio API
            function AudioContextMock() {{
                return {{
                    state: "running",
                    sampleRate: 44100,
                    currentTime: 0.1,
                    destination: {{ maxChannelCount: 2, channelCount: 2, channelCountMode: "explicit" }},
                    createOscillator: function() {{
                        return {{
                            type: "sine",
                            frequency: {{ value: 440, setValueAtTime: function() {{}} }},
                            connect: function() {{}},
                            start: function() {{}},
                            stop: function() {{}}
                        }};
                    }},
                    createGain: function() {{
                        return {{ gain: {{ value: 1.0, setValueAtTime: function() {{}} }}, connect: function() {{}} }};
                    }},
                    createDynamicsCompressor: function() {{
                        return {{
                            threshold: {{ value: -24 }},
                            knee: {{ value: 30 }},
                            ratio: {{ value: 12 }},
                            reduction: -10,
                            attack: {{ value: 0.003 }},
                            release: {{ value: 0.25 }},
                            connect: function() {{}}
                        }};
                    }},
                    createBufferSource: function() {{ return {{ connect: function() {{}}, start: function() {{}}, stop: function() {{}} }}; }},
                    createAnalyser: function() {{
                        return {{
                            fftSize: 2048,
                            frequencyBinCount: 1024,
                            minDecibels: -100,
                            maxDecibels: -30,
                            smoothingTimeConstant: 0.8,
                            getByteFrequencyData: function(arr) {{ if (arr && arr.fill) arr.fill(128); }},
                            getFloatFrequencyData: function(arr) {{ if (arr && arr.fill) arr.fill(-50.0); }}
                        }};
                    }}
                }};
            }}
            globalThis.AudioContext = AudioContextMock;
            globalThis.webkitAudioContext = AudioContextMock;
            globalThis.OfflineAudioContext = function(ch, len, sr) {{
                const ctx = AudioContextMock();
                ctx.startRendering = function() {{
                    return Promise.resolve({{
                        length: len,
                        duration: len / (sr || 44100),
                        sampleRate: sr || 44100,
                        numberOfChannels: ch || 2,
                        getChannelData: function(c) {{
                            const d = new Float32Array(len || 100);
                            for (let i = 0; i < d.length; i++) {{
                                d[i] = Math.sin(i * 0.05) * 0.5 + 0.0001 * Math.sin(i * 1.5);
                            }}
                            return d;
                        }}
                    }});
                }};
                return ctx;
            }};

            // 10. Performance API
            const _startTime = Date.now() - 320;
            globalThis.performance = {{
                now: function() {{ return Date.now() - _startTime; }},
                timeOrigin: _startTime,
                timing: {{
                    navigationStart: _startTime,
                    unloadEventStart: 0,
                    unloadEventEnd: 0,
                    redirectStart: 0,
                    redirectEnd: 0,
                    fetchStart: _startTime + 5,
                    domainLookupStart: _startTime + 12,
                    domainLookupEnd: _startTime + 25,
                    connectStart: _startTime + 25,
                    connectEnd: _startTime + 75,
                    secureConnectionStart: _startTime + 40,
                    requestStart: _startTime + 76,
                    responseStart: _startTime + 140,
                    responseEnd: _startTime + 185,
                    domLoading: _startTime + 190,
                    domInteractive: _startTime + 280,
                    domContentLoadedEventStart: _startTime + 290,
                    domContentLoadedEventEnd: _startTime + 295,
                    domComplete: _startTime + 310,
                    loadEventStart: _startTime + 315,
                    loadEventEnd: _startTime + 320
                }},
                navigation: {{
                    type: 0,
                    redirectCount: 0
                }},
                memory: {{
                    jsHeapSizeLimit: 4294705152,
                    totalJSHeapSize: 58410652,
                    usedJSHeapSize: 42892110
                }},
                getEntriesByType: function(type) {{
                    if (type === "navigation") {{
                        return [{{
                            name: globalThis.location ? globalThis.location.href : "https://example.com",
                            entryType: "navigation",
                            startTime: 0,
                            duration: 320,
                            initiatorType: "navigation",
                            nextHopProtocol: "h2",
                            renderBlockingStatus: "non-blocking",
                            responseStatus: 200
                        }}];
                    }}
                    return [];
                }},
                getEntriesByName: function() {{ return []; }},
                getEntries: function() {{ return []; }}
            }};
            globalThis.window.performance = globalThis.performance;

            // 11. CSS Media & Animation APIs
            globalThis.matchMedia = function(query) {{
                return {{
                    matches: query.includes("prefers-color-scheme") || query.includes("screen"),
                    media: query,
                    onchange: null,
                    addListener: function() {{}},
                    removeListener: function() {{}},
                    addEventListener: function() {{}},
                    removeEventListener: function() {{}},
                    dispatchEvent: function() {{ return false; }}
                }};
            }};
            globalThis.window.matchMedia = globalThis.matchMedia;
            globalThis.requestAnimationFrame = function(cb) {{ return setTimeout(cb, 16); }};
            globalThis.cancelAnimationFrame = function(id) {{ clearTimeout(id); }};

            // 12. Document & Location Defaults
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
                    const tagUpper = String(tag).toUpperCase();
                    if (tagUpper === "CANVAS") {{
                        return {{
                            tagName: "CANVAS",
                            width: 300,
                            height: 150,
                            getContext: function(type) {{
                                if (type === "2d") {{
                                    return {{
                                        fillStyle: "#000000",
                                        strokeStyle: "#000000",
                                        font: "10px sans_serif",
                                        fillRect: function() {{}},
                                        strokeRect: function() {{}},
                                        clearRect: function() {{}},
                                        beginPath: function() {{}},
                                        closePath: function() {{}},
                                        moveTo: function() {{}},
                                        lineTo: function() {{}},
                                        arc: function() {{}},
                                        fill: function() {{}},
                                        stroke: function() {{}},
                                        fillText: function() {{}},
                                        strokeText: function() {{}},
                                        measureText: function(text) {{
                                            return {{
                                                width: String(text).length * 7.5 + 0.02,
                                                actualBoundingBoxAscent: 8,
                                                actualBoundingBoxDescent: 2,
                                                fontBoundingBoxAscent: 10,
                                                fontBoundingBoxDescent: 3
                                            }};
                                        }},
                                        getImageData: function(sx, sy, sw, sh) {{
                                            const data = new Uint8ClampedArray((sw || 16) * (sh || 16) * 4);
                                            for (let i = 0; i < data.length; i += 4) {{
                                                data[i] = (i * 3 + 120) % 256;
                                                data[i + 1] = (i * 7 + 80) % 256;
                                                data[i + 2] = (i * 11 + 200) % 256;
                                                data[i + 3] = 255;
                                            }}
                                            return {{ data: data, width: sw || 16, height: sh || 16 }};
                                        }}
                                    }};
                                }}
                                return new globalThis.WebGLRenderingContext();
                            }},
                            toDataURL: function() {{
                                return "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAASwAAACWCAYAAAB5DiTlAAA=";
                            }},
                            style: {{}}
                        }};
                    }}
                    return {{
                        tagName: tagUpper,
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
            "###,
            ua = fp.user_agent,
            platform = fp.platform,
            touch = max_touch_points,
            dpr = dpr,
            is_mobile = mobile_bool,
            sw = fp.screen_width,
            sh = fp.screen_height,
            gpu_vendor = gpu_vendor,
            gpu_renderer = gpu_renderer
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
