"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.HeadlessBrowser = exports.REPO = exports.VERSION = void 0;
exports.resolveBinary = resolveBinary;
const child_process_1 = require("child_process");
const readline = __importStar(require("readline"));
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const os = __importStar(require("os"));
const https = __importStar(require("https"));
exports.VERSION = '1.0.0';
exports.REPO = 'yutuknown/headless-engine';
function getCacheDir() {
    if (process.platform === 'win32') {
        const base = process.env.LOCALAPPDATA || os.homedir();
        return path.join(base, 'headless-engine', 'bin', `v${exports.VERSION}`);
    }
    else {
        return path.join(os.homedir(), '.cache', 'headless-engine', 'bin', `v${exports.VERSION}`);
    }
}
function detectAssetName() {
    const platform = os.platform();
    const arch = os.arch();
    if (platform === 'win32') {
        return 'headless-engine-windows-x86_64.zip';
    }
    else if (platform === 'darwin') {
        if (arch === 'arm64') {
            return 'headless-engine-macos-arm64.tar.gz';
        }
        else {
            return 'headless-engine-macos-x86_64.tar.gz';
        }
    }
    else if (platform === 'linux') {
        if (arch === 'arm64') {
            return 'headless-engine-linux-arm64.tar.gz';
        }
        else {
            return 'headless-engine-linux-x86_64.tar.gz';
        }
    }
    throw new Error(`Unsupported platform/architecture: ${platform} (${arch}). Please build from source.`);
}
function downloadFile(url, dest) {
    return new Promise((resolve, reject) => {
        const request = (targetUrl) => {
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
                fs.unlink(dest, () => { });
                reject(err);
            });
        };
        request(url);
    });
}
async function downloadAndExtract(cacheDir) {
    fs.mkdirSync(cacheDir, { recursive: true });
    const assetName = detectAssetName();
    const downloadUrl = `https://github.com/${exports.REPO}/releases/download/v${exports.VERSION}/${assetName}`;
    const exeName = process.platform === 'win32' ? 'headless-engine.exe' : 'headless-engine';
    const targetBin = path.join(cacheDir, exeName);
    process.stderr.write(`[headless-engine] Downloading precompiled engine binary from ${downloadUrl}...\n`);
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'headless-engine-'));
    const archivePath = path.join(tmpDir, assetName);
    try {
        await downloadFile(downloadUrl, archivePath);
        if (assetName.endsWith('.zip')) {
            if (process.platform === 'win32') {
                (0, child_process_1.execSync)(`powershell -NoProfile -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${tmpDir}' -Force"`);
            }
            else {
                (0, child_process_1.execSync)(`unzip -q "${archivePath}" -d "${tmpDir}"`);
            }
        }
        else {
            (0, child_process_1.execSync)(`tar -xzf "${archivePath}" -C "${tmpDir}"`);
        }
        let extractedBin = path.join(tmpDir, exeName);
        if (!fs.existsSync(extractedBin)) {
            // Find recursively
            const findBin = (dir) => {
                const entries = fs.readdirSync(dir, { withFileTypes: true });
                for (const entry of entries) {
                    const fullPath = path.join(dir, entry.name);
                    if (entry.isDirectory()) {
                        const found = findBin(fullPath);
                        if (found)
                            return found;
                    }
                    else if (entry.name === exeName) {
                        return fullPath;
                    }
                }
                return null;
            };
            const found = findBin(tmpDir);
            if (found)
                extractedBin = found;
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
    }
    finally {
        try {
            fs.rmSync(tmpDir, { recursive: true, force: true });
        }
        catch { }
    }
}
function resolveBinary(explicitPath) {
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
        const whichOut = (0, child_process_1.execSync)(cmd, { stdio: ['ignore', 'pipe', 'ignore'], encoding: 'utf-8' }).trim();
        if (whichOut && fs.existsSync(whichOut.split('\n')[0].trim())) {
            return whichOut.split('\n')[0].trim();
        }
    }
    catch { }
    // Synchronously download if missing
    try {
        const assetName = detectAssetName();
        const downloadUrl = `https://github.com/${exports.REPO}/releases/download/v${exports.VERSION}/${assetName}`;
        fs.mkdirSync(cacheDir, { recursive: true });
        process.stderr.write(`[headless-engine] Fetching engine binary from ${downloadUrl}...\n`);
        const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'headless-engine-'));
        const archivePath = path.join(tmpDir, assetName);
        if (process.platform === 'win32') {
            (0, child_process_1.execSync)(`powershell -NoProfile -Command "Invoke-WebRequest -Uri '${downloadUrl}' -OutFile '${archivePath}'; Expand-Archive -Path '${archivePath}' -DestinationPath '${tmpDir}' -Force"`);
        }
        else {
            (0, child_process_1.execSync)(`curl -fsSL "${downloadUrl}" -o "${archivePath}" && tar -xzf "${archivePath}" -C "${tmpDir}"`);
        }
        const extracted = path.join(tmpDir, exeName);
        if (fs.existsSync(extracted)) {
            fs.copyFileSync(extracted, cachedBin);
            if (process.platform !== 'win32')
                fs.chmodSync(cachedBin, 0o755);
            return cachedBin;
        }
    }
    catch (err) {
        process.stderr.write(`[headless-engine] Auto-download warning: ${err}\n`);
    }
    return 'headless-engine';
}
class HeadlessBrowser {
    process;
    rl;
    reqId = 0;
    pendingCallbacks = new Map();
    constructor(binaryPath) {
        const bin = resolveBinary(binaryPath);
        this.process = (0, child_process_1.spawn)(bin, ['--stdio'], {
            stdio: ['pipe', 'pipe', 'pipe'],
        });
        this.rl = readline.createInterface({
            input: this.process.stdout,
            crlfDelay: Infinity,
        });
        this.rl.on('line', (line) => {
            if (!line.trim())
                return;
            try {
                const resp = JSON.parse(line);
                const id = resp.id;
                const cb = this.pendingCallbacks.get(id);
                if (cb) {
                    this.pendingCallbacks.delete(id);
                    if (resp.error) {
                        cb.reject(new Error(`[${resp.error.code}] ${resp.error.message}`));
                    }
                    else {
                        cb.resolve(resp.result);
                    }
                }
            }
            catch (e) {
                // ignore JSON parse errors on malformed lines
            }
        });
    }
    call(method, params = {}) {
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
            this.process.stdin.write(payload);
        });
    }
    async navigate(url, tabId) {
        return this.call('tab.navigate', { url, tab_id: tabId });
    }
    async extractMarkdown(selector, tabId) {
        const res = await this.call('tab.extractMarkdown', { selector, tab_id: tabId });
        return res.markdown;
    }
    async extractResults(tabId) {
        return this.call('tab.extractResults', { tab_id: tabId });
    }
    async extractLinks(tabId) {
        const res = await this.call('tab.extractLinks', { tab_id: tabId });
        return res.links;
    }
    async extractForms(tabId) {
        const res = await this.call('tab.extractForms', { tab_id: tabId });
        return res.forms;
    }
    async click(target, tabId) {
        return this.call('tab.click', { target, tab_id: tabId });
    }
    async typeText(selector, text, tabId) {
        const res = await this.call('tab.type', { selector, text, tab_id: tabId });
        return res.status;
    }
    async evaluateJs(code, tabId) {
        const res = await this.call('tab.evaluateJs', { code, tab_id: tabId });
        return res.result;
    }
    async createTab(profile) {
        const res = await this.call('engine.createTab', { profile });
        return res.tab_id;
    }
    async closeTab(tabId) {
        const res = await this.call('engine.closeTab', { tab_id: tabId });
        return res.closed;
    }
    close() {
        try {
            this.call('shutdown', {});
        }
        catch { }
        this.rl.close();
        this.process.kill();
    }
}
exports.HeadlessBrowser = HeadlessBrowser;
