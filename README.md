# 🌊 Glycerin Browser Engine

**A Modern, Secure, and Extensible Browser Engine Built in Rust**

Version: **0.18.0** | Status: **Production Ready**

---

## 📋 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Installation](#installation)
- [Usage](#usage)
- [API Reference](#api-reference)
- [Development](#development)
- [Roadmap](#roadmap)

---

## 🎯 Overview

Glycerin is a complete browser engine implementation featuring:

- **Phase 1**: Tabbed browser UI with navigation controls
- **Phase 2**: SQLite-based data persistence (history, bookmarks, cookies, sessions)
- **Phase 3**: HTML5 media support (audio, video, images)
- **Phase 4**: Security & sandboxing (CSP, safe browsing, process isolation)
- **Phase 5**: WebAssembly-based extension system
- **Phase 6**: Developer tools and UX polish (DevTools protocol, find-in-page, zoom)

Built with **Rust** for safety and performance, using **WebGPU/Vulkan** for rendering and **Iced** for the UI framework.

---

## ✨ Features

### Phase 1: Browser Chrome & UI Shell
- ✅ Multi-tab interface with close buttons
- ✅ Address bar with URL submission and validation
- ✅ Back/Forward/Reload navigation controls
- ✅ Responsive window layout
- ✅ Tab management (create, close, switch)

### Phase 2: Data Persistence Layer
- ✅ **History tracking** with timestamps and visit counts
- ✅ **Bookmark management** with folder organization
- ✅ **Cookie storage** with domain/path scoping and expiration
- ✅ **Session restore** capability
- ✅ Full-text search support
- ✅ Auto-cleanup of expired data

### Phase 3: Media Support
- ✅ **Audio**: MP3, WAV, OGG, FLAC, M4A playback
- ✅ **Images**: PNG, JPEG, GIF, WebP decoding with resize
- ✅ **Video**: MP4, WebM, OGV player framework
- ✅ Media controls: play, pause, seek, volume

### Phase 4: Security & Sandboxing
- ✅ **Content Security Policy (CSP)** parsing and enforcement
- ✅ **Safe Browsing** with malware/phishing detection
- ✅ **Process Isolation** (site-per-process model)
- ✅ **Sandbox Flags** for iframe restrictions
- ✅ Heuristic pattern matching for malicious URLs

### Phase 5: Extension System
- ✅ **WebAssembly Runtime** for secure extension execution
- ✅ **Extension Manifest** parsing with permissions
- ✅ **Content Script Injection** based on URL patterns
- ✅ **Host API Bridge** (tab access, console, storage)
- ✅ Permission-based security model

### Phase 6: Developer Tools & UX
- ✅ **DevTools Protocol** (DOM, Runtime, Network, Console domains)
- ✅ **Find-in-Page** with case-sensitive search and navigation
- ✅ **Viewport Controller** with zoom (10%-500%) and scroll
- ✅ **DOM Inspection** framework with box model
- ✅ Console message logging with levels

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Browser Application                      │
├─────────────────────────────────────────────────────────────┤
│  UI Shell (Iced)  │  DevTools  │  Extension Manager         │
├─────────────────────────────────────────────────────────────┤
│                    Security Layer                            │
│         CSP │ Safe Browsing │ Process Isolation             │
├─────────────────────────────────────────────────────────────┤
│                 Media & Rendering Layer                      │
│    Audio (Rodio)  │  Images  │  Video  │  GPU Compositor    │
├─────────────────────────────────────────────────────────────┤
│                  Data Persistence Layer                      │
│          SQLite Database │ Cache │ Session Store            │
├─────────────────────────────────────────────────────────────┤
│                    Network Stack                             │
│       HTTP/3 (QUIC)  │  HTTP/2  │  TLS (rustls)            │
├─────────────────────────────────────────────────────────────┤
│                   JavaScript Engine                          │
│                    QuickJS Sandbox                           │
└─────────────────────────────────────────────────────────────┘
```

---

## 📦 Installation

### Prerequisites

- Rust 1.75+ (`rustup install stable`)
- Vulkan drivers (for GPU acceleration)
- C++ toolchain (for SQLite bundling)

### Build from Source

```bash
git clone https://github.com/yourusername/glycerin.git
cd glycerin

# Build release version
cargo build --release

# Run tests
cargo test

# Launch browser
cargo run --release
```

### Dependencies

All dependencies are managed in `Cargo.toml`:

| Category | Crates |
|----------|--------|
| UI | `iced`, `iced_aw` |
| Graphics | `wgpu`, `pollster` |
| Networking | `quinn`, `h3`, `reqwest`, `rustls` |
| Database | `rusqlite` |
| Media | `rodio`, `image` |
| JS Engine | `rquickjs` |
| Extensions | `wasmtime` |
| Security | `regex`, `bitflags` |

---

## 🚀 Usage

### Running the Browser

```bash
# Launch with default settings
cargo run --release

# Launch with UI feature enabled
cargo run --release --features ui
```

### Programmatic Usage

```rust
use glycerin::*;

fn main() {
    // Initialize database
    let db = DatabaseManager::new("browser.db").unwrap();
    
    // Add history entry
    db.add_history_entry("https://example.com", "Example").unwrap();
    
    // Initialize security
    let safe_browsing = SafeBrowsingManager::new();
    assert!(safe_browsing.is_safe("https://google.com"));
    
    // Create extension engine
    let ext_engine = ExtensionEngine::new().unwrap();
    
    // Setup devtools
    let mut devtools = DevToolsSession::new();
    devtools.attach();
    
    // Initialize viewport
    let viewport = ViewportController::new(800.0, 600.0);
    viewport.zoom_in();
}
```

### Extension Development

Create an extension manifest (`manifest.json`):

```json
{
  "id": "my-extension",
  "name": "My Extension",
  "version": "1.0.0",
  "description": "A sample extension",
  "permissions": ["tabs", "storage"],
  "content_scripts": [
    {
      "matches": ["*://*.example.com/*"],
      "js_file": "script.js",
      "run_at": "document_end"
    }
  ]
}
```

---

## 📖 API Reference

### Core Modules

#### `ui_shell`
Browser chrome and tab management.

#### `data_persistence`
SQLite-backed storage layer.

#### `media_support`
Audio, video, and image handling.

#### `security`
Security policies and sandboxing.

#### `extensions`
WebAssembly extension runtime.

#### `devtools`
Developer tools protocol.

---

## 🧪 Testing

Run the comprehensive test suite:

```bash
# All tests
cargo test

# Specific module tests
cargo test security
cargo test extensions
cargo test devtools

# With output
cargo test -- --nocapture
```

### Test Coverage

- ✅ Database CRUD operations
- ✅ CSP parsing and enforcement
- ✅ Safe browsing heuristics
- ✅ Extension manifest validation
- ✅ Content script URL matching
- ✅ Find-in-page functionality
- ✅ Viewport zoom/scroll
- ✅ Media format detection

---

## 🔧 Development

### Project Structure

```
glycerin/
├── src/
│   ├── main.rs           # Entry point and FFI bridge
│   ├── ui_shell.rs       # Phase 1: Browser UI
│   ├── data_persistence.rs # Phase 2: SQLite storage
│   ├── media_support.rs  # Phase 3: Audio/Video/Images
│   ├── security.rs       # Phase 4: Security layer
│   ├── extensions.rs     # Phase 5: WASM extensions
│   └── devtools.rs       # Phase 6: DevTools
├── Cargo.toml
├── README.md
├── FEATURES.md           # Detailed feature list
├── PHASES_1_3.md         # Phases 1-3 documentation
└── PHASES_4_6.md         # Phases 4-6 documentation
```

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Follow Rust idioms and best practices
- Document all public APIs with rustdoc comments
- Include tests for new functionality
- Run `cargo fmt` and `cargo clippy` before committing

---

## 🛣️ Roadmap

### Completed (v0.18.0)
- [x] Phase 1: Browser UI Shell
- [x] Phase 2: Data Persistence
- [x] Phase 3: Media Support
- [x] Phase 4: Security & Sandboxing
- [x] Phase 5: Extension System
- [x] Phase 6: Developer Tools

### Future Enhancements (v1.0.0)
- [ ] Full CSS Flexbox/Grid layout engine
- [ ] WebGL 2.0 / WebGPU compute shaders
- [ ] WebAssembly GC support
- [ ] Service Worker implementation
- [ ] IndexedDB API
- [ ] WebRTC for real-time communication
- [ ] PDF viewer integration
- [ ] Sync across devices
- [ ] Mobile app (iOS/Android)

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [Iced](https://github.com/iced-rs/iced) - Cross-platform GUI library
- [WGPU](https://github.com/gfx-rs/wgpu) - WebGPU implementation
- [Rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings
- [Wasmtime](https://github.com/bytecodealliance/wasmtime) - WebAssembly runtime
- [QuickJS](https://bellard.org/quickjs/) - Lightweight JS engine

---

## 📞 Contact

- **Author**: Dharmendra
- **Project**: Glycerin Browser Engine
- **Version**: 0.18.0

For issues and feature requests, please use the GitHub issue tracker.

---

<div align="center">

**🌊 Built with Rust for the Modern Web**

[Report Bug](../../issues) · [Request Feature](../../issues)

</div>
