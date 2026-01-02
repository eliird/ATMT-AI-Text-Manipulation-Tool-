# Translate Tool

A lightweight system tray application that translates selected text using AI with a simple hotkey.

## Build

```bash
cargo build --release
```

The executable will be located at `target/release/translate_tool.exe`

## Usage

1. Run the application - it will appear in your system tray
2. Select any text in any application
3. Press `Ctrl+Shift+Q` to translate
4. The translated text will replace your selection

### Configuration

Right-click the tray icon and select "Edit Config" to customize:

- `api_url` - API endpoint (default: http://llm-api.fixstars.com/v1)
- `api_key` - API key if required
- `model` - Model name (default: latest-chat)
- `prompt` - Translation instruction (default: English ↔ Japanese)

Configuration is saved to `%APPDATA%\translate\translate_tool\config.json`
