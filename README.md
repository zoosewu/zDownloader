# Zoo Downloader

A powerful, feature-rich download manager built with Tauri, React, and Rust. Zoo Downloader provides multi-threaded downloads, proxy support, Chrome browser integration, and intelligent download management.

![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

## ✨ Features

### Core Download Engine
- **Multi-threaded Segmented Downloads**: Automatically splits large files into segments for faster downloads
- **Resume Capability**: Pause and resume downloads without losing progress
- **Real-time Progress Tracking**: Monitor download speed, progress, and status in real-time
- **Intelligent Download Strategy**: Automatically chooses optimal download method based on server capabilities

### Proxy Management
- **Proxy Pool**: Manage multiple proxies with automatic load balancing
- **Health Monitoring**: Track proxy performance and availability
- **Automatic Failover**: Seamlessly switch to backup proxies when primary fails
- **Direct Connection Fallback**: Option to bypass proxies when all fail

### Download Management
- **Create, Pause, Resume, Cancel**: Full control over individual downloads
- **Batch Import**: Import multiple URLs from text files
- **Smart Filtering**: View downloads by status (active, paused, completed, failed)
- **Clipboard Monitoring**: Automatically detect download URLs from clipboard
- **Download History**: Keep track of all downloads with timestamps

### Chrome Browser Integration
- **Chrome Extension**: Detect downloadable links on web pages
- **Right-click Context Menu**: Quick download from any link
- **Visual Indicators**: Automatically highlights download links with icon
- **Seamless Communication**: HTTP-based integration for easy setup

### CAPTCHA Support
- **Interactive CAPTCHA Solver**: Handle CAPTCHAs during downloads
- **Multiple CAPTCHA Types**: Support for various CAPTCHA challenges
- **User-friendly Dialog**: Clear interface for CAPTCHA resolution

### User Interface
- **Modern Material-UI Design**: Clean and intuitive interface
- **Multi-language Support**: English and Traditional Chinese (繁體中文)
- **Dark/Light Theme**: Adaptive theme support
- **Responsive Layout**: Works seamlessly on different screen sizes

## 📋 Prerequisites

### For Users
- **Windows**: Windows 10/11
- **macOS**: macOS 10.15 or later
- **Linux**: Ubuntu 20.04 or later (or equivalent)

### For Developers
- **Node.js**: v18 or later
- **pnpm**: v8 or later (recommended) or npm
- **Rust**: Latest stable version
- **Cargo**: Included with Rust installation

## 🚀 Installation

### Download Pre-built Binary (Recommended)
Coming soon! Pre-built binaries will be available on the [Releases](https://github.com/your-username/zoo-downloader/releases) page.

### Build from Source

1. **Clone the repository**
```bash
git clone https://github.com/your-username/zoo-downloader.git
cd zoo-downloader
```

2. **Install dependencies**
```bash
pnpm install
# or
npm install
```

3. **Build the application**
```bash
pnpm tauri build
# or
npm run tauri build
```

4. **Find the installer**
- Windows: `src-tauri/target/release/bundle/msi/zoo-downloader_0.1.0_x64_en-US.msi`
- macOS: `src-tauri/target/release/bundle/dmg/zoo-downloader_0.1.0_x64.dmg`
- Linux: `src-tauri/target/release/bundle/deb/zoo-downloader_0.1.0_amd64.deb`

## 📖 Usage Guide

### Basic Download

1. **Start a download**
   - Click the "+" button or use "Add Download" menu
   - Enter the download URL
   - Optionally specify filename
   - Click "Start Download"

2. **Monitor progress**
   - View real-time download speed
   - Track percentage completed
   - See estimated time remaining

3. **Control downloads**
   - **Pause**: Temporarily stop download (can resume later)
   - **Resume**: Continue paused download from where it stopped
   - **Cancel**: Remove download from list
   - **Retry**: Restart failed download

### Batch Downloads

1. **Prepare URL file**
   - Create a text file with one URL per line
   - Example:
     ```
     https://example.com/file1.zip
     https://example.com/file2.pdf
     https://example.com/file3.mp4
     ```

2. **Import URLs**
   - Click "Batch Import" button
   - Select your URL file
   - All URLs will be added to download queue

### Proxy Configuration

1. **Add Proxy**
   - Open Settings → Proxy Management
   - Click "Add Proxy"
   - Enter proxy details (protocol, host, port)
   - Optionally add username/password

2. **Manage Proxies**
   - View proxy health status
   - Remove or update existing proxies
   - Enable/disable individual proxies
   - View proxy statistics

### Chrome Extension Setup

1. **Install Extension**
   - Open Chrome and navigate to `chrome://extensions/`
   - Enable "Developer mode" (top right)
   - Click "Load unpacked"
   - Select the `chrome-extension` folder from the Zoo Downloader directory

2. **Verify Connection**
   - Ensure Zoo Downloader application is running
   - Extension icon should show green (connected)
   - If red, check that HTTP server is running on port 45677

3. **Use Extension**
   - Browse any website
   - Downloadable links will show ⬇️ icon
   - Right-click any link → "Download with Zoo Downloader"

## 🏗️ Architecture

### Technology Stack

**Frontend**
- **React 19**: Modern UI framework
- **TypeScript**: Type-safe JavaScript
- **Material-UI (MUI)**: Component library
- **Jotai**: State management
- **i18next**: Internationalization
- **Vite**: Build tool and dev server

**Backend**
- **Rust**: High-performance systems language
- **Tauri**: Desktop application framework
- **Tokio**: Async runtime
- **Reqwest**: HTTP client
- **Warp**: HTTP server for Chrome extension
- **Serde**: Serialization/deserialization

### Project Structure

```
zoo-downloader/
├── src/                          # Frontend source code
│   ├── components/               # React components
│   │   ├── DownloadList.tsx      # Main download list view
│   │   ├── AddDownloadDialog.tsx # Add download dialog
│   │   ├── CaptchaDialog.tsx     # CAPTCHA solver dialog
│   │   └── ...
│   ├── hooks/                    # Custom React hooks
│   │   ├── useDownloads.ts       # Download management
│   │   ├── useTauriEvents.ts     # Event listeners
│   │   └── ...
│   ├── stores/                   # Jotai state atoms
│   │   ├── downloadStore.ts      # Download state
│   │   ├── captchaStore.ts       # CAPTCHA state
│   │   └── ...
│   ├── locales/                  # i18n translation files
│   │   ├── en.json               # English
│   │   └── zh-TW.json            # Traditional Chinese
│   └── types/                    # TypeScript type definitions
│
├── src-tauri/                    # Backend source code
│   ├── src/
│   │   ├── commands/             # Tauri command handlers
│   │   │   ├── download.rs       # Download operations
│   │   │   ├── proxy.rs          # Proxy management
│   │   │   └── ...
│   │   ├── download/             # Download engine
│   │   │   ├── engine.rs         # Core download logic
│   │   │   ├── segment.rs        # Segmentation logic
│   │   │   ├── merger.rs         # Segment merging
│   │   │   └── ...
│   │   ├── models/               # Data models
│   │   │   ├── download_task.rs  # Download task model
│   │   │   ├── download_segment.rs
│   │   │   └── proxy.rs          # Proxy model
│   │   ├── proxy/                # Proxy pool
│   │   │   └── pool.rs           # Proxy management
│   │   ├── modules/              # Website-specific modules
│   │   │   ├── captcha.rs        # CAPTCHA handling
│   │   │   ├── auth.rs           # Authentication
│   │   │   └── ...
│   │   ├── native_messaging/     # Chrome extension integration
│   │   │   └── http_server.rs    # HTTP server
│   │   └── lib.rs                # Library entry point
│   └── Cargo.toml                # Rust dependencies
│
├── chrome-extension/             # Chrome extension
│   ├── manifest.json             # Extension manifest
│   ├── background.js             # Background service worker
│   ├── content.js                # Content script
│   └── popup.html                # Extension popup
│
└── package.json                  # Node.js dependencies
```

### Key Design Patterns

1. **Event-Driven Architecture**: Frontend and backend communicate via Tauri events
2. **Atomic State Management**: Jotai atoms for granular state updates
3. **Async/Await**: Non-blocking operations throughout
4. **Segmented Downloads**: Parallel segment downloads with automatic merging
5. **Proxy Pooling**: Round-robin with health-based selection

## 🛠️ Development

### Run in Development Mode

```bash
pnpm tauri dev
# or
npm run tauri dev
```

This will:
- Start Vite dev server (frontend)
- Compile and run Rust backend
- Launch application with hot-reload

### Run Tests

```bash
# Rust tests
cd src-tauri
cargo test

# Frontend tests (if configured)
pnpm test
```

### Code Quality

```bash
# Rust linting
cd src-tauri
cargo clippy

# Rust formatting
cargo fmt

# TypeScript linting
pnpm run lint
```

### Building for Production

```bash
# Build optimized binary
pnpm tauri build

# Build for specific platform
pnpm tauri build --target x86_64-pc-windows-msvc  # Windows
pnpm tauri build --target x86_64-apple-darwin     # macOS Intel
pnpm tauri build --target aarch64-apple-darwin    # macOS Apple Silicon
pnpm tauri build --target x86_64-unknown-linux-gnu # Linux
```

## 🌐 Internationalization

Zoo Downloader supports multiple languages. To add a new language:

1. Create a new translation file in `src/locales/` (e.g., `fr.json`)
2. Copy the structure from `en.json`
3. Translate all strings
4. Import and register in `src/i18n.ts`

Example translation file structure:
```json
{
  "downloads": {
    "title": "Downloads",
    "addDownload": "Add Download",
    "status": {
      "pending": "Pending",
      "downloading": "Downloading",
      "completed": "Completed"
    }
  }
}
```

## 🤝 Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Coding Standards
- **Rust**: Follow Rust idioms and run `cargo fmt` and `cargo clippy`
- **TypeScript**: Use TypeScript strict mode, follow ESLint rules
- **Commits**: Use conventional commits format

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - Desktop application framework
- [Material-UI](https://mui.com/) - React component library
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP client for Rust
- All contributors who help improve this project

## 📧 Contact

- **Issues**: [GitHub Issues](https://github.com/your-username/zoo-downloader/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-username/zoo-downloader/discussions)

## 🗺️ Roadmap

- [ ] Support for more protocols (FTP, BitTorrent)
- [ ] Cloud storage integration (Google Drive, Dropbox)
- [ ] Schedule downloads for specific times
- [ ] Bandwidth limiting
- [ ] Browser integration for Firefox and Edge
- [ ] Video/Audio stream detection and download
- [ ] Plugin system for website-specific modules

---

**Made with ❤️ using Tauri, React, and Rust**
