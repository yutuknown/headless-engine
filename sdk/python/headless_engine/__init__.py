"""
Headless Engine Python SDK
Ultra-lightweight (<30MB RAM), detection-free headless browser for AI agents and web scraping.
"""

import json
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import zipfile
from typing import Any, Dict, List, Optional

VERSION = "1.0.0"
REPO = "yutuknown/headless-engine"


def _get_cache_dir() -> str:
    if platform.system() == "Windows":
        base = os.environ.get("LOCALAPPDATA") or os.path.expanduser("~")
        return os.path.join(base, "headless-engine", "bin", f"v{VERSION}")
    else:
        return os.path.expanduser(f"~/.cache/headless-engine/bin/v{VERSION}")


def _detect_asset_name() -> str:
    system = platform.system().lower()
    machine = platform.machine().lower()

    is_arm = machine in ("arm64", "aarch64")
    is_x86 = machine in ("x86_64", "amd64", "x64")

    if system == "windows":
        if is_x86 or is_arm:  # Windows ARM runs x86_64 via emulation
            return "headless-engine-windows-x86_64.zip"
    elif system == "darwin":
        if is_arm:
            return "headless-engine-macos-arm64.tar.gz"
        elif is_x86:
            return "headless-engine-macos-x86_64.tar.gz"
    elif system == "linux":
        if is_arm:
            return "headless-engine-linux-arm64.tar.gz"
        elif is_x86:
            return "headless-engine-linux-x86_64.tar.gz"

    raise RuntimeError(
        f"Unsupported platform or architecture: {system} ({machine}). "
        "Please build from source: https://github.com/yutuknown/headless-engine"
    )


def _download_and_extract(cache_dir: str) -> str:
    os.makedirs(cache_dir, exist_ok=True)
    asset_name = _detect_asset_name()
    download_url = f"https://github.com/{REPO}/releases/download/v{VERSION}/{asset_name}"
    exe_name = "headless-engine.exe" if platform.system() == "Windows" else "headless-engine"
    target_bin = os.path.join(cache_dir, exe_name)

    sys.stderr.write(f"[headless-engine] Downloading precompiled engine binary from {download_url}...\n")
    sys.stderr.flush()

    with tempfile.TemporaryDirectory() as tmp_dir:
        archive_path = os.path.join(tmp_dir, asset_name)
        req = urllib.request.Request(
            download_url,
            headers={"User-Agent": "headless-engine-python-sdk"},
        )
        with urllib.request.urlopen(req) as resp, open(archive_path, "wb") as f:
            shutil.copyfileobj(resp, f)

        if asset_name.endswith(".zip"):
            with zipfile.ZipFile(archive_path, "r") as z:
                z.extractall(tmp_dir)
        else:
            with tarfile.open(archive_path, "r:*") as t:
                t.extractall(tmp_dir)

        extracted_bin = os.path.join(tmp_dir, exe_name)
        if not os.path.exists(extracted_bin):
            for root, _, files in os.walk(tmp_dir):
                if exe_name in files:
                    extracted_bin = os.path.join(root, exe_name)
                    break

        if not os.path.exists(extracted_bin):
            raise RuntimeError(f"Failed to find {exe_name} inside downloaded archive {asset_name}")

        shutil.move(extracted_bin, target_bin)

    if platform.system() != "Windows":
        os.chmod(target_bin, 0o755)

    sys.stderr.write(f"[headless-engine] Binary installed successfully to {target_bin}\n")
    sys.stderr.flush()
    return target_bin


def _resolve_binary(explicit_path: Optional[str] = None) -> str:
    if explicit_path and os.path.exists(explicit_path):
        return os.path.abspath(explicit_path)

    env_path = os.environ.get("HEADLESS_ENGINE_BIN")
    if env_path and os.path.exists(env_path):
        return os.path.abspath(env_path)

    which_bin = shutil.which("headless-engine") or shutil.which("headless-engine.exe")
    if which_bin and os.path.exists(which_bin):
        return os.path.abspath(which_bin)

    local_candidates = [
        "headless-engine",
        "headless-engine.exe",
        "./target/release/headless-engine",
        "./target/release/headless-engine.exe",
        "./target/debug/headless-engine",
        "./target/debug/headless-engine.exe",
        "../target/release/headless-engine",
        "../target/release/headless-engine.exe",
        "../../target/release/headless-engine",
        "../../target/release/headless-engine.exe",
    ]
    for c in local_candidates:
        if os.path.exists(c):
            return os.path.abspath(c)

    cache_dir = _get_cache_dir()
    exe_name = "headless-engine.exe" if platform.system() == "Windows" else "headless-engine"
    cached_bin = os.path.join(cache_dir, exe_name)
    if os.path.exists(cached_bin):
        return cached_bin

    return _download_and_extract(cache_dir)


class HeadlessBrowser:
    """Synchronous & context-managed client for Headless Engine."""

    def __init__(self, binary_path: Optional[str] = None):
        self.binary_path = _resolve_binary(binary_path)

        self._process = subprocess.Popen(
            [self.binary_path, "--stdio"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            errors="replace",
            bufsize=1,
        )
        self._req_id = 0

    def _call(self, method: str, params: Optional[Dict[str, Any]] = None) -> Any:
        self._req_id += 1
        payload = {
            "jsonrpc": "2.0",
            "id": self._req_id,
            "method": method,
            "params": params or {},
        }
        json_line = json.dumps(payload) + "\n"
        self._process.stdin.write(json_line)
        self._process.stdin.flush()

        response_line = self._process.stdout.readline()
        if not response_line:
            raise RuntimeError("Headless engine process terminated unexpectedly.")

        resp = json.loads(response_line)
        if "error" in resp and resp["error"] is not None:
            err = resp["error"]
            raise RuntimeError(f"Engine RPC Error [{err.get('code')}]: {err.get('message')}")

        return resp.get("result")

    def navigate(self, url: str, tab_id: Optional[str] = None) -> Dict[str, Any]:
        """Navigates to a URL with full anti-detection HTTP/2 & stealth fingerprint."""
        params: Dict[str, Any] = {"url": url}
        if tab_id:
            params["tab_id"] = tab_id
        return self._call("tab.navigate", params)

    def set_content(self, html: str, url: Optional[str] = None, tab_id: Optional[str] = None) -> Dict[str, Any]:
        """Sets HTML content directly on the active tab for offline parsing."""
        params: Dict[str, Any] = {"html": html}
        if url:
            params["url"] = url
        if tab_id:
            params["tab_id"] = tab_id
        return self._call("tab.setContent", params)

    def set_html(self, html: str, url: Optional[str] = None, tab_id: Optional[str] = None) -> Dict[str, Any]:
        """Alias for set_content."""
        return self.set_content(html, url, tab_id)

    def observe(self, tab_id: Optional[str] = None) -> Dict[str, Any]:
        """Returns the full agent observation (indexed action tree, page title, URL, and summary)."""
        params: Dict[str, Any] = {}
        if tab_id:
            params["tab_id"] = tab_id
        return self._call("tab.observe", params)

    def screenshot(self, tab_id: Optional[str] = None) -> Dict[str, Any]:
        """Captures a pure-Rust vector SVG screenshot and ASCII visual layout wireframe."""
        params: Dict[str, Any] = {}
        if tab_id:
            params["tab_id"] = tab_id
        return self._call("tab.screenshot", params)

    def screenshot_svg(self, tab_id: Optional[str] = None) -> str:
        """Returns the raw vector SVG markup of the page."""
        res = self.screenshot(tab_id)
        return res.get("svg", "") if isinstance(res, dict) else ""

    def screenshot_layout(self, tab_id: Optional[str] = None) -> str:
        """Returns a dense text/ASCII wireframe layout snapshot of the page."""
        res = self.screenshot(tab_id)
        return res.get("layout_wireframe", "") if isinstance(res, dict) else ""

    def extract_markdown(self, selector: Optional[str] = None, tab_id: Optional[str] = None) -> str:
        """Extracts clean, token-efficient LLM Markdown (~80% token compression)."""
        params: Dict[str, Any] = {}
        if selector:
            params["selector"] = selector
        if tab_id:
            params["tab_id"] = tab_id
        res = self._call("tab.extractMarkdown", params)
        return res.get("markdown", "") if isinstance(res, dict) else str(res)

    def extract_results(self, tab_id: Optional[str] = None) -> Dict[str, Any]:
        """Extracts structured multi-modal search results (AI Overview, news, videos, images)."""
        params: Dict[str, Any] = {}
        if tab_id:
            params["tab_id"] = tab_id
        return self._call("tab.extractResults", params)

    def extract_links(self, tab_id: Optional[str] = None) -> List[Dict[str, str]]:
        """Extracts all actionable links with anchor text and absolute URLs."""
        params: Dict[str, Any] = {}
        if tab_id:
            params["tab_id"] = tab_id
        res = self._call("tab.extractLinks", params)
        return res.get("links", []) if isinstance(res, dict) else []

    def extract_forms(self, tab_id: Optional[str] = None) -> List[Dict[str, Any]]:
        """Extracts interactive form schemas and input field attributes."""
        params: Dict[str, Any] = {}
        if tab_id:
            params["tab_id"] = tab_id
        res = self._call("tab.extractForms", params)
        return res.get("forms", []) if isinstance(res, dict) else []

    def click(self, target: str, tab_id: Optional[str] = None) -> Dict[str, Any]:
        """Simulates clicking a link or button and automatically navigates if it's a link."""
        params: Dict[str, Any] = {"target": target}
        if tab_id:
            params["tab_id"] = tab_id
        return self._call("tab.click", params)

    def type_text(self, selector: str, text: str, tab_id: Optional[str] = None) -> str:
        """Injects text into a form input or DOM element."""
        params: Dict[str, Any] = {"selector": selector, "text": text}
        if tab_id:
            params["tab_id"] = tab_id
        res = self._call("tab.type", params)
        return res.get("status", "") if isinstance(res, dict) else str(res)

    def evaluate_js(self, code: str, tab_id: Optional[str] = None) -> str:
        """Evaluates sandboxed JavaScript in the pure-Rust Boa runtime."""
        params: Dict[str, Any] = {"code": code}
        if tab_id:
            params["tab_id"] = tab_id
        res = self._call("tab.evaluateJs", params)
        return res.get("result", "") if isinstance(res, dict) else str(res)

    def create_tab(self, profile: Optional[str] = None) -> str:
        """Spawns an isolated tab instance and returns its tab_id."""
        params: Dict[str, Any] = {}
        if profile:
            params["profile"] = profile
        res = self._call("engine.createTab", params)
        return res.get("tab_id", "")

    def close_tab(self, tab_id: str) -> bool:
        """Closes an active tab instance."""
        res = self._call("engine.closeTab", {"tab_id": tab_id})
        return res.get("closed", False)

    def close(self):
        """Terminates the engine subprocess gracefully."""
        if self._process and self._process.poll() is None:
            try:
                self._call("shutdown")
            except Exception:
                pass
            self._process.terminate()
            self._process.wait(timeout=2)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()


__all__ = ["HeadlessBrowser", "VERSION"]
