# Example Downloader Module

This is an example website module for Zoo Downloader that demonstrates how to implement support for custom file hosting websites.

## Overview

This module serves as a template and reference implementation for community developers who want to create their own website modules. It shows best practices for:

- Implementing the `WebsiteModule` trait
- URL pattern matching and validation
- Extracting download information from web pages
- Handling authentication requirements
- Handling CAPTCHA challenges
- Error handling and result types

## What This Module Does

This example module simulates support for a fictional file hosting website at `example-filehost.com`. It demonstrates:

1. **URL Matching**: Recognizes URLs in the format `https://example-filehost.com/file/{file_id}`
2. **Download Info Extraction**: Parses the page and extracts direct download links
3. **Authentication**: Shows how to handle login credentials
4. **CAPTCHA Handling**: Demonstrates the CAPTCHA challenge/response flow
5. **Metadata**: Provides module information (name, version, supported domains)

## Building the Module

```bash
cd modules/example-downloader
cargo build --release
```

The compiled module will be located at:
- Linux: `target/release/libexample_downloader.so`
- macOS: `target/release/libexample_downloader.dylib`
- Windows: `target/release/example_downloader.dll`

## Installing the Module

1. Copy the compiled library to Zoo Downloader's modules directory:
   ```bash
   # Linux/macOS
   cp target/release/libexample_downloader.so ~/.local/share/zoo-downloader/modules/

   # Windows
   copy target\release\example_downloader.dll %APPDATA%\zoo-downloader\modules\
   ```

2. Restart Zoo Downloader. The module will be automatically detected and loaded.

## Testing the Module

Run the unit tests:

```bash
cargo test
```

All tests should pass:
```
running 10 tests
test tests::test_can_handle ... ok
test tests::test_captcha_handling ... ok
test tests::test_create_module ... ok
test tests::test_empty_captcha_solution ... ok
test tests::test_extract_download_info_no_auth ... ok
test tests::test_extract_download_info_with_auth ... ok
test tests::test_extract_file_id ... ok
test tests::test_invalid_credentials ... ok
test tests::test_metadata ... ok

test result: ok. 10 passed; 0 failed
```

## Module Architecture

### Core Components

1. **WebsiteModule Trait**: The main interface that all modules must implement
   - `metadata()`: Returns module information
   - `can_handle()`: Checks if a URL is supported
   - `extract_download_info()`: Extracts download details from a URL
   - `handle_captcha()`: Handles CAPTCHA challenges

2. **Data Structures**:
   - `ModuleMetadata`: Module information (name, version, supported domains)
   - `DownloadInfo`: Direct download URL, filename, size, type
   - `CaptchaChallenge`: CAPTCHA challenge data
   - `CaptchaSolution`: User-provided CAPTCHA solution
   - `AuthCredentials`: Login credentials

3. **Error Handling**:
   - `ModuleError`: Enum of all possible errors
   - `ModuleResult<T>`: Type alias for `Result<T, ModuleError>`

### Entry Point

```rust
#[no_mangle]
pub extern "C" fn create_module() -> BoxedModule {
    Box::new(ExampleDownloader::new())
}
```

This function is called by Zoo Downloader when loading the module. It must be marked with `#[no_mangle]` to ensure the symbol name is preserved.

## Creating Your Own Module

To create a module for a different website:

1. **Copy this example** as a starting point:
   ```bash
   cp -r modules/example-downloader modules/my-website-downloader
   cd modules/my-website-downloader
   ```

2. **Update Cargo.toml**:
   ```toml
   [package]
   name = "my-website-downloader"
   ```

3. **Implement the WebsiteModule trait** in `src/lib.rs`:
   - Update `metadata()` with your module's information
   - Implement `can_handle()` to match your target website's URLs
   - Implement `extract_download_info()` to parse your target website
   - Implement `handle_captcha()` if the website requires CAPTCHA

4. **Add necessary dependencies** to Cargo.toml:
   ```toml
   [dependencies]
   scraper = "0.17"  # For HTML parsing
   select = "0.6"     # Alternative HTML parser
   ```

5. **Write tests** to verify your implementation

6. **Build and install** the module

## Common Implementation Patterns

### HTML Parsing

Use an HTML parser like `scraper` or `select`:

```rust
use scraper::{Html, Selector};

fn parse_page(&self, url: &str) -> ModuleResult<DownloadInfo> {
    let response = self.client.get(url).send()
        .map_err(|e| ModuleError::NetworkError(e.to_string()))?;

    let html = response.text()
        .map_err(|e| ModuleError::NetworkError(e.to_string()))?;

    let document = Html::parse_document(&html);
    let selector = Selector::parse("a.download-link").unwrap();

    let download_url = document.select(&selector)
        .next()
        .and_then(|el| el.value().attr("href"))
        .ok_or_else(|| ModuleError::ParseError("Download link not found".to_string()))?;

    Ok(DownloadInfo {
        direct_url: download_url.to_string(),
        // ... extract other fields
    })
}
```

### Session Management

Use `reqwest` cookies to maintain login sessions:

```rust
use reqwest::blocking::Client;

fn authenticate(&mut self, credentials: &AuthCredentials) -> ModuleResult<()> {
    let mut params = HashMap::new();
    params.insert("username", &credentials.username);
    params.insert("password", &credentials.password);

    let response = self.client
        .post("https://example.com/login")
        .form(&params)
        .send()
        .map_err(|e| ModuleError::NetworkError(e.to_string()))?;

    // Check if login was successful
    if !response.status().is_success() {
        return Err(ModuleError::InvalidCredentials);
    }

    // Cookies are automatically stored in the client
    Ok(())
}
```

### Error Handling

Always use `ModuleResult` and proper error types:

```rust
fn extract_download_info(&self, url: &str, _credentials: Option<&AuthCredentials>)
    -> ModuleResult<DownloadInfo>
{
    // Network errors
    let response = self.client.get(url).send()
        .map_err(|e| ModuleError::NetworkError(e.to_string()))?;

    // Parse errors
    let file_id = self.extract_file_id(url)
        .ok_or_else(|| ModuleError::ParseError("Invalid URL format".to_string()))?;

    // CAPTCHA required
    if self.needs_captcha(&response) {
        return Err(ModuleError::CaptchaRequired(/* challenge data */));
    }

    // Auth required
    if response.status() == 401 {
        return Err(ModuleError::AuthRequired);
    }

    Ok(/* download info */)
}
```

## API Reference

### WebsiteModule Trait

```rust
pub trait WebsiteModule: Send + Sync {
    fn metadata(&self) -> ModuleMetadata;
    fn can_handle(&self, url: &str) -> bool;
    fn extract_download_info(
        &self,
        url: &str,
        credentials: Option<&AuthCredentials>,
    ) -> ModuleResult<DownloadInfo>;
    fn handle_captcha(
        &self,
        challenge: &CaptchaChallenge,
        solution: &CaptchaSolution,
    ) -> ModuleResult<()>;
}
```

### ModuleMetadata

```rust
pub struct ModuleMetadata {
    pub id: String,              // Unique module identifier
    pub name: String,            // Human-readable name
    pub version: String,         // Semantic version (e.g., "1.0.0")
    pub author: String,          // Author name or organization
    pub description: String,     // Brief description
    pub supported_domains: Vec<String>,  // List of supported domains
}
```

### DownloadInfo

```rust
pub struct DownloadInfo {
    pub direct_url: String,           // Direct download URL
    pub filename: String,             // Original filename
    pub filesize: Option<u64>,        // File size in bytes (if available)
    pub mime_type: Option<String>,    // MIME type (if available)
    pub extra: HashMap<String, String>,  // Additional metadata
}
```

## Best Practices

1. **Always validate URLs** before processing them
2. **Use proper error types** instead of panicking
3. **Implement comprehensive tests** for all code paths
4. **Handle network errors gracefully** with timeouts and retries
5. **Respect robots.txt** and rate limiting
6. **Store session data** securely
7. **Document your module** with clear comments and examples
8. **Version your module** using semantic versioning

## Troubleshooting

### Module Not Loading

1. Check that the library is in the correct directory
2. Verify the library has the correct extension (.so/.dll/.dylib)
3. Check Zoo Downloader logs for loading errors

### Symbol Not Found Error

Make sure the `create_module` function is marked with `#[no_mangle]`:

```rust
#[no_mangle]
pub extern "C" fn create_module() -> BoxedModule {
    // ...
}
```

### Type Errors

Ensure your data structures match the types expected by Zoo Downloader. All structures must implement `Serialize` and `Deserialize`.

## License

This example module is provided as a reference implementation for the Zoo Downloader community. Feel free to use it as a template for your own modules.

## Support

For questions or issues:
- Check the main Zoo Downloader documentation
- Review other community modules for examples
- Ask in the Zoo Downloader community forums
