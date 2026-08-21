use url::Url;

/// Helper to build URLs for various Google and YouTube multimodal capabilities
pub struct GoogleEndpoints;

impl GoogleEndpoints {
    pub fn search(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}",
            urlencoding::encode(query)
        )
    }

    pub fn web_search(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}&udm=14",
            urlencoding::encode(query)
        )
    }

    pub fn image_search(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}&udm=2",
            urlencoding::encode(query)
        )
    }

    pub fn video_search(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}&tbm=vid",
            urlencoding::encode(query)
        )
    }

    pub fn short_video_search(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}+shorts&tbm=vid",
            urlencoding::encode(query)
        )
    }

    pub fn news_search(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}&tbm=nws",
            urlencoding::encode(query)
        )
    }

    pub fn forum_search(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}&udm=18",
            urlencoding::encode(query)
        )
    }

    pub fn shopping_search(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}&tbm=shop",
            urlencoding::encode(query)
        )
    }

    pub fn product_search(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}&tbm=shop",
            urlencoding::encode(query)
        )
    }

    pub fn books_search(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}&tbm=bks",
            urlencoding::encode(query)
        )
    }

    pub fn autocomplete(query: &str) -> String {
        format!(
            "https://suggestqueries.google.com/complete/search?client=chrome&q={}",
            urlencoding::encode(query)
        )
    }

    pub fn ai_overview(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}",
            urlencoding::encode(query)
        )
    }

    pub fn ai_mode(query: &str) -> String {
        format!(
            "https://www.google.com/search?q={}&udm=50",
            urlencoding::encode(query)
        )
    }

    pub fn scholar_search(query: &str) -> String {
        format!(
            "https://scholar.google.com/scholar?q={}",
            urlencoding::encode(query)
        )
    }

    pub fn patents_search(query: &str) -> String {
        format!(
            "https://patents.google.com/?q={}",
            urlencoding::encode(query)
        )
    }

    pub fn maps_search(query: &str) -> String {
        format!(
            "https://www.google.com/maps/search/{}",
            urlencoding::encode(query)
        )
    }

    pub fn finance_quote(ticker: &str) -> String {
        format!(
            "https://www.google.com/finance/quote/{}",
            urlencoding::encode(ticker)
        )
    }

    pub fn trends_search(query: &str) -> String {
        format!(
            "https://trends.google.com/trends/explore?q={}",
            urlencoding::encode(query)
        )
    }

    pub fn flights_search(origin: &str, dest: &str) -> String {
        format!(
            "https://www.google.com/travel/flights?q=Flights+from+{}+to+{}",
            urlencoding::encode(origin),
            urlencoding::encode(dest)
        )
    }

    pub fn hotels_search(location: &str) -> String {
        format!(
            "https://www.google.com/travel/hotels/{}",
            urlencoding::encode(location)
        )
    }

    pub fn travel_explore(destination: &str) -> String {
        format!(
            "https://www.google.com/travel/explore?q={}",
            urlencoding::encode(destination)
        )
    }

    pub fn youtube_search(query: &str) -> String {
        format!(
            "https://www.youtube.com/results?search_query={}",
            urlencoding::encode(query)
        )
    }

    pub fn youtube_shorts_search(query: &str) -> String {
        format!(
            "https://www.youtube.com/results?search_query={}&sp=mREBAgAL",
            urlencoding::encode(query)
        )
    }

    pub fn youtube_video(video_id: &str) -> String {
        if video_id.starts_with("http") {
            video_id.to_string()
        } else {
            format!(
                "https://www.youtube.com/watch?v={}",
                urlencoding::encode(video_id)
            )
        }
    }

    pub fn youtube_channel(channel: &str) -> String {
        if channel.starts_with("http") {
            channel.to_string()
        } else if channel.starts_with('@') {
            format!("https://www.youtube.com/{}", channel)
        } else {
            format!(
                "https://www.youtube.com/channel/{}",
                urlencoding::encode(channel)
            )
        }
    }

    pub fn youtube_playlist(playlist_id: &str) -> String {
        if playlist_id.starts_with("http") {
            playlist_id.to_string()
        } else {
            format!(
                "https://www.youtube.com/playlist?list={}",
                urlencoding::encode(playlist_id)
            )
        }
    }

    pub fn lens_visual_matches(image_url: &str) -> String {
        format!(
            "https://lens.google.com/uploadbyurl?url={}",
            urlencoding::encode(image_url)
        )
    }

    pub fn lens_exact_matches(image_url: &str) -> String {
        format!(
            "https://lens.google.com/uploadbyurl?url={}",
            urlencoding::encode(image_url)
        )
    }

    pub fn lens_products(image_url: &str) -> String {
        format!(
            "https://lens.google.com/uploadbyurl?url={}",
            urlencoding::encode(image_url)
        )
    }

    pub fn lens_about_image(image_url: &str) -> String {
        format!(
            "https://lens.google.com/uploadbyurl?url={}",
            urlencoding::encode(image_url)
        )
    }
}
