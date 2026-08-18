use crate::browser::engine::BrowserEngine;
use crate::network::fingerprint::DeviceProfile;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RpcRequest {
    pub jsonrpc: Option<String>,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl RpcResponse {
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<serde_json::Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
            }),
        }
    }
}

pub struct JsonRpcHandler {
    pub engine: BrowserEngine,
}

impl JsonRpcHandler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            engine: BrowserEngine::new()?,
        })
    }

    pub async fn handle_line(&mut self, line: &str) -> RpcResponse {
        let req: RpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => return RpcResponse::error(None, -32700, format!("Parse error: {}", e)),
        };

        self.handle_request(req).await
    }

    pub async fn handle_request(&mut self, req: RpcRequest) -> RpcResponse {
        let id = req.id.clone();
        let method = req.method.as_str();
        let params = &req.params;

        match method {
            // Engine Methods
            "engine.createTab" | "createTab" => {
                let profile: Option<DeviceProfile> = params
                    .get("profile")
                    .and_then(|p| serde_json::from_value(p.clone()).ok());
                match self.engine.create_tab(profile) {
                    Ok(tab_id) => RpcResponse::success(id, serde_json::json!({ "tab_id": tab_id })),
                    Err(e) => RpcResponse::error(id, -32000, format!("CreateTab error: {}", e)),
                }
            }
            "engine.closeTab" | "closeTab" => {
                let tab_id = params.get("tab_id").and_then(|v| v.as_str()).unwrap_or("");
                let closed = self.engine.close_tab(tab_id);
                RpcResponse::success(id, serde_json::json!({ "closed": closed }))
            }
            "engine.listTabs" | "listTabs" => {
                let tabs = self.engine.list_tabs();
                RpcResponse::success(id, serde_json::json!({ "tabs": tabs, "count": tabs.len() }))
            }

            // Tab Methods (with optional tab_id)
            "tab.navigate" | "navigate" | "Navigate" => {
                let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("");
                if url.is_empty() {
                    return RpcResponse::error(id, -32602, "Missing 'url' parameter");
                }
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.navigate(url).await {
                    Ok(report) => match serde_json::to_value(report) {
                        Ok(v) => RpcResponse::success(id, v),
                        Err(e) => {
                            RpcResponse::error(id, -32000, format!("Serialization failed: {}", e))
                        }
                    },
                    Err(e) => RpcResponse::error(id, -32000, format!("Navigation failed: {}", e)),
                }
            }
            "tab.observe" | "observe" | "Observe" => {
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.observe() {
                    Some(obs) => match serde_json::to_value(obs) {
                        Ok(v) => RpcResponse::success(id, v),
                        Err(e) => {
                            RpcResponse::error(id, -32000, format!("Serialization failed: {}", e))
                        }
                    },
                    None => RpcResponse::error(id, -32000, "No page loaded. Call navigate first."),
                }
            }
            "tab.extractInteractive" | "extractInteractive" => {
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                let elements = tab.extract_interactive_elements();
                RpcResponse::success(
                    id,
                    serde_json::json!({ "elements": elements, "count": elements.len() }),
                )
            }
            "tab.screenshot" | "screenshot" | "Screenshot" => {
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.screenshot_async().await {
                    Some(shot) => match serde_json::to_value(shot) {
                        Ok(v) => RpcResponse::success(id, v),
                        Err(e) => {
                            RpcResponse::error(id, -32000, format!("Serialization failed: {}", e))
                        }
                    },
                    None => RpcResponse::error(id, -32000, "No page loaded. Call navigate first."),
                }
            }
            "tab.screenshotLayout" | "screenshotLayout" => {
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.screenshot_layout() {
                    Some(layout) => {
                        RpcResponse::success(id, serde_json::json!({ "layout": layout }))
                    }
                    None => RpcResponse::error(id, -32000, "No page loaded. Call navigate first."),
                }
            }
            "tab.screenshotSvg" | "screenshotSvg" => {
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.screenshot_svg() {
                    Some(svg) => RpcResponse::success(id, serde_json::json!({ "svg": svg })),
                    None => RpcResponse::error(id, -32000, "No page loaded. Call navigate first."),
                }
            }
            "tab.extractMarkdown" | "extractMarkdown" | "ExtractMarkdown" => {
                let selector = params.get("selector").and_then(|v| v.as_str());
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.extract_markdown(selector) {
                    Some(markdown) => RpcResponse::success(
                        id,
                        serde_json::json!({ "markdown": markdown, "length": markdown.len() }),
                    ),
                    None => RpcResponse::error(id, -32000, "No DOM loaded. Call navigate first."),
                }
            }
            "tab.extractResults" | "extractResults" | "ExtractResults" => {
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.extract_search_results() {
                    Some(results) => match serde_json::to_value(results) {
                        Ok(v) => RpcResponse::success(id, v),
                        Err(e) => {
                            RpcResponse::error(id, -32000, format!("Serialization failed: {}", e))
                        }
                    },
                    None => RpcResponse::error(
                        id,
                        -32000,
                        "No search results available. Call navigate first.",
                    ),
                }
            }
            "tab.extractLinks" | "extractLinks" | "ExtractLinks" => {
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                let links = tab.extract_links();
                RpcResponse::success(
                    id,
                    serde_json::json!({ "links": links, "count": links.len() }),
                )
            }
            "tab.extractForms" | "extractForms" | "ExtractForms" => {
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                let forms = tab.extract_forms();
                RpcResponse::success(
                    id,
                    serde_json::json!({ "forms": forms, "count": forms.len() }),
                )
            }
            "tab.extractDom" | "extractDom" | "ExtractDom" => {
                let selector = params.get("selector").and_then(|v| v.as_str());
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.extract_dom(selector) {
                    Some(html) => RpcResponse::success(
                        id,
                        serde_json::json!({ "html": html, "length": html.len() }),
                    ),
                    None => RpcResponse::error(id, -32000, "No DOM loaded. Call navigate first."),
                }
            }
            "tab.click" | "click" | "Click" | "tab.actClick" => {
                let target = params
                    .get("target")
                    .or_else(|| params.get("index"))
                    .map(|v| {
                        if v.is_number() {
                            v.to_string()
                        } else {
                            v.as_str().unwrap_or("").to_string()
                        }
                    })
                    .unwrap_or_default();
                if target.is_empty() {
                    return RpcResponse::error(id, -32602, "Missing 'target' or 'index' parameter");
                }
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.act_click(&target).await {
                    Ok(Some(report)) => RpcResponse::success(
                        id,
                        serde_json::json!({ "action": "navigated", "report": report }),
                    ),
                    Ok(None) => {
                        RpcResponse::success(id, serde_json::json!({ "action": "clicked_in_page" }))
                    }
                    Err(e) => RpcResponse::error(id, -32000, format!("Click error: {}", e)),
                }
            }
            "tab.type" | "type" | "Type" | "tab.actType" => {
                let selector = params
                    .get("selector")
                    .or_else(|| params.get("target"))
                    .or_else(|| params.get("index"))
                    .map(|v| {
                        if v.is_number() {
                            v.to_string()
                        } else {
                            v.as_str().unwrap_or("").to_string()
                        }
                    })
                    .unwrap_or_default();
                let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.act_type(&selector, text).await {
                    Ok(res) => RpcResponse::success(id, serde_json::json!({ "status": res })),
                    Err(e) => RpcResponse::error(id, -32000, format!("Type error: {}", e)),
                }
            }
            "tab.evaluateJs" | "evaluateJs" | "EvaluateJs" => {
                let code = params.get("code").and_then(|v| v.as_str()).unwrap_or("");
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.evaluate_js(code) {
                    Ok(res) => RpcResponse::success(id, serde_json::json!({ "result": res })),
                    Err(e) => RpcResponse::error(id, -32000, format!("JS Error: {}", e)),
                }
            }
            "tab.setProfile" | "setProfile" | "SetProfile" => {
                let profile: DeviceProfile = match params
                    .get("profile")
                    .and_then(|p| serde_json::from_value(p.clone()).ok())
                {
                    Some(p) => p,
                    None => {
                        return RpcResponse::error(
                            id,
                            -32602,
                            "Invalid or missing 'profile' parameter",
                        )
                    }
                };
                let tab = match self.get_target_tab_mut(params) {
                    Ok(t) => t,
                    Err(e) => return RpcResponse::error(id, -32000, e.to_string()),
                };
                match tab.set_profile(profile) {
                    Ok(_) => RpcResponse::success(
                        id,
                        serde_json::json!({ "status": "profile_updated", "profile": profile }),
                    ),
                    Err(e) => RpcResponse::error(id, -32000, format!("SetProfile error: {}", e)),
                }
            }
            "ping" => RpcResponse::success(id, serde_json::json!({ "pong": true })),
            "shutdown" | "Shutdown" => {
                RpcResponse::success(id, serde_json::json!({ "status": "shutting down" }))
            }
            _ => RpcResponse::error(id, -32601, format!("Method not found: {}", method)),
        }
    }

    fn get_target_tab_mut(
        &mut self,
        params: &serde_json::Value,
    ) -> Result<&mut crate::browser::tab::BrowserTab> {
        if let Some(tab_id) = params.get("tab_id").and_then(|v| v.as_str()) {
            self.engine
                .get_tab_mut(tab_id)
                .ok_or_else(|| anyhow::anyhow!("Tab '{}' not found", tab_id))
        } else {
            self.engine.default_tab_mut()
        }
    }
}
