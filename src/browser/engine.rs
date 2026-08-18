use crate::browser::builder::BrowserBuilder;
use crate::browser::tab::BrowserTab;
use crate::network::fingerprint::DeviceProfile;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSummary {
    pub id: String,
    pub url: Option<String>,
    pub profile: DeviceProfile,
}

pub struct BrowserEngine {
    builder: BrowserBuilder,
    tabs: HashMap<String, BrowserTab>,
    tab_counter: usize,
    default_tab_id: String,
}

impl BrowserEngine {
    pub fn new() -> Result<Self> {
        Self::with_builder(BrowserBuilder::new())
    }

    pub fn with_builder(builder: BrowserBuilder) -> Result<Self> {
        let mut engine = Self {
            builder: builder.clone(),
            tabs: HashMap::new(),
            tab_counter: 0,
            default_tab_id: String::new(),
        };

        // Initialize default tab
        let tab_id = engine.create_tab(Some(builder.profile))?;
        engine.default_tab_id = tab_id;

        Ok(engine)
    }

    pub fn create_tab(&mut self, profile: Option<DeviceProfile>) -> Result<String> {
        self.tab_counter += 1;
        let tab_id = format!("tab_{}", self.tab_counter);

        let mut tab_builder = self.builder.clone();
        if let Some(p) = profile {
            tab_builder = tab_builder.profile(p);
        }

        let tab = tab_builder.build()?;
        self.tabs.insert(tab_id.clone(), tab);

        if self.default_tab_id.is_empty() {
            self.default_tab_id = tab_id.clone();
        }

        Ok(tab_id)
    }

    pub fn get_tab(&self, tab_id: &str) -> Option<&BrowserTab> {
        self.tabs.get(tab_id)
    }

    pub fn get_tab_mut(&mut self, tab_id: &str) -> Option<&mut BrowserTab> {
        self.tabs.get_mut(tab_id)
    }

    pub fn close_tab(&mut self, tab_id: &str) -> bool {
        let removed = self.tabs.remove(tab_id).is_some();
        if self.default_tab_id == tab_id {
            self.default_tab_id = self.tabs.keys().next().cloned().unwrap_or_default();
        }
        removed
    }

    pub fn list_tabs(&self) -> Vec<TabSummary> {
        self.tabs
            .iter()
            .map(|(id, tab)| TabSummary {
                id: id.clone(),
                url: tab.current_url.clone(),
                profile: tab.profile(),
            })
            .collect()
    }

    pub fn default_tab_mut(&mut self) -> Result<&mut BrowserTab> {
        if self.default_tab_id.is_empty() || !self.tabs.contains_key(&self.default_tab_id) {
            self.default_tab_id = self.create_tab(None)?;
        }
        self.tabs
            .get_mut(&self.default_tab_id)
            .ok_or_else(|| anyhow!("No active browser tab found"))
    }
}
