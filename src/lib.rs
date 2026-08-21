//! # Headless Engine
//!
//! An ultra-lightweight (<30MB RAM), detection-free pure-Rust headless browser engine
//! architected specifically for AI agents, web scraping, and Go-based MCP servers.
//!
//! ## Quick Start (Rust SDK)
//! ```no_run
//! use headless_engine::{BrowserTab, DeviceProfile};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
//!     let report = tab.navigate("https://en.wikipedia.org/wiki/Rust_(programming_language)").await?;
//!     println!("Title: {}", report.page_title);
//!
//!     let markdown = tab.extract_markdown(None).unwrap();
//!     println!("Clean LLM Markdown:\n{}", markdown);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Multi-Tab Engine Example
//! ```no_run
//! use headless_engine::{BrowserEngine, DeviceProfile};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut engine = BrowserEngine::new()?;
//!     let tab1 = engine.create_tab(Some(DeviceProfile::ChromeWindows))?;
//!     let tab2 = engine.create_tab(Some(DeviceProfile::SafariIos))?;
//!
//!     engine.get_tab_mut(&tab1).unwrap().navigate("https://news.ycombinator.com/").await?;
//!     engine.get_tab_mut(&tab2).unwrap().navigate("https://google.com/").await?;
//!
//!     Ok(())
//! }
//! ```

#![allow(dead_code)]

pub mod api;
pub mod browser;
pub mod dom;
pub mod google;
pub mod js;
pub mod network;
pub mod render;

pub use api::rpc::{JsonRpcHandler, RpcError, RpcRequest, RpcResponse};
pub use browser::builder::BrowserBuilder;
pub use browser::engine::{BrowserEngine, TabSummary};
pub use browser::tab::{BrowserTab, NavigationReport};
pub use dom::markdown::HtmlToMarkdown;
pub use dom::{
    AiOverview, DomTree, FormInfo, FormInputInfo, ImageResult, InteractiveElement,
    InteractiveParser, KnowledgePanel, LinkInfo, NewsResult, OrganicResult, PageObservation,
    PageRenderer, RealBrowserScreenshot, ScreenshotResult, SearchResults, VideoResult,
};
pub use google::{
    GenericGoogleResult, GoogleAutocompleteResult, GoogleEndpoints, GoogleParser,
    GoogleSearchResult, OrganicResult as GoogleOrganicResult,
};
pub use network::fingerprint::{DeviceProfile, Fingerprint};
pub use render::{HtmlRenderer, LayoutEngine, PaintEngine};
