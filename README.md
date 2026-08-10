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
  <a href="#-settings-and-data">Settings</a> •
  <a href="#-comparison">Comparison</a> •
  <a href="#-contributing">Contributing</a> •
  <a href="#-license">License</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.2-blue" alt="Version">
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
- **GUI App** — macOS menu bar app with keyboard-friendly search
- **Settings** — Pause recording and choose how long records are kept
- **History controls** — Clear all Hindsight records with an explicit confirmation
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

The build creates both bundles on Apple Silicon:

- `src-tauri/target/release/bundle/macos/Hindsight.app`
- `src-tauri/target/release/bundle/dmg/Hindsight_0.1.2_aarch64.dmg`

The GUI and CLI share the same local database. To record commands through the
shell hooks, install the CLI as well (for a local checkout, use
`cargo install --path .`).

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

Add the matching hook to your `~/.zshrc` (or `~/.bashrc`):

```bash
# For Homebrew install
source $(brew --prefix)/share/hindsight/hindsight.zsh

# For a source checkout
mkdir -p ~/.hindsight
cp hindsight.zsh ~/.hindsight/hindsight.zsh
source ~/.hindsight/hindsight.zsh
```

For Bash, use `hindsight.bash` in the same way. The hook calls the installed
`hindsight` CLI; the macOS GUI reads the records written by that CLI.

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
hindsight search --exit 1

# Most used commands
hindsight top

# All recent commands
hindsight
```

## Settings and data

Open the gear icon in the app to manage:

- **Record commands** — Pause or resume new records without removing the shell hooks.
- **Keep history** — Keep records forever, or retain 7, 30, 90 days, or 1 year.
  Older records are pruned when settings are saved or a new command is recorded.
- **Clear all** — Permanently remove every record stored by Hindsight. This does
  not modify your shell's own history file (such as `~/.zsh_history`).

Hindsight stores everything locally:

```text
~/.hindsight/history.db       # command history and search index
~/.hindsight/settings.json    # recording and retention preferences
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

The GUI's Settings panel controls recording and retention without changing the
shell hook files. Clearing history removes rows from the SQLite database and
rebuilds its search index.

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

Run the checks before opening a pull request:

```bash
cargo test
cargo check
cargo test --manifest-path src-tauri/Cargo.toml
npx vite build
```

`npm run tauri build` builds the macOS `.app` and `.dmg` bundles. DMG creation
requires macOS tooling such as `hdiutil`.

## License

[MIT](LICENSE)
