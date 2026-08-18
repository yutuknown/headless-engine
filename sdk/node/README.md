# headless-engine

Ultra-lightweight (<30MB RAM), detection-free pure-Rust headless browser engine for AI agents, web scraping, and MCP servers.

## Installation

```bash
npm install headless-engine
```

The first run automatically downloads the precompiled `headless-engine` binary for your platform from GitHub Releases. No manual setup required.

## Usage

```typescript
import { HeadlessBrowser } from 'headless-engine';

const browser = new HeadlessBrowser();
const report = await browser.navigate('https://news.ycombinator.com');
const markdown = await browser.extractMarkdown();
console.log(markdown);
browser.close();
```

## Environment Variables

| Variable | Description |
| :--- | :--- |
| `HEADLESS_ENGINE_BIN` | Override the path to the `headless-engine` binary |

## Links

- [GitHub](https://github.com/yutuknown/headless-engine)
- [Releases](https://github.com/yutuknown/headless-engine/releases)
