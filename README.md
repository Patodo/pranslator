# Pranslator

A fast, lightweight translation app built with Rust + Tauri 2.0 + React.

## Features

- **Translation**: EN<->CN bidirectional translation via LLM API
- **Settings**: Configurable API key, base URL, and model
- **System Tray**: Background running with tray icon
- **Global Shortcuts**: (Planned) System-wide hotkeys
- **History**: (Planned) SQLite-based translation history

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | React 18 + TypeScript + Zustand |
| Backend | Rust + Tauri 2.0 |
| HTTP Client | reqwest |
| Config | TOML |

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri build
```

## Configuration

The app stores settings in:
- **Windows**: `%APPDATA%/pranslator/settings.toml`
- **macOS**: `~/Library/Application Support/pranslator/settings.toml`
- **Linux**: `~/.config/pranslator/settings.toml`

### Example settings.toml

```toml
[llm]
api_key = "sk-your-api-key"
api_base = "https://api.openai.com/v1"
model = "gpt-4o-mini"
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT
