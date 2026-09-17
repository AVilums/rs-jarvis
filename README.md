# Jarvis

A small streaming terminal client for an LLM for easy, quick access.

## Run

```powershell
$env:OPENAI_API_KEY = "your-api-key"
cargo run
```

On first run, Jarvis creates `config.toml` in the operating system's user config
directory. On Windows this is normally `%APPDATA%\Roaming\jarvis\config\config.toml`.
Values in a `.jarvis.toml` in the current directory override that user config.

```toml
default = "openai"

[profiles.openai]
connector = "openai"
model = "gpt-5.2"
api_key_env = "OPENAI_API_KEY"
```

On Windows, Jarvis runs as an icon in the notification area and opens its
terminal on launch. Closing the terminal leaves Jarvis running. Press **Shift+J**
anywhere, left-click the tray icon, or choose **Open Jarvis** from its menu to
open the terminal again. Choose **Exit** from the tray menu to stop Jarvis.

Commands inside the chat are `/clear`, `/hide`, and `/exit`. `/exit` closes the
current terminal session; the tray host remains available on Windows.
