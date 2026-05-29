<p align="center">
  <img src="src-tauri/icons/icon-512.png" width="120" alt="Hindsight Logo">
</p>

<h1 align="center">Hindsight</h1>

<p align="center">
  <strong>20/20 vision for your shell history</strong><br>
  <em>Never lose a command again.</em>
</p>

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-install">Install</a> •
  <a href="#-usage">Usage</a> •
  <a href="#-comparison">Comparison</a> •
  <a href="#-contributing">Contributing</a> •
  <a href="#-license">License</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.1-blue" alt="Version">
  <img src="https://img.shields.io/badge/platform-macOS-lightgrey" alt="Platform">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
  <img src="https://img.shields.io/badge/Rust-2021-orange" alt="Rust">
</p>

<p align="center">
  <a href="README.zh-CN.md">🇨🇳 简体中文</a>
</p>

---

## The Problem

You wrote a complex `ffmpeg` command three weeks ago. You need it again. But you can't remember the exact flags.

`Ctrl+R` only searches by prefix. You don't remember how the command started — you remember it had something to do with `ffmpeg` and `scale`.

**Hindsight fixes this.** It records every command with context (working directory, exit code, duration) and lets you search with full-text matching.

## Features

- **Full-text search** — Find any command by typing any word in it
- **Context-aware** — See where each command was run (working directory)
- **Failed command filter** — Quickly find what went wrong
- **Privacy-first** — 100% local, no cloud, no account needed
- **Non-invasive** — Works alongside your existing history, doesn't replace it
- **GUI App** — Beautiful macOS menu bar app with fuzzy search
- **CLI Tool** — Fast terminal interface for power users

## Install

### macOS App (Recommended)

```bash
# Download the DMG from Releases
# https://github.com/Maoshan1/hindsight/releases

# Or build from source
git clone https://github.com/Maoshan1/hindsight.git
cd hindsight
npm install
cargo install tauri-cli
npm run tauri build
```

### CLI (Homebrew)

```bash
brew tap Maoshan1/hindsight
brew install Maoshan1/hindsight/hindsight
```

### CLI (Cargo)

```bash
cargo install hindsight
```

## Setup

Add to your `~/.zshrc` (or `~/.bashrc`):

```bash
# For Homebrew install
source $(brew --prefix)/share/hindsight/hindsight.zsh

# For cargo install
source ~/.hindsight/hindsight.zsh
```

Every command you run is now automatically recorded.

## Usage

### GUI App

Click the tray icon in the menu bar to open the search window. Type to search instantly.

| Action | Shortcut |
|--------|----------|
| Open search | Click tray icon |
| Navigate results | `↑` / `↓` |
| Copy command | `Enter` on selected item |
| Close window | Click red button (app stays in tray) |
| Quit app | Right-click tray → Quit |

### CLI

```bash
# Search history
hindsight ffmpeg
hindsight "docker compose"

# Search failed commands
hindsight --exit 1

# Most used commands
hindsight top

# All recent commands
hindsight
```

## How It Works

```
┌─────────────────────────────────────────────────┐
│  Your Shell (zsh/bash)                          │
│  preexec/precmd hooks → record command + metadata│
└──────────────────────┬──────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────┐
│  SQLite Database (~/.hindsight/history.db)       │
│  - Full command text                             │
│  - Working directory                             │
│  - Exit code & duration                          │
│  - FTS5 full-text search index                   │
└──────────────────────┬──────────────────────────┘
                       │
              ┌────────┴────────┐
              ▼                 ▼
        ┌──────────┐    ┌──────────┐
        │  GUI App │    │   CLI    │
        │ (Tauri)  │    │ (Rust)   │
        └──────────┘    └──────────┘
```

## Comparison

| Feature | **Hindsight** | atuin | mcfly |
|---------|:------------:|:-----:|:-----:|
| Non-invasive (keeps existing history) | ✅ | ❌ | ❌ |
| Full-text search | ✅ | ✅ | ❌ |
| Filter by directory | ✅ | ✅ | ❌ |
| Filter by exit code | ✅ | ✅ | ❌ |
| macOS GUI app | ✅ | ❌ | ❌ |
| Privacy (no cloud required) | ✅ | ❌ | ✅ |
| Works without daemon | ✅ | ❌ | ❌ |
| Cloud sync | 🔜 | ✅ | ❌ |
| Team shared history | 🔜 | ✅ | ❌ |

**Key difference:** Hindsight doesn't replace your shell history system. It sits alongside it — zero risk, zero migration, zero config changes.

## Roadmap

- [x] CLI with full-text search
- [x] macOS GUI app
- [x] Shell hooks (zsh, bash)
- [ ] Cloud sync (optional)
- [ ] Team shared history
- [ ] AI semantic search
- [ ] Linux support
- [ ] Windows support

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

```bash
# Development
git clone https://github.com/Maoshan1/hindsight.git
cd hindsight
npm install
npm run tauri dev
```

## License

[MIT](LICENSE)
