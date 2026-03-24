# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Pranslator is a lightweight translation app built with Tauri 2.0 + React for EN↔CN bidirectional translation using LLM APIs.

## Development Commands

```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm dev

# Build for production
pnpm tauri build

# Linting
pnpm lint          # Check code
pnpm lint:fix      # Auto-fix issues

# Formatting
pnpm format        # Format code
pnpm format:check  # Check formatting
```

## Architecture

### Frontend (React + TypeScript)

- **Entry**: `src/main.tsx` → `src/App.tsx`
- **State Management**: Zustand (`src/stores/settings.ts`)
- **API Layer**: `src/api/translate.ts` - wraps Tauri commands via `invoke()`
- **Components**: `src/components/` - TranslationPanel, SettingsPanel, ShortcutInput
- **Types**: `src/types/index.ts` - shared TypeScript interfaces

### Backend (Rust + Tauri)

- **Entry**: `src-tauri/src/main.rs` → `src-tauri/src/lib.rs`
- **Config**: `src-tauri/src/config/settings.rs` - TOML-based settings management
- **LLM Client**: `src-tauri/src/llm/client.rs` - OpenAI-compatible API calls
- **Commands**: `src-tauri/src/commands/` - Tauri command handlers

### Frontend-Backend Communication

Frontend calls Rust backend via Tauri's `invoke()`:
- `translate(request)` - translate text
- `get_config()` - get current settings
- `set_config(newSettings)` - update settings
- `validate_shortcut(shortcut)` - validate shortcut string

## Key Patterns

### **Don't** Hold A `pnpm dev` In Backstage
Never try to start a dev server in your backstage, I will start it manually.

### Check After Modify
When finish a task, run Linting, Formatting and `cargo check` to ensure your change do not introduce warnings or syntax issues, if exist, fix it until theree are no issues.

### Custom Hook Pattern
`TranslationPanel()` is a custom hook that returns state and handlers, separating logic from rendering (`TranslationView`).

### Settings State
Zustand store with async load/update that syncs with Rust backend.

### Global Shortcuts
Handled in `lib.rs` via `tauri-plugin-global-shortcut`. Shortcut strings follow format like `"Alt+Shift+T"`.

### Leader Key Shortcuts
Add app shortcuts via `src/config/shortcuts.ts` config and `App.tsx` handlers; never use raw `window.addEventListener`.

### System Tray
Window close is intercepted to hide instead of quit. Double-click tray icon shows window.

## Code Style

- Single quotes, semicolons, 2-space indentation (see `.prettierrc`)
- Import order enforced: builtin → external → internal (see `eslint.config.js`)
- Conventional Commits: `type(scope): description` (see `DEVELOPMENT.md`)
