use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GoogleSearchResult {
    pub query: String,
    pub title: String,
    pub ai_overview: Option<String>,
    pub knowledge_panel: Option<String>,
    pub organic_results: Vec<OrganicResult>,
    pub related_questions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OrganicResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

impl GoogleSearchResult {
    pub fn to_markdown(&self) -> String {
        let mut md = format!("# {} - Google Search\n\n", self.title);
        if let Some(ai) = &self.ai_overview {
            md.push_str("## ✨ AI Overview\n");
            md.push_str(ai);
            md.push_str("\n\n");
        }
        if let Some(kp) = &self.knowledge_panel {
            md.push_str("## 🧠 Knowledge Panel\n");
            md.push_str(kp);
            md.push_str("\n\n");
        }
        if !self.organic_results.is_empty() {
            md.push_str("## 🔍 Organic Results\n\n");
            for (i, res) in self.organic_results.iter().enumerate() {
                md.push_str(&format!(
                    "### {}. [{}]({})\n{}\n\n",
                    i + 1,
                    res.title,
                    res.link,
                    res.snippet
                ));
            }
        }
        if !self.related_questions.is_empty() {
            md.push_str("## ❓ People Also Ask\n");
            for q in &self.related_questions {
                md.push_str(&format!("- {}\n", q));
            }
        }
        md
    }
}

// Minimal implementations for the other 30 capabilities
// Since there are 31 methods, we will define a generalized struct for the ones we don't have full extractors for yet.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GenericGoogleResult {
    pub query: String,
    pub title: String,
    pub raw_markdown: String,
}

impl GenericGoogleResult {
    pub fn to_markdown(&self) -> String {
        self.raw_markdown.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GoogleAutocompleteResult {
    pub query: String,
    pub suggestions: Vec<String>,
}

impl GoogleAutocompleteResult {
    pub fn to_markdown(&self) -> String {
        let mut md = format!("# Autocomplete for '{}'\n\n", self.query);
        for s in &self.suggestions {
            md.push_str(&format!("- {}\n", s));
        }
        md
    }
}

// You can expand these structs as specific parsers are added.
