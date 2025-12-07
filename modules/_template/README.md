# Module Template

This is a template for creating new Zoo Downloader site modules.

## Quick Start

1. Copy this `_template` directory to a new directory with your module ID:
   ```bash
   cp -r modules/_template modules/your-module-id
   ```

2. Edit `manifest.json`:
   - Change `id` to your module's unique ID
   - Update `name`, `author`, `description`
   - Add your URL patterns
   - Configure permissions (domains, credentials)

3. Edit `index.js`:
   - Rename the class from `YourModule` to your module name
   - Implement `canHandle()` for URL matching
   - Implement `extractDownloadInfo()` for download logic
   - Add any helper methods you need

4. Edit this `README.md` with your module documentation

5. Test your module:
   ```bash
   # After Phase 3 infrastructure is complete
   cd src-tauri
   cargo test your_module
   ```

## Module Structure

```
your-module-id/
├── manifest.json   # Module metadata and permissions
├── index.js        # Module implementation
└── README.md       # Module documentation
```

## Manifest Fields

### Required Fields

- `id`: Unique module identifier (lowercase, alphanumeric, dashes)
- `name`: Display name
- `version`: Semantic version (e.g., "1.0.0")
- `author`: Your name or organization
- `description`: Brief description
- `urlPatterns`: Array of URL patterns this module handles
- `permissions.network.allowDomains`: Whitelisted domains

### Optional Fields

- `homepage`: Project URL
- `permissions.credentials`: Credential requirements

## Module Class Requirements

Your module class must implement:

1. **`canHandle(url)`**: Return `true` if this module can handle the URL
2. **`extractDownloadInfo(url, context)`**: Extract download information

The `extractDownloadInfo()` method must return an object with:
```javascript
{
  downloadUrl: string,    // Required: Direct download URL
  filename: string,       // Required: File name
  filesize: number|null,  // Optional: File size in bytes
  headers: object         // Optional: Additional HTTP headers
}
```

## Context API

The `context` object provided to your module includes:

### `context.http`
- `context.http.get(url, options)`: Make GET request
- `context.http.post(url, options)`: Make POST request

### `context.credentials` (if configured)
- Access to saved credentials (username, password, etc.)
- Never exposes raw credentials to JS

### `context.captcha`
- `context.captcha.request(options)`: Request user to solve CAPTCHA

### `context.log`
- `context.log.info(message)`: Log info message
- `context.log.warn(message)`: Log warning
- `context.log.error(message)`: Log error

## Security Guidelines

1. **Only access whitelisted domains** - Listed in `permissions.network.allowDomains`
2. **Never hardcode credentials** - Use `context.credentials`
3. **Validate all inputs** - Check URLs, parse safely
4. **Handle errors gracefully** - Return clear error messages
5. **Stay within limits**:
   - CPU time: ≤5 seconds per request
   - Memory: ≤50 MB
   - Network: ≤60 requests/minute

## Testing

Create test files in `src-tauri/src/modules/tests/` to test your module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_module_url_detection() {
        // Test URL matching
    }

    #[test]
    fn test_your_module_parsing() {
        // Test HTML parsing
    }
}
```

## Documentation

See the main documentation:
- **Quick Start**: `specs/004-site-module-integration/quickstart.md`
- **Data Model**: `specs/004-site-module-integration/data-model.md`
- **API Contracts**: `specs/004-site-module-integration/contracts/tauri-commands.md`

## Support

For questions or issues:
- GitHub Issues: [your repository]
- Documentation: [link to docs]

## License

Specify your module's license here.
