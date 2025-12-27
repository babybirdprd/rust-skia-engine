# director-cli

Command-line interface for rendering Rhai scripts to video.

## Usage

```bash
# Render a script
director-cli script.rhai output.mp4

# With options
director-cli script.rhai output.mp4 --width 1920 --height 1080 --fps 30
```

## Installation

```bash
cargo install director-cli

# Or build from source
cargo build --release -p director-cli
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Script error |
| 2 | Render error |

---

*See [scripting-guide.md](../../docs/user/scripting-guide.md) for script syntax.*
