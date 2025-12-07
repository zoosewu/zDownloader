# Keep2Share (K2S) Module

## Overview

This module enables Zoo Downloader to support file downloads from Keep2Share (k2s.cc) and its mirrors.

## Features

- ✅ Automatic URL detection for k2s.cc domains
- 🔄 **In Development** - Full implementation in Phase 4
- 🔄 User authentication support
- 🔄 CAPTCHA handling
- 🔄 Premium/Free account detection
- 🔄 Download link extraction

## Status

**Current Phase**: Phase 1 - Template Created
**Next Phase**: Phase 4 - Full Implementation (Task T078-T083)

This is currently a skeleton module used for testing the module loading infrastructure. Full K2S download functionality will be implemented in Phase 4.

## Supported URLs

- `https://k2s.cc/file/*`
- `https://*.k2s.cc/file/*`
- `https://keep2share.cc/file/*`

## Required Credentials

- **username**: K2S account username/email
- **password**: K2S account password
- **apiKey**: (Optional) K2S API key for premium accounts

## Network Permissions

This module is allowed to access:
- `k2s.cc`
- `*.k2s.cc`
- `keep2share.cc`
- `api.keep2share.cc`

## Development Roadmap

### Phase 4 Implementation Tasks

- [ ] T078: Complete manifest.json permissions
- [ ] T079: Implement canHandle() URL matching
- [ ] T080: Implement authentication flow
- [ ] T081: Implement URL parsing logic
- [ ] T082: Implement CAPTCHA handling
- [ ] T083: Implement account type detection
- [ ] T084: Add integration tests

## Testing

To test this module (after Phase 3 infrastructure is complete):

```bash
# Backend tests
cd src-tauri
cargo test k2s

# Manual testing
# 1. Place a K2S URL in the download input
# 2. System should auto-detect this module
# 3. (Phase 4) Module should authenticate and extract download link
```

## License

Part of Zoo Downloader project.

## Author

Zoo Downloader Community

## Version

1.0.0 (Template/Skeleton)
