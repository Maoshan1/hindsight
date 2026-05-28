# hindsight

> 20/20 vision for your shell history.

`hindsight` is a shell history search tool that **doesn't replace your history system**. It runs alongside your existing `~/.zsh_history` — just makes it searchable.

## Install

```bash
# Build from source
git clone https://github.com/yourname/hindsight.git
cd hindsight
cargo install --path .

# Or via cargo
cargo install hindsight
```

## Setup

Add to your `~/.zshrc`:

```bash
source /path/to/hindsight.zsh
```

That's it. Every command you run is now recorded.

## Usage

```bash
# Search history
hindsight ffmpeg
hindsight "docker compose"

# Filter by directory
hindsight --cwd ~/projects npm

# Filter by exit code (failed commands only)
hindsight --exit 1

# Top 20 most used commands
hindsight top

# Stats
hindsight stats

# Recent commands
hindsight
```

## How it works

1. Shell hooks (`preexec`/`precmd`) record every command with metadata (cwd, exit code, duration)
2. Data is stored in a local SQLite database at `~/.hindsight/history.db`
3. Full-text search powered by SQLite FTS5

## vs atuin vs mcfly

| Feature | hindsight | atuin | mcfly |
|---------|-----------|-------|-------|
| Replaces history | No | Yes | Yes |
| Full-text search | Yes | Yes | No (fuzzy) |
| Filter by cwd | Yes | Yes | No |
| Filter by exit code | Yes | Yes | No |
| Cloud sync | No (planned) | Yes | No |
| Privacy | Local only | Cloud | Local only |

## License

MIT
