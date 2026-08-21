use crate::dom::DomTree;
use crate::google::types::{GenericGoogleResult, GoogleAutocompleteResult, GoogleSearchResult, OrganicResult};
use serde_json::Value;

pub struct GoogleParser;

impl GoogleParser {
    pub fn parse_search_results(html: &str, url: &str) -> GoogleSearchResult {
        let dom = match DomTree::parse(html) {
            Ok(d) => d,
            Err(_) => return GoogleSearchResult {
                query: url.to_string(),
                title: "Google Search (Parse Error)".to_string(),
                ..Default::default()
            }
        };
        
        let title = dom.extract(Some("title")).unwrap_or_else(|| "Google Search".to_string());
        
        // This leverages the existing SearchResults logic implicitly or we build it explicitly.
        let search_results = dom.parse_google_search_results();
        
        let mut organic = Vec::new();
        for res in search_results.organic_results {
            organic.push(OrganicResult {
                title: res.title,
                link: res.link,
                snippet: res.snippet,
            });
        }
        
        GoogleSearchResult {
            query: url.to_string(), // In a real app we'd parse the URL q= param
            title,
            ai_overview: search_results.ai_overview.map(|a| a.summary),
            knowledge_panel: search_results.knowledge_panel.map(|k| k.description),
            organic_results: organic,
            related_questions: search_results.related_questions,
        }
    }

    pub fn parse_autocomplete(json: &str) -> GoogleAutocompleteResult {
        if let Ok(val) = serde_json::from_str::<Value>(json) {
            if let Some(arr) = val.as_array() {
                if arr.len() >= 2 {
                    if let Some(query) = arr[0].as_str() {
                        if let Some(suggestions) = arr[1].as_array() {
                            let mut suggs = Vec::new();
                            for s in suggestions {
                                if let Some(s_str) = s.as_str() {
                                    suggs.push(s_str.to_string());
                                }
                            }
                            return GoogleAutocompleteResult {
                                query: query.to_string(),
                                suggestions: suggs,
                            };
                        }
                    }
                }
            }
        }
        GoogleAutocompleteResult::default()
    }

    pub fn parse_generic(html: &str, url: &str) -> GenericGoogleResult {
        let dom = match DomTree::parse(html) {
            Ok(d) => d,
            Err(_) => return GenericGoogleResult {
                query: url.to_string(),
                title: "Parse Error".to_string(),
                raw_markdown: "Error parsing HTML".to_string(),
            }
        };
        let title = dom.extract(Some("title")).unwrap_or_else(|| "Google Result".to_string());
        let md = dom.extract_markdown(None, Some(url));
        GenericGoogleResult {
            query: url.to_string(),
            title,
            raw_markdown: md,
        }
    }
}
