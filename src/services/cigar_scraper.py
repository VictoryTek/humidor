"""
Cigar URL Scraper Service
Scrapes cigar information from popular cigar retailer and review sites
"""
import re
import requests
from bs4 import BeautifulSoup
from typing import Optional, Dict, Any
from urllib.parse import urlparse


class CigarScraper:
    """Scrapes cigar information from various cigar websites"""
    
    TIMEOUT = 15
    USER_AGENT = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
    
    def __init__(self):
        self.headers = {
            'User-Agent': self.USER_AGENT,
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
            'Accept-Language': 'en-US,en;q=0.5',
            'Accept-Encoding': 'gzip, deflate',
            'Connection': 'keep-alive',
        }
    
    def scrape(self, url: str) -> Optional[Dict[str, Any]]:
        """
        Main scraping method that routes to appropriate scraper based on URL
        
        Args:
            url: The cigar product URL to scrape
            
        Returns:
            Dictionary with cigar information or None if scraping failed
        """
        try:
            parsed_url = urlparse(url)
            domain = parsed_url.netloc.lower()
            
            # Route to appropriate scraper based on domain
            if 'cigaraficionado.com' in domain:
                return self._scrape_cigar_aficionado(url)
            elif 'famous-smoke.com' in domain or 'famous' in domain:
                return self._scrape_famous_smoke(url)
            elif 'cigarsinternational.com' in domain or 'cigars international' in domain:
                return self._scrape_cigars_international(url)
            elif 'jrcigars.com' in domain or 'jr cigars' in domain:
                return self._scrape_jr_cigars(url)
            else:
                # Generic scraper for unknown sites
                return self._scrape_generic(url)
                
        except Exception as e:
            print(f"Error scraping URL: {e}")
            return None
    
    def _fetch_html(self, url: str) -> Optional[str]:
        """Fetch HTML content from URL"""
        try:
            response = requests.get(url, headers=self.headers, timeout=self.TIMEOUT)
            response.raise_for_status()
            return response.text
        except Exception as e:
            print(f"Error fetching URL: {e}")
            return None
    
    def _clean_text(self, text: Optional[str]) -> Optional[str]:
        """Clean and normalize text"""
        if not text:
            return None
        # Remove extra whitespace
        text = re.sub(r'\s+', ' ', text.strip())
        return text if text else None
    
    def _extract_brand_and_name(self, full_name: str) -> tuple[Optional[str], Optional[str]]:
        """Extract brand and cigar name from full product name"""
        if not full_name:
            return None, None
            
        # Common patterns: "Brand Name Cigar Name" or "Brand - Cigar Name"
        parts = re.split(r'\s+-\s+|\s+by\s+', full_name, maxsplit=1)
        if len(parts) == 2:
            return self._clean_text(parts[0]), self._clean_text(parts[1])
        
        # Try splitting on first word (often the brand)
        words = full_name.split()
        if len(words) > 1:
            return self._clean_text(words[0]), self._clean_text(' '.join(words[1:]))
        
        return None, self._clean_text(full_name)
    
    def _scrape_cigar_aficionado(self, url: str) -> Optional[Dict[str, Any]]:
        """Scrape from Cigar Aficionado reviews"""
        html = self._fetch_html(url)
        if not html:
            return None
            
        soup = BeautifulSoup(html, 'html.parser')
        
        result = {}
        
        # Try to find title
        title_elem = soup.find('h1', class_='entry-title') or soup.find('h1')
        if title_elem:
            full_name = self._clean_text(title_elem.get_text())
            brand, name = self._extract_brand_and_name(full_name)
            result['brand'] = brand
            result['name'] = name or full_name
        
        # Try to find cigar details
        details = soup.find_all(['p', 'div'], class_=re.compile('detail|spec|info', re.I))
        for detail in details:
            text = detail.get_text().lower()
            
            if 'size' in text or 'length' in text:
                # Extract size (e.g., "6 x 52" or "Robusto")
                size_match = re.search(r'(\d+\.?\d*)\s*x\s*(\d+)', text)
                if size_match:
                    result['length'] = size_match.group(1)
                    result['ring_gauge'] = size_match.group(2)
                else:
                    # Look for size names
                    for size_name in ['Robusto', 'Churchill', 'Toro', 'Corona', 'Torpedo']:
                        if size_name.lower() in text:
                            result['size'] = size_name
                            break
            
            if 'strength' in text:
                for strength in ['Mild', 'Medium', 'Full']:
                    if strength.lower() in text:
                        result['strength'] = strength
                        break
            
            if 'country' in text or 'origin' in text:
                # Common origins
                for origin in ['Nicaragua', 'Dominican Republic', 'Honduras', 'Cuba', 'Ecuador', 'Connecticut']:
                    if origin.lower() in text:
                        result['origin'] = origin
                        break
            
            if 'wrapper' in text:
                wrapper_match = re.search(r'wrapper[:\s]+([^,\n]+)', text, re.I)
                if wrapper_match:
                    result['wrapper'] = self._clean_text(wrapper_match.group(1))
        
        return result if result else None
    
    def _scrape_famous_smoke(self, url: str) -> Optional[Dict[str, Any]]:
        """Scrape from Famous Smoke Shop"""
        html = self._fetch_html(url)
        if not html:
            return None
            
        soup = BeautifulSoup(html, 'html.parser')
        result = {}
        
        # Product title
        title_elem = soup.find('h1', class_=re.compile('product|title', re.I)) or soup.find('h1')
        if title_elem:
            full_name = self._clean_text(title_elem.get_text())
            brand, name = self._extract_brand_and_name(full_name)
            result['brand'] = brand
            result['name'] = name or full_name
        
        # Product specifications
        specs = soup.find_all(['li', 'div', 'span'], class_=re.compile('spec|attribute|detail', re.I))
        for spec in specs:
            text = spec.get_text()
            label = text.lower()
            
            if 'strength' in label:
                for strength in ['Mild', 'Medium', 'Full']:
                    if strength.lower() in text:
                        result['strength'] = strength
                        break
            
            if 'size' in label or 'length' in label:
                size_match = re.search(r'(\d+\.?\d*)\s*x\s*(\d+)', text)
                if size_match:
                    result['length'] = size_match.group(1)
                    result['ring_gauge'] = size_match.group(2)
            
            if 'origin' in label or 'country' in label:
                for origin in ['Nicaragua', 'Dominican Republic', 'Honduras', 'Cuba']:
                    if origin.lower() in text:
                        result['origin'] = origin
                        break
            
            if 'wrapper' in label:
                wrapper_text = re.sub(r'wrapper[:\s]*', '', text, flags=re.I)
                result['wrapper'] = self._clean_text(wrapper_text)
        
        return result if result else None
    
    def _scrape_cigars_international(self, url: str) -> Optional[Dict[str, Any]]:
        """Scrape from Cigars International"""
        html = self._fetch_html(url)
        if not html:
            return None
            
        soup = BeautifulSoup(html, 'html.parser')
        result = {}
        
        # Similar pattern to Famous Smoke
        title_elem = soup.find('h1') or soup.find(class_=re.compile('product.*title', re.I))
        if title_elem:
            full_name = self._clean_text(title_elem.get_text())
            brand, name = self._extract_brand_and_name(full_name)
            result['brand'] = brand
            result['name'] = name or full_name
        
        # Look for structured data
        details = soup.find_all(['dt', 'dd', 'li'], class_=re.compile('spec|detail|attribute', re.I))
        for i, detail in enumerate(details):
            text = detail.get_text().lower()
            
            if 'strength' in text and i + 1 < len(details):
                next_text = details[i + 1].get_text()
                for strength in ['Mild', 'Medium', 'Full']:
                    if strength.lower() in next_text.lower():
                        result['strength'] = strength
                        break
        
        return result if result else None
    
    def _scrape_jr_cigars(self, url: str) -> Optional[Dict[str, Any]]:
        """Scrape from JR Cigars"""
        html = self._fetch_html(url)
        if not html:
            return None
            
        soup = BeautifulSoup(html, 'html.parser')
        result = {}
        
        # Similar to other scrapers
        title_elem = soup.find('h1')
        if title_elem:
            full_name = self._clean_text(title_elem.get_text())
            brand, name = self._extract_brand_and_name(full_name)
            result['brand'] = brand
            result['name'] = name or full_name
        
        return result if result else None
    
    def _scrape_generic(self, url: str) -> Optional[Dict[str, Any]]:
        """Generic scraper for unknown sites - looks for common patterns"""
        html = self._fetch_html(url)
        if not html:
            return None
            
        soup = BeautifulSoup(html, 'html.parser')
        result = {}
        
        # Try to find any h1 as product name
        title_elem = soup.find('h1')
        if title_elem:
            full_name = self._clean_text(title_elem.get_text())
            brand, name = self._extract_brand_and_name(full_name)
            result['brand'] = brand
            result['name'] = name or full_name
        
        # Look through all text for common patterns
        text_content = soup.get_text()
        
        # Try to find size
        size_match = re.search(r'(\d+\.?\d*)\s*x\s*(\d+)', text_content)
        if size_match:
            result['length'] = size_match.group(1)
            result['ring_gauge'] = size_match.group(2)
        
        # Try to find strength
        for strength in ['Mild', 'Medium-Full', 'Medium', 'Full']:
            if strength.lower() in text_content.lower():
                result['strength'] = strength
                break
        
        return result if result else None


def scrape_cigar_url(url: str) -> Optional[Dict[str, Any]]:
    """
    Convenience function to scrape a cigar URL
    
    Args:
        url: The cigar product URL to scrape
        
    Returns:
        Dictionary with cigar information or None if scraping failed
    """
    scraper = CigarScraper()
    return scraper.scrape(url)


if __name__ == '__main__':
    # Test the scraper
    test_url = input("Enter a cigar product URL to test: ")
    result = scrape_cigar_url(test_url)
    
    if result:
        print("\nScraped cigar information:")
        for key, value in result.items():
            print(f"  {key}: {value}")
    else:
        print("Failed to scrape cigar information")
