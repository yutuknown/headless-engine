# headless-engine

Ultra-lightweight (<30MB RAM), detection-free pure-Rust headless browser engine for AI agents, web scraping, and MCP servers.

## Installation

```bash
pip install headless-engine
```

The first run automatically downloads the precompiled `headless-engine` binary for your platform from GitHub Releases. No manual setup required.

## Usage

```python
from headless_engine import HeadlessBrowser

with HeadlessBrowser() as browser:
    report = browser.navigate("https://en.wikipedia.org/wiki/Artificial_intelligence")
    markdown = browser.extract_markdown()
    print(markdown)
```

## Environment Variables

| Variable | Description |
| :--- | :--- |
| `HEADLESS_ENGINE_BIN` | Override the path to the `headless-engine` binary |

## Links

- [GitHub](https://github.com/yutuknown/headless-engine)
- [Releases](https://github.com/yutuknown/headless-engine/releases)
