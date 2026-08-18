import { spawn, ChildProcess } from 'child_process';
import * as readline from 'readline';
import * as fs from 'fs';
import * as path from 'path';

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

export class HeadlessBrowser {
  private process: ChildProcess;
  private rl: readline.Interface;
  private reqId: number = 0;
  private pendingCallbacks: Map<number, { resolve: (val: any) => void; reject: (err: Error) => void }> = new Map();

  constructor(binaryPath?: string) {
    const bin = binaryPath || this.findBinary();
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

  private findBinary(): string {
    const candidates = [
      'headless-engine',
      'headless-engine.exe',
      './target/release/headless-engine',
      './target/release/headless-engine.exe',
      './target/debug/headless-engine',
      './target/debug/headless-engine.exe',
      '../target/release/headless-engine',
      '../target/release/headless-engine.exe',
    ];
    for (const c of candidates) {
      if (fs.existsSync(c)) {
        return path.resolve(c);
      }
    }
    return 'headless-engine';
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
