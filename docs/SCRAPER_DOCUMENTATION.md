# Cigar URL Scraper - Technical Documentation

## Overview

The Humidor application includes a web scraper that can extract cigar information from retailer product pages. This feature allows users to quickly add cigars by importing data from supported websites.

## Supported Retailers

✅ **Cigars International** (cigarsinternational.com)
✅ **Famous Smoke Shop** (famous-smoke.com)
✅ **JR Cigars** (jrcigars.com)

## How It Works

### 1. User Input
Users click "🔗 Import from URL" in the add cigar modal and paste a product URL from a supported retailer.

### 2. Scraping Process
The backend:
1. Fetches the HTML from the provided URL
2. Parses the page structure using CSS selectors
3. Extracts cigar information using multiple strategies
4. Returns structured data to populate the form

### 3. Data Extraction

The scraper attempts to find:

| Field | Description | Example |
|-------|-------------|---------|
| **Brand** | Cigar manufacturer | "Arturo Fuente", "Padron" |
| **Name** | Specific cigar line/name | "Hemingway", "1964 Anniversary" |
| **Length** | Cigar length in inches | "6", "7.5" |
| **Ring Gauge** | Diameter in 64ths of an inch | "52", "60" |
| **Strength** | Flavor intensity | "Mild", "Medium", "Full" |
| **Origin** | Country of manufacture | "Dominican Republic", "Nicaragua" |
| **Wrapper** | Wrapper tobacco type | "Connecticut", "Maduro" |

### 4. Form Population
Extracted data auto-fills the cigar form fields. Users can review and modify before saving.

## Technical Implementation

### Extraction Strategy

The scraper uses a **multi-layered approach**:

1. **Primary Selectors**: Look for common product page elements
   - `h1.product-name`, `h1.product-title`
   - `.product-details`, `.product-specifications`
   - Structured data attributes (`[itemprop='name']`)

2. **Pattern Matching**: Extract data using regex patterns
   - Size: `6 x 52`, `7.5 x 60`
   - Strength keywords: "mild", "medium", "full"
   - Origin names: "Nicaragua", "Dominican Republic"

3. **Fallback**: Search entire page text if structured data not found

### Example URL Formats

**Cigars International:**
```
https://www.cigarsinternational.com/p/arturo-fuente-hemingway-cigars/...
```

**Famous Smoke:**
```
https://www.famous-smoke.com/padron+1964+anniversary+cigars/...
```

**JR Cigars:**
```
https://www.jrcigars.com/item/my-father-cigars/...
```

## Debugging

### Server Logs

When scraping, detailed logs are written to Docker logs:

```powershell
docker logs humidor-web-1 --tail 50 | Select-String "Scrape"
```

Expected output:
```
🔍 Starting scrape for URL: https://...
📊 Scrape results:
  - Brand: Some("Arturo Fuente")
  - Name: Some("Hemingway Short Story")
  - Length: Some("4")
  - Ring Gauge: Some("49")
  - Strength: Some("Medium")
  - Origin: Some("Dominican Republic")
```

### Common Issues

#### 1. No Data Extracted

**Symptoms**: Scraper succeeds but returns empty fields

**Causes**:
- Website structure changed (CSS selectors outdated)
- Dynamic content loaded via JavaScript (not accessible to scraper)
- Anti-scraping measures (blocking automated requests)

**Solutions**:
- Check Docker logs to see what was found
- Try a different URL from the same site
- Manually enter the data

#### 2. Partial Data

**Symptoms**: Some fields populated, others empty

**Cause**: Data not present in standard locations on the page

**Solution**: Fill in missing fields manually - the scraper provides a starting point

#### 3. Network Errors

**Symptoms**: "Failed to scrape cigar information" error

**Causes**:
- Invalid URL
- Website temporarily unavailable
- Network connectivity issues

**Solutions**:
- Verify the URL is correct and accessible in a browser
- Try again in a few moments
- Check Docker logs for specific error messages

## Limitations

### What the Scraper Can't Do

1. **JavaScript-Heavy Sites**: Cannot execute JavaScript, so dynamically loaded content may not be accessible
2. **Anti-Bot Measures**: Sites with CAPTCHA or anti-scraping protection will fail
3. **Varying Formats**: Different product pages on the same site may use different layouts
4. **Image URLs**: Currently doesn't extract product images (would require handling CDN URLs)

### Legal Considerations

**Web scraping for personal use** (like importing your own cigar data) is generally acceptable, but:

- ⚠️ Don't scrape at high frequency (rate limiting)
- ⚠️ Respect robots.txt files
- ⚠️ Don't redistribute scraped data commercially
- ⚠️ Be aware of website Terms of Service

This feature is intended for **personal inventory management only**.

## Future Improvements

Potential enhancements:

1. **More Retailers**: Add support for additional cigar websites
2. **Image Scraping**: Extract and download product images
3. **Price Tracking**: Store historical pricing data
4. **Review Data**: Import ratings and tasting notes
5. **Batch Import**: Process multiple URLs at once
6. **Browser Automation**: Use headless browser for JavaScript sites (Selenium/Puppeteer)

## Developer Notes

### Adding New Retailers

To add support for a new retailer:

1. **Identify URL Pattern**: Determine how to detect the site
2. **Inspect Page Structure**: Find CSS selectors for product data
3. **Add Site-Specific Scraper**: Implement in `src/services/mod.rs`
4. **Test Thoroughly**: Verify with multiple product pages
5. **Update Documentation**: Add to supported retailers list

Example:

```rust
fn scrape_new_retailer(
    &self,
    document: &Html,
    _url: &str,
) -> Result<ScrapedCigarData, Box<dyn std::error::Error>> {
    let mut result = ScrapedCigarData::new();
    
    // Find product title
    if let Ok(selector) = Selector::parse("h1.product-name") {
        if let Some(title) = document.select(&selector).next() {
            let full_name = title.text().collect::<String>().trim().to_string();
            let (brand, name) = self.extract_brand_and_name(&full_name);
            result.brand = brand;
            result.name = name;
        }
    }
    
    // Extract other fields...
    
    Ok(result)
}
```

### Testing

Manual testing procedure:

1. Navigate to a retailer's product page
2. Copy the URL
3. Open Humidor's "Add Cigar" modal
4. Click "Import from URL"
5. Paste URL and click "Import"
6. Verify extracted data
7. Check Docker logs for detailed output

### Code Location

- **Rust Scraper**: `src/services/mod.rs`
- **Handler**: `src/handlers/cigars.rs` (`scrape_cigar_url`)
- **Frontend**: `static/index.html` (import modal)
- **JavaScript**: `static/app.js` (`importFromUrl` function)
