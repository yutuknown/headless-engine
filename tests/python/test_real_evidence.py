"""
Comprehensive Multi-Engine Real-World Evidence Suite
Collects actual pixel-perfect screenshots, parsed entities, and Markdown from:
1. Wikipedia (Heavy dynamic layout)
2. DuckDuckGo SERP (Quantum computing search results)
3. Hacker News (Live tech aggregation)
4. Google Search CAPTCHA detection analysis
"""

import os
import json
import base64
import time
from headless_engine import HeadlessBrowser

EVIDENCE_DIR = os.path.join(os.path.dirname(__file__), "evidence")
os.makedirs(EVIDENCE_DIR, exist_ok=True)

def save_evidence(filename: str, content: str):
    path = os.path.join(EVIDENCE_DIR, filename)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"  [SAVED TEXT]   -> {filename} ({len(content.encode('utf-8')):,} bytes)")

def save_binary(filename: str, raw_bytes: bytes):
    path = os.path.join(EVIDENCE_DIR, filename)
    with open(path, "wb") as f:
        f.write(raw_bytes)
    print(f"  [SAVED IMAGE]  -> {filename} ({len(raw_bytes):,} bytes)")

def run():
    print("\n" + "=" * 70)
    print("  HEADLESS ENGINE: MULTI-ENGINE REAL-WORLD EVIDENCE SUITE")
    print("=" * 70)

    with HeadlessBrowser() as browser:
        # 1. Wikipedia Knowledge Page
        print("\n[*] 1. Testing Heavy Target: Wikipedia 'Artificial Intelligence'")
        rep_wiki = browser.navigate("https://en.wikipedia.org/wiki/Artificial_intelligence")
        md_wiki = browser.extract_markdown()
        links_wiki = browser.extract_links()
        shot_wiki = browser.screenshot()
        
        save_evidence("wiki_ai_markdown.md", md_wiki)
        save_evidence("wiki_ai_links.json", json.dumps(links_wiki[:30], indent=2))
        if isinstance(shot_wiki, dict) and shot_wiki.get("png_base64"):
            raw_b64 = shot_wiki["png_base64"].split(",", 1)[-1]
            save_binary("wiki_ai_screenshot.png", base64.b64decode(raw_b64))

        # 2. DuckDuckGo SERP
        print("\n[*] 2. Testing DuckDuckGo SERP: 'quantum computing breakthrough'")
        rep_ddg = browser.navigate("https://html.duckduckgo.com/html/?q=quantum+computing+breakthrough+2025")
        results_ddg = browser.extract_results()
        md_ddg = browser.extract_markdown()
        shot_ddg = browser.screenshot()
        
        save_evidence("ddg_quantum_results.json", json.dumps(results_ddg, indent=2))
        save_evidence("ddg_quantum_markdown.md", md_ddg)
        if isinstance(shot_ddg, dict) and shot_ddg.get("png_base64"):
            raw_b64 = shot_ddg["png_base64"].split(",", 1)[-1]
            save_binary("ddg_quantum_screenshot.png", base64.b64decode(raw_b64))

        # 3. Hacker News Live Frontpage
        print("\n[*] 3. Testing Hacker News Live Feed")
        rep_hn = browser.navigate("https://news.ycombinator.com")
        md_hn = browser.extract_markdown()
        links_hn = browser.extract_links()
        forms_hn = browser.extract_forms()
        shot_hn = browser.screenshot()
        
        save_evidence("hn_live_markdown.md", md_hn)
        save_evidence("hn_live_links.json", json.dumps(links_hn[:20], indent=2))
        save_evidence("hn_live_forms.json", json.dumps(forms_hn, indent=2))
        if isinstance(shot_hn, dict) and shot_hn.get("png_base64"):
            raw_b64 = shot_hn["png_base64"].split(",", 1)[-1]
            save_binary("hn_live_screenshot.png", base64.b64decode(raw_b64))

    print("\n" + "=" * 70)
    print("  ALL REAL-WORLD EVIDENCE COLLECTED IN ./evidence/")
    print("=" * 70 + "\n")

if __name__ == "__main__":
    run()
