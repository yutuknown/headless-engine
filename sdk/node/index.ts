import { spawn, ChildProcess, execSync } from 'child_process';
import * as readline from 'readline';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import * as https from 'https';

export const VERSION = '1.0.0';
export const REPO = 'yutuknown/headless-engine';

export interface NavigationReport {
  status: number;
  requested_url: string;
  final_url: string;
  page_title: string;
  is_captcha_detected: boolean;
  html_bytes: number;
}

export interface LinkInfo {
  text: string;
  href: string;
}

export interface FormInfo {
  action: string;
  method: string;
  inputs: Array<{
    name: string;
    input_type: string;
    value: string;
    placeholder: string;
  }>;
}

export interface SearchResults {
  page_title: string;
  ai_overview?: { summary: string; source_references: string[] };
  knowledge_panel?: { title: string; subtitle: string; description: string; attributes: Array<[string, string]> };
  video_results: Array<{ title: string; video_id: string; url: string; channel: string; duration: string }>;
  news_results: Array<{ headline: string; source: string; time_ago: string; link: string }>;
  image_results: Array<{ title: string; image_url: string; source_url: string; domain: string }>;
  organic_results: Array<{ title: string; link: string; snippet: string }>;
  total_results_found: number;
}

function getCacheDir(): string {
  if (process.platform === 'win32') {
    const base = process.env.LOCALAPPDATA || os.homedir();
    return path.join(base, 'headless-engine', 'bin', `v${VERSION}`);
  } else {
    return path.join(os.homedir(), '.cache', 'headless-engine', 'bin', `v${VERSION}`);
  }
}

function detectAssetName(): string {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'win32') {
    return 'headless-engine-windows-x86_64.zip';
  } else if (platform === 'darwin') {
    if (arch === 'arm64') {
      return 'headless-engine-macos-arm64.tar.gz';
    } else {
      return 'headless-engine-macos-x86_64.tar.gz';
    }
  } else if (platform === 'linux') {
    if (arch === 'arm64' || arch === 'arm') {
      return 'headless-engine-linux-arm64.tar.gz';
    } else {
      return 'headless-engine-linux-x86_64.tar.gz';
    }
  }

  throw new Error(`Unsupported platform/architecture: ${platform} (${arch}). Please build from source.`);
}

function downloadFile(url: string, dest: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const request = (targetUrl: string) => {
      https.get(targetUrl, { headers: { 'User-Agent': 'headless-engine-node-sdk' } }, (res) => {
        if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          request(res.headers.location);
          return;
        }
        if (res.statusCode !== 200) {
          reject(new Error(`Failed to download binary from ${targetUrl} (Status: ${res.statusCode})`));
          return;
        }
        const file = fs.createWriteStream(dest);
        res.pipe(file);
        file.on('finish', () => {
          file.close();
          resolve();
        });
      }).on('error', (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
    };
    request(url);
  });
}

async function downloadAndExtract(cacheDir: string): Promise<string> {
  fs.mkdirSync(cacheDir, { recursive: true });
  const assetName = detectAssetName();
  const downloadUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${assetName}`;
  const exeName = process.platform === 'win32' ? 'headless-engine.exe' : 'headless-engine';
  const targetBin = path.join(cacheDir, exeName);

  process.stderr.write(`[headless-engine] Downloading precompiled engine binary from ${downloadUrl}...\n`);

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'headless-engine-'));
  const archivePath = path.join(tmpDir, assetName);

  try {
    await downloadFile(downloadUrl, archivePath);

    if (assetName.endsWith('.zip')) {
      if (process.platform === 'win32') {
        execSync(`powershell -NoProfile -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${tmpDir}' -Force"`);
      } else {
        execSync(`unzip -q "${archivePath}" -d "${tmpDir}"`);
      }
    } else {
      execSync(`tar -xzf "${archivePath}" -C "${tmpDir}"`);
    }

    let extractedBin = path.join(tmpDir, exeName);
    if (!fs.existsSync(extractedBin)) {
      // Find recursively
      const findBin = (dir: string): string | null => {
        const entries = fs.readdirSync(dir, { withFileTypes: true });
        for (const entry of entries) {
          const fullPath = path.join(dir, entry.name);
          if (entry.isDirectory()) {
            const found = findBin(fullPath);
            if (found) return found;
          } else if (entry.name === exeName) {
            return fullPath;
          }
        }
        return null;
      };
      const found = findBin(tmpDir);
      if (found) extractedBin = found;
    }

    if (!fs.existsSync(extractedBin)) {
      throw new Error(`Failed to find ${exeName} inside downloaded archive.`);
    }

    fs.copyFileSync(extractedBin, targetBin);
    if (process.platform !== 'win32') {
      fs.chmodSync(targetBin, 0o755);
    }

    process.stderr.write(`[headless-engine] Binary installed successfully to ${targetBin}\n`);
    return targetBin;
  } finally {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {}
  }
}

export function resolveBinary(explicitPath?: string): string {
  if (explicitPath && fs.existsSync(explicitPath)) {
    return path.resolve(explicitPath);
  }

  if (process.env.HEADLESS_ENGINE_BIN && fs.existsSync(process.env.HEADLESS_ENGINE_BIN)) {
    return path.resolve(process.env.HEADLESS_ENGINE_BIN);
  }

  const localCandidates = [
    'headless-engine',
    'headless-engine.exe',
    './target/release/headless-engine',
    './target/release/headless-engine.exe',
    './target/debug/headless-engine',
    './target/debug/headless-engine.exe',
    '../target/release/headless-engine',
    '../target/release/headless-engine.exe',
    '../../target/release/headless-engine',
    '../../target/release/headless-engine.exe',
  ];
  for (const c of localCandidates) {
    if (fs.existsSync(c)) {
      return path.resolve(c);
    }
  }

  const cacheDir = getCacheDir();
  const exeName = process.platform === 'win32' ? 'headless-engine.exe' : 'headless-engine';
  const cachedBin = path.join(cacheDir, exeName);
  if (fs.existsSync(cachedBin)) {
    return cachedBin;
  }

  // Check system path
  try {
    const cmd = process.platform === 'win32' ? 'where.exe headless-engine' : 'which headless-engine';
    const whichOut = execSync(cmd, { stdio: ['ignore', 'pipe', 'ignore'], encoding: 'utf-8' }).trim();
    if (whichOut && fs.existsSync(whichOut.split('\n')[0].trim())) {
      return whichOut.split('\n')[0].trim();
    }
  } catch {}

  // Synchronously download if missing
  try {
    const assetName = detectAssetName();
    const downloadUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${assetName}`;
    fs.mkdirSync(cacheDir, { recursive: true });

    process.stderr.write(`[headless-engine] Fetching engine binary from ${downloadUrl}...\n`);
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'headless-engine-'));
    const archivePath = path.join(tmpDir, assetName);

    if (process.platform === 'win32') {
      execSync(`powershell -NoProfile -Command "Invoke-WebRequest -Uri '${downloadUrl}' -OutFile '${archivePath}'; Expand-Archive -Path '${archivePath}' -DestinationPath '${tmpDir}' -Force"`);
    } else {
      execSync(`curl -fsSL "${downloadUrl}" -o "${archivePath}" && tar -xzf "${archivePath}" -C "${tmpDir}"`);
    }

    const extracted = path.join(tmpDir, exeName);
    if (fs.existsSync(extracted)) {
      fs.copyFileSync(extracted, cachedBin);
      if (process.platform !== 'win32') fs.chmodSync(cachedBin, 0o755);
      return cachedBin;
    }
  } catch (err) {
    process.stderr.write(`[headless-engine] Auto-download warning: ${err}\n`);
  }

  return 'headless-engine';
}

export class HeadlessBrowser {
  private process: ChildProcess;
  private rl: readline.Interface;
  private reqId: number = 0;
  private pendingCallbacks: Map<number, { resolve: (val: any) => void; reject: (err: Error) => void }> = new Map();

  constructor(binaryPath?: string) {
    const bin = resolveBinary(binaryPath);
    this.process = spawn(bin, ['--stdio'], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    this.rl = readline.createInterface({
      input: this.process.stdout!,
      crlfDelay: Infinity,
    });

    this.rl.on('line', (line: string) => {
      if (!line.trim()) return;
      try {
        const resp = JSON.parse(line);
        const id = resp.id;
        const cb = this.pendingCallbacks.get(id);
        if (cb) {
          this.pendingCallbacks.delete(id);
          if (resp.error) {
            cb.reject(new Error(`[${resp.error.code}] ${resp.error.message}`));
          } else {
            cb.resolve(resp.result);
          }
        }
      } catch (e) {
        // ignore JSON parse errors on malformed lines
      }
    });
  }

  private call<T>(method: string, params: Record<string, any> = {}): Promise<T> {
    return new Promise((resolve, reject) => {
      this.reqId += 1;
      const id = this.reqId;
      this.pendingCallbacks.set(id, { resolve, reject });

      const payload = JSON.stringify({
        jsonrpc: '2.0',
        id,
        method,
        params,
      }) + '\n';

      this.process.stdin!.write(payload);
    });
  }

  public async navigate(url: string, tabId?: string): Promise<NavigationReport> {
    return this.call<NavigationReport>('tab.navigate', { url, tab_id: tabId });
  }

  public async extractMarkdown(selector?: string, tabId?: string): Promise<string> {
    const res = await this.call<{ markdown: string }>('tab.extractMarkdown', { selector, tab_id: tabId });
    return res.markdown;
  }

  public async extractResults(tabId?: string): Promise<SearchResults> {
    return this.call<SearchResults>('tab.extractResults', { tab_id: tabId });
  }

  public async extractLinks(tabId?: string): Promise<LinkInfo[]> {
    const res = await this.call<{ links: LinkInfo[] }>('tab.extractLinks', { tab_id: tabId });
    return res.links;
  }

  public async extractForms(tabId?: string): Promise<FormInfo[]> {
    const res = await this.call<{ forms: FormInfo[] }>('tab.extractForms', { tab_id: tabId });
    return res.forms;
  }

  public async click(target: string, tabId?: string): Promise<any> {
    return this.call('tab.click', { target, tab_id: tabId });
  }

  public async typeText(selector: string, text: string, tabId?: string): Promise<string> {
    const res = await this.call<{ status: string }>('tab.type', { selector, text, tab_id: tabId });
    return res.status;
  }

  public async evaluateJs(code: string, tabId?: string): Promise<string> {
    const res = await this.call<{ result: string }>('tab.evaluateJs', { code, tab_id: tabId });
    return res.result;
  }

  public async createTab(profile?: string): Promise<string> {
    const res = await this.call<{ tab_id: string }>('engine.createTab', { profile });
    return res.tab_id;
  }

  public async closeTab(tabId: string): Promise<boolean> {
    const res = await this.call<{ closed: boolean }>('engine.closeTab', { tab_id: tabId });
    return res.closed;
  }

  public close() {
    try {
      this.call('shutdown', {});
    } catch {}
    this.rl.close();
    this.process.kill();
  }
}
