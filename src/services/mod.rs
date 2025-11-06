pub mod backup;
pub mod email;

use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub use email::EmailService;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScrapedCigarData {
    pub brand: Option<String>,
    pub name: Option<String>,
    pub size: Option<String>,
    pub length: Option<String>,
    pub ring_gauge: Option<String>,
    pub strength: Option<String>,
    pub origin: Option<String>,
    pub wrapper: Option<String>,
}

impl ScrapedCigarData {
    fn new() -> Self {
        Self {
            brand: None,
            name: None,
            size: None,
            length: None,
            ring_gauge: None,
            strength: None,
            origin: None,
            wrapper: None,
        }
    }
}

pub struct CigarScraper {
    client: reqwest::Client,
}

impl CigarScraper {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to build HTTP client, using default");
                reqwest::Client::new()
            });

        Self { client }
    }

    pub async fn scrape(&self, url: &str) -> Result<ScrapedCigarData, Box<dyn std::error::Error>> {
        // Fetch HTML
        let html = self.fetch_html(url).await?;
        let document = Html::parse_document(&html);

        // Determine which scraper to use based on URL
        if url.contains("cigaraficionado.com") {
            self.scrape_cigar_aficionado(&document, url)
        } else if url.contains("famous-smoke.com") || url.contains("famous") {
            self.scrape_famous_smoke(&document, url)
        } else if url.contains("cigarsinternational.com") {
            self.scrape_cigars_international(&document, url)
        } else if url.contains("jrcigars.com") {
            self.scrape_jr_cigars(&document, url)
        } else {
            self.scrape_generic(&document, url)
        }
    }

    async fn fetch_html(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.client.get(url).send().await?;
        let html = response.text().await?;
        Ok(html)
    }

    fn clean_text(&self, text: &str) -> Option<String> {
        let cleaned = text.trim().to_string();
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    }

    fn extract_brand_and_name(&self, full_name: &str) -> (Option<String>, Option<String>) {
        // Try to split on common separators
        if let Some((brand, name)) = full_name.split_once(" - ") {
            return (self.clean_text(brand), self.clean_text(name));
        }

        if let Some((brand, name)) = full_name.split_once(" by ") {
            return (self.clean_text(brand), self.clean_text(name));
        }

        // Try splitting on first word
        let words: Vec<&str> = full_name.split_whitespace().collect();
        if words.len() > 1 {
            return (
                self.clean_text(words[0]),
                self.clean_text(&words[1..].join(" ")),
            );
        }

        (None, self.clean_text(full_name))
    }

    fn extract_size_info(&self, text: &str) -> (Option<String>, Option<String>) {
        // Look for size pattern like "6 x 52" or "6.5 x 52"
        // Regex pattern is known to be valid, so this should never fail
        let size_re = match Regex::new(r"(\d+\.?\d*)\s*x\s*(\d+)") {
            Ok(re) => re,
            Err(e) => {
                tracing::error!(error = %e, "Failed to compile size regex - this should never happen");
                return (None, None);
            }
        };
        
        if let Some(caps) = size_re.captures(text) {
            return (Some(caps[1].to_string()), Some(caps[2].to_string()));
        }
        (None, None)
    }

    fn extract_strength(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();
        for strength in &["medium-full", "medium full", "full", "medium", "mild"] {
            if text_lower.contains(strength) {
                return Some(
                    match *strength {
                        "medium-full" | "medium full" => "Medium-Full",
                        "full" => "Full",
                        "medium" => "Medium",
                        "mild" => "Mild",
                        _ => strength,
                    }
                    .to_string(),
                );
            }
        }
        None
    }

    fn extract_origin(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();
        for origin in &[
            "nicaragua",
            "dominican republic",
            "honduras",
            "cuba",
            "ecuador",
            "connecticut",
            "mexico",
        ] {
            if text_lower.contains(origin) {
                return Some(
                    match *origin {
                        "nicaragua" => "Nicaragua",
                        "dominican republic" => "Dominican Republic",
                        "honduras" => "Honduras",
                        "cuba" => "Cuba",
                        "ecuador" => "Ecuador",
                        "connecticut" => "Connecticut",
                        "mexico" => "Mexico",
                        _ => origin,
                    }
                    .to_string(),
                );
            }
        }
        None
    }

    fn scrape_cigar_aficionado(
        &self,
        document: &Html,
        _url: &str,
    ) -> Result<ScrapedCigarData, Box<dyn std::error::Error>> {
        let mut result = ScrapedCigarData::new();

        // Try to find title
        if let Ok(selector) = Selector::parse("h1.entry-title, h1") {
            if let Some(title) = document.select(&selector).next() {
                let full_name = title.text().collect::<String>();
                let (brand, name) = self.extract_brand_and_name(&full_name);
                result.brand = brand;
                result.name = name;
            }
        }

        // Look through all text for details
        let body_text = document.root_element().text().collect::<String>();

        let (length, ring_gauge) = self.extract_size_info(&body_text);
        result.length = length;
        result.ring_gauge = ring_gauge;

        result.strength = self.extract_strength(&body_text);
        result.origin = self.extract_origin(&body_text);

        // Look for wrapper info
        if let Some(pos) = body_text.to_lowercase().find("wrapper") {
            let wrapper_text = &body_text[pos..pos.saturating_add(100).min(body_text.len())];
            if let Some(line) = wrapper_text.lines().next() {
                result.wrapper = self.clean_text(
                    line.trim_start_matches("wrapper")
                        .trim_start_matches(":")
                        .trim(),
                );
            }
        }

        Ok(result)
    }

    fn scrape_famous_smoke(
        &self,
        document: &Html,
        _url: &str,
    ) -> Result<ScrapedCigarData, Box<dyn std::error::Error>> {
        let mut result = ScrapedCigarData::new();

        // Try multiple h1 selectors
        let title_selectors = vec!["h1.product-name", "h1.product-title", "h1"];
        
        for selector_str in title_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(title) = document.select(&selector).next() {
                    let full_name = title.text().collect::<String>().trim().to_string();
                    if !full_name.is_empty() {
                        let (brand, name) = self.extract_brand_and_name(&full_name);
                        result.brand = brand;
                        result.name = name;
                        break;
                    }
                }
            }
        }

        // Look for product specifications/attributes
        let spec_selectors = vec![
            ".product-specs", 
            ".product-attributes",
            ".specifications",
            "table.specs",
        ];

        for selector_str in spec_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(specs) = document.select(&selector).next() {
                    let specs_text = specs.text().collect::<String>();
                    
                    let (length, ring_gauge) = self.extract_size_info(&specs_text);
                    if length.is_some() {
                        result.length = length;
                        result.ring_gauge = ring_gauge;
                    }
                    
                    if result.strength.is_none() {
                        result.strength = self.extract_strength(&specs_text);
                    }
                    
                    if result.origin.is_none() {
                        result.origin = self.extract_origin(&specs_text);
                    }
                }
            }
        }

        // Fall back to body text
        if result.length.is_none() || result.strength.is_none() || result.origin.is_none() {
            let body_text = document.root_element().text().collect::<String>();
            
            if result.length.is_none() {
                let (length, ring_gauge) = self.extract_size_info(&body_text);
                result.length = length;
                result.ring_gauge = ring_gauge;
            }
            
            if result.strength.is_none() {
                result.strength = self.extract_strength(&body_text);
            }
            
            if result.origin.is_none() {
                result.origin = self.extract_origin(&body_text);
            }
        }

        Ok(result)
    }

    fn scrape_cigars_international(
        &self,
        document: &Html,
        _url: &str,
    ) -> Result<ScrapedCigarData, Box<dyn std::error::Error>> {
        let mut result = ScrapedCigarData::new();

        // Try multiple selectors for product title
        let title_selectors = vec![
            "h1.product-name",
            "h1.product-title", 
            "h1[itemprop='name']",
            ".product-info h1",
            "h1",
        ];

        for selector_str in title_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(title) = document.select(&selector).next() {
                    let full_name = title.text().collect::<String>().trim().to_string();
                    if !full_name.is_empty() {
                        let (brand, name) = self.extract_brand_and_name(&full_name);
                        result.brand = brand;
                        result.name = name;
                        break;
                    }
                }
            }
        }

        // Look for structured product details
        let detail_selectors = vec![
            ".product-details",
            ".product-info",
            ".product-specifications",
            ".product-attributes",
        ];

        for selector_str in detail_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(details) = document.select(&selector).next() {
                    let details_text = details.text().collect::<String>();
                    
                    // Extract size, strength, origin from details
                    let (length, ring_gauge) = self.extract_size_info(&details_text);
                    if length.is_some() {
                        result.length = length;
                        result.ring_gauge = ring_gauge;
                    }
                    
                    if result.strength.is_none() {
                        result.strength = self.extract_strength(&details_text);
                    }
                    
                    if result.origin.is_none() {
                        result.origin = self.extract_origin(&details_text);
                    }
                }
            }
        }

        // Fall back to searching all body text if we didn't find details
        let body_text = document.root_element().text().collect::<String>();
        
        if result.length.is_none() {
            let (length, ring_gauge) = self.extract_size_info(&body_text);
            result.length = length;
            result.ring_gauge = ring_gauge;
        }
        
        if result.strength.is_none() {
            result.strength = self.extract_strength(&body_text);
        }
        
        if result.origin.is_none() {
            result.origin = self.extract_origin(&body_text);
        }

        Ok(result)
    }

    fn scrape_jr_cigars(
        &self,
        document: &Html,
        _url: &str,
    ) -> Result<ScrapedCigarData, Box<dyn std::error::Error>> {
        let mut result = ScrapedCigarData::new();

        // Try to find product title
        let title_selectors = vec!["h1.product-title", "h1.prod-title", "h1"];
        
        for selector_str in title_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(title) = document.select(&selector).next() {
                    let full_name = title.text().collect::<String>().trim().to_string();
                    if !full_name.is_empty() {
                        let (brand, name) = self.extract_brand_and_name(&full_name);
                        result.brand = brand;
                        result.name = name;
                        break;
                    }
                }
            }
        }

        // Look for detailed specs
        let detail_selectors = vec![
            ".product-details",
            ".prod-specs",
            ".specifications",
        ];

        for selector_str in detail_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(details) = document.select(&selector).next() {
                    let details_text = details.text().collect::<String>();
                    
                    let (length, ring_gauge) = self.extract_size_info(&details_text);
                    if length.is_some() {
                        result.length = length;
                        result.ring_gauge = ring_gauge;
                    }
                    
                    if result.strength.is_none() {
                        result.strength = self.extract_strength(&details_text);
                    }
                    
                    if result.origin.is_none() {
                        result.origin = self.extract_origin(&details_text);
                    }
                }
            }
        }

        // Fall back to all text
        let body_text = document.root_element().text().collect::<String>();
        
        if result.length.is_none() {
            let (length, ring_gauge) = self.extract_size_info(&body_text);
            result.length = length;
            result.ring_gauge = ring_gauge;
        }
        
        if result.strength.is_none() {
            result.strength = self.extract_strength(&body_text);
        }
        
        if result.origin.is_none() {
            result.origin = self.extract_origin(&body_text);
        }

        Ok(result)
    }

    fn scrape_generic(
        &self,
        document: &Html,
        _url: &str,
    ) -> Result<ScrapedCigarData, Box<dyn std::error::Error>> {
        let mut result = ScrapedCigarData::new();

        // Try to find any h1 as product name
        if let Ok(selector) = Selector::parse("h1") {
            if let Some(title) = document.select(&selector).next() {
                let full_name = title.text().collect::<String>();
                let (brand, name) = self.extract_brand_and_name(&full_name);
                result.brand = brand;
                result.name = name;
            }
        }

        // Scrape all text
        let body_text = document.root_element().text().collect::<String>();
        let (length, ring_gauge) = self.extract_size_info(&body_text);
        result.length = length;
        result.ring_gauge = ring_gauge;
        result.strength = self.extract_strength(&body_text);
        result.origin = self.extract_origin(&body_text);

        Ok(result)
    }
}

pub async fn scrape_cigar_url(url: &str) -> Result<ScrapedCigarData, Box<dyn std::error::Error>> {
    tracing::debug!(url = %url, "Starting cigar scrape");
    
    let scraper = CigarScraper::new();
    let result = scraper.scrape(url).await?;
    
    tracing::info!(
        brand = ?result.brand,
        name = ?result.name,
        size = ?result.size,
        length = ?result.length,
        ring_gauge = ?result.ring_gauge,
        strength = ?result.strength,
        origin = ?result.origin,
        wrapper = ?result.wrapper,
        "Cigar scrape completed successfully"
    );
    
    Ok(result)
}
