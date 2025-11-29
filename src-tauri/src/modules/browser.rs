// Headless browser integration for dynamic website support
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::Arc;
use std::time::Duration;

/// Error type for browser operations
#[derive(Debug)]
pub enum BrowserError {
    /// Failed to launch browser
    LaunchFailed(String),
    /// Failed to create new tab
    TabCreationFailed(String),
    /// Navigation error
    NavigationFailed(String),
    /// JavaScript execution error
    ScriptExecutionFailed(String),
    /// Element not found
    ElementNotFound(String),
    /// Timeout error
    Timeout(String),
}

impl std::fmt::Display for BrowserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrowserError::LaunchFailed(msg) => write!(f, "Failed to launch browser: {}", msg),
            BrowserError::TabCreationFailed(msg) => write!(f, "Failed to create tab: {}", msg),
            BrowserError::NavigationFailed(msg) => write!(f, "Navigation failed: {}", msg),
            BrowserError::ScriptExecutionFailed(msg) => {
                write!(f, "Script execution failed: {}", msg)
            }
            BrowserError::ElementNotFound(msg) => write!(f, "Element not found: {}", msg),
            BrowserError::Timeout(msg) => write!(f, "Operation timeout: {}", msg),
        }
    }
}

impl std::error::Error for BrowserError {}

pub type BrowserResult<T> = Result<T, BrowserError>;

/// Configuration for browser instance
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    /// Browser window width
    pub window_width: u32,
    /// Browser window height
    pub window_height: u32,
    /// Whether to run in headless mode
    pub headless: bool,
    /// Navigation timeout in seconds
    pub timeout_secs: u64,
    /// User agent string
    pub user_agent: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            window_width: 1920,
            window_height: 1080,
            headless: true,
            timeout_secs: 30,
            user_agent: Some(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            ),
        }
    }
}

/// Headless browser wrapper for website interaction
pub struct HeadlessBrowser {
    browser: Browser,
    config: BrowserConfig,
}

impl HeadlessBrowser {
    /// Creates a new headless browser instance
    ///
    /// # Arguments
    /// * `config` - Browser configuration
    ///
    /// # Returns
    /// A new browser instance or an error
    pub fn new(config: BrowserConfig) -> BrowserResult<Self> {
        let launch_options = LaunchOptions::default_builder()
            .headless(config.headless)
            .window_size(Some((config.window_width, config.window_height)))
            .build()
            .map_err(|e| BrowserError::LaunchFailed(format!("Invalid options: {}", e)))?;

        // Note: User agent customization requires different approach with headless_chrome
        // For now, we use the default user agent provided by Chrome

        let browser = Browser::new(launch_options)
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;

        Ok(Self { browser, config })
    }

    /// Creates a browser with default configuration
    pub fn default() -> BrowserResult<Self> {
        Self::new(BrowserConfig::default())
    }

    /// Navigates to a URL and returns the page content
    ///
    /// # Arguments
    /// * `url` - The URL to navigate to
    ///
    /// # Returns
    /// The HTML content of the page
    pub fn get_page_content(&self, url: &str) -> BrowserResult<String> {
        let tab = self.create_tab()?;

        tab.navigate_to(url)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        tab.wait_for_element("body")
            .map_err(|e| BrowserError::Timeout(format!("Waiting for body: {}", e)))?;

        let content = tab
            .get_content()
            .map_err(|e| BrowserError::ScriptExecutionFailed(e.to_string()))?;

        Ok(content)
    }

    /// Executes JavaScript on a page
    ///
    /// # Arguments
    /// * `url` - The URL to navigate to
    /// * `script` - JavaScript code to execute
    ///
    /// # Returns
    /// The result of script execution as a string
    pub fn execute_script(&self, url: &str, script: &str) -> BrowserResult<String> {
        let tab = self.create_tab()?;

        tab.navigate_to(url)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        tab.wait_for_element("body")
            .map_err(|e| BrowserError::Timeout(format!("Waiting for body: {}", e)))?;

        let result = tab
            .evaluate(script, false)
            .map_err(|e| BrowserError::ScriptExecutionFailed(e.to_string()))?;

        let value_str = result
            .value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string());

        Ok(value_str)
    }

    /// Waits for an element and returns its text content
    ///
    /// # Arguments
    /// * `url` - The URL to navigate to
    /// * `selector` - CSS selector for the element
    ///
    /// # Returns
    /// The text content of the element
    pub fn wait_for_element_text(&self, url: &str, selector: &str) -> BrowserResult<String> {
        let tab = self.create_tab()?;

        tab.navigate_to(url)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        let element = tab
            .wait_for_element(selector)
            .map_err(|e| BrowserError::ElementNotFound(format!("{}: {}", selector, e)))?;

        let text = element
            .get_inner_text()
            .map_err(|e| BrowserError::ScriptExecutionFailed(e.to_string()))?;

        Ok(text)
    }

    /// Clicks an element and waits for navigation
    ///
    /// # Arguments
    /// * `url` - The URL to navigate to
    /// * `selector` - CSS selector for the element to click
    ///
    /// # Returns
    /// Ok(()) if successful
    pub fn click_element(&self, url: &str, selector: &str) -> BrowserResult<()> {
        let tab = self.create_tab()?;

        tab.navigate_to(url)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        let element = tab
            .wait_for_element(selector)
            .map_err(|e| BrowserError::ElementNotFound(format!("{}: {}", selector, e)))?;

        element
            .click()
            .map_err(|e| BrowserError::ScriptExecutionFailed(e.to_string()))?;

        // Wait a bit for any navigation or dynamic content loading
        std::thread::sleep(Duration::from_millis(500));

        Ok(())
    }

    /// Extracts all links from a page
    ///
    /// # Arguments
    /// * `url` - The URL to navigate to
    ///
    /// # Returns
    /// Vector of URLs found on the page
    pub fn extract_links(&self, url: &str) -> BrowserResult<Vec<String>> {
        let script = r#"
            Array.from(document.querySelectorAll('a[href]'))
                .map(a => a.href)
                .filter(href => href.startsWith('http'))
        "#;

        let result = self.execute_script(url, script)?;

        // Parse JSON array result
        let links: Vec<String> = serde_json::from_str(&result)
            .unwrap_or_else(|_| Vec::new());

        Ok(links)
    }

    /// Takes a screenshot of the page
    ///
    /// # Arguments
    /// * `url` - The URL to navigate to
    ///
    /// # Returns
    /// PNG image data as bytes
    pub fn screenshot(&self, url: &str) -> BrowserResult<Vec<u8>> {
        let tab = self.create_tab()?;

        tab.navigate_to(url)
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;

        tab.wait_for_element("body")
            .map_err(|e| BrowserError::Timeout(format!("Waiting for body: {}", e)))?;

        let screenshot = tab
            .capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                None,
                None,
                true,
            )
            .map_err(|e| BrowserError::ScriptExecutionFailed(e.to_string()))?;

        Ok(screenshot)
    }

    /// Creates a new browser tab
    fn create_tab(&self) -> BrowserResult<Arc<Tab>> {
        self.browser
            .new_tab()
            .map_err(|e| BrowserError::TabCreationFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_config_default() {
        let config = BrowserConfig::default();
        assert_eq!(config.window_width, 1920);
        assert_eq!(config.window_height, 1080);
        assert!(config.headless);
        assert_eq!(config.timeout_secs, 30);
        assert!(config.user_agent.is_some());
    }

    #[test]
    fn test_browser_error_display() {
        let error = BrowserError::LaunchFailed("test error".to_string());
        assert!(error.to_string().contains("Failed to launch browser"));

        let error = BrowserError::ElementNotFound("button".to_string());
        assert!(error.to_string().contains("Element not found"));
    }

    // Note: Actual browser tests are skipped in CI environments
    // as they require Chrome/Chromium to be installed
    #[test]
    #[ignore]
    fn test_browser_creation() {
        let browser = HeadlessBrowser::default();
        assert!(browser.is_ok());
    }
}
