"""
Headless Engine Python SDK
Ultra-lightweight (<30MB RAM), detection-free headless browser for AI agents and web scraping.
"""

import json
import os
import shutil
import subprocess
from typing import Any, Dict, List, Optional


class HeadlessBrowser:
    """Synchronous & context-managed client for Headless Engine."""

    def __init__(self, binary_path: Optional[str] = None):
        self.binary_path = binary_path or self._find_binary()
        if not self.binary_path or not os.path.exists(self.binary_path):
            # Check PATH
            which_bin = shutil.which("headless-engine")
            if which_bin:
                self.binary_path = which_bin
            else:
                raise FileNotFoundError(
                    "Could not locate `headless-engine` binary. Please install or provide `binary_path`."
                )

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

    def _find_binary(self) -> Optional[str]:
        # Search relative release/debug paths
        candidates = [
            "headless-engine",
            "headless-engine.exe",
            "./target/release/headless-engine",
            "./target/release/headless-engine.exe",
            "./target/debug/headless-engine",
            "./target/debug/headless-engine.exe",
            "../target/release/headless-engine",
            "../target/release/headless-engine.exe",
        ]
        for c in candidates:
            if os.path.exists(c):
                return os.path.abspath(c)
        return None

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


__all__ = ["HeadlessBrowser"]
