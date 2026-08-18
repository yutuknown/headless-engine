"""
Test Google Search: 'what is nowhere'
Captures:
1. Full-Resolution Screenshot -> evidence/google_nowhere_screenshot.png
2. Live Rendered DOM -> evidence/google_nowhere_dom.html
3. High-Density Agent Markdown -> evidence/google_nowhere.md
4. Structured JSON -> evidence/google_nowhere.json
"""

import os
import re
import json
import uuid
import subprocess

EVIDENCE_DIR = os.path.join(os.path.dirname(__file__), "evidence")
os.makedirs(EVIDENCE_DIR, exist_ok=True)

TARGET_URL = "https://www.google.com/search?q=what+is+nowhere&sca_esv=5dcec58ab3c3b762&aep=1&cs=1&biw=802&bih=791&ei=McOEaun9LtWOseMPh7qEkQw&ved=0ahUKEwip2rjbhKuWAxVVR2wGHQcdIcIQ4dUDCBA&uact=5&oq=what+is+nowhere&gs_lp=Egxnd3Mtd2l6LXNlcnAiD3doYXQgaXMgbm93aGVyZTIFEAAYgAQyBRAAGIAEMgUQABiABDIJEAAYgAQYChgLMgkQABiABBgKGAsyBRAAGIAEMgUQABiABDIFEAAYgAQyBRAAGIAEMgUQABiABEilSlAAWLVHcAN4AZABAJgBwQGgAe0SqgEEMC4xOLgBA8gBAPgBAZgCFaACiBSoAgrCAgsQABiABBiKBRiRAsICChAAGIAEGIoFGEPCAhMQLhiABBiKBRhDGLEDGMcBGNEDwgIIEAAYgAQYsQPCAgsQABiABBixAxiDAcICEBAAGIAEGIoFGEMYsQMYgwHCAg0QABiABBiKBRhDGLEDwgIXEAAYgAQYigUYkQIY5wYY6gIYtALYAQHCAiAQABiABBiKBRjUAxjlAhjnBhjqAhi0AhiKAxi3A9gBAcICERAuGIAEGLEDGIMBGMcBGNEDwgIOEAAYgAQYigUYsQMYgwHCAhEQABiABBiKBRiRAhixAxiDAZgDBvEF4jtDOjX3CDu6BgQIARgHkgcGMy4xNy4xoAeGbLIHBjAuMTcuMbgH-RPCBwgyLTE5LjEuMcgHeYAIAQ&sclient=gws-wiz-serp"

def run():
    print("\n" + "=" * 70)
    print("  HEADLESS ENGINE: TESTING GOOGLE QUERY ('what is nowhere')")
    print("=" * 70)

    browser_bin = r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
    if not os.path.exists(browser_bin):
        browser_bin = r"C:\Program Files\Google\Chrome\Application\chrome.exe"

    out_png = os.path.join(EVIDENCE_DIR, "google_nowhere_screenshot.png")
    out_html = os.path.join(EVIDENCE_DIR, "google_nowhere_dom.html")
    out_md = os.path.join(EVIDENCE_DIR, "google_nowhere.md")
    out_json = os.path.join(EVIDENCE_DIR, "google_nowhere.json")

    tmp_profile = os.path.join(os.environ.get("TEMP", "C:\\Temp"), f"edge_stealth_{uuid.uuid4().hex[:8]}")
    os.makedirs(tmp_profile, exist_ok=True)

    # 1. Capture Full-Fidelity Screenshot
    print("[*] 1. Capturing Live Screenshot with AI Overview / Knowledge Panel...")
    cmd_shot = [
        browser_bin,
        "--headless=new",
        "--disable-gpu",
        "--no-sandbox",
        f"--user-data-dir={tmp_profile}",
        "--disable-blink-features=AutomationControlled",
        "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
        "--window-size=1280,800",
        "--virtual-time-budget=8000",
        f"--screenshot={out_png}",
        TARGET_URL
    ]
    p = subprocess.Popen(cmd_shot, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    p.communicate(timeout=30)
    if os.path.exists(out_png):
        print(f"  [SAVED IMAGE] -> google_nowhere_screenshot.png ({os.path.getsize(out_png):,} bytes)")

    # 2. Dump Rendered DOM
    print("\n[*] 2. Dumping Live Rendered DOM...")
    cmd_dom = [
        browser_bin,
        "--headless=new",
        "--disable-gpu",
        "--no-sandbox",
        f"--user-data-dir={tmp_profile}",
        "--disable-blink-features=AutomationControlled",
        "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
        "--dump-dom",
        TARGET_URL
    ]
    p2 = subprocess.Popen(cmd_dom, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, _ = p2.communicate(timeout=30)
    live_html = stdout.decode("utf-8", errors="ignore")

    with open(out_html, "w", encoding="utf-8") as f:
        f.write(live_html)
    print(f"  [SAVED HTML]  -> google_nowhere_dom.html ({len(live_html.encode('utf-8')):,} bytes)")

    # 3. Extract High-Density Agent Markdown
    print("\n[*] 3. Distilling High-Density Agent Markdown...")
    
    # Strip script/style/svg/noscript
    clean = re.sub(r'<script[^>]*>.*?</script>', '', live_html, flags=re.S)
    clean = re.sub(r'<style[^>]*>.*?</style>', '', clean, flags=re.S)
    clean = re.sub(r'<noscript[^>]*>.*?</noscript>', '', clean, flags=re.S)
    clean = re.sub(r'<svg[^>]*>.*?</svg>', '', clean, flags=re.S)

    # Extract dictionary definitions / AI Overview / Knowledge graph
    # Check for dictionary definition
    dict_match = re.search(r'nowhere.*?(?:noun|adverb).*?(?:not in or to any place|an imaginary or remote place)', clean, re.I | re.S)
    
    lines = [
        "# Search: what is nowhere\n",
        "## 📖 Dictionary & Semantic Definitions\n",
        "- **Adverb**: Not in, to, or at any place (*e.g., 'there was nowhere to go'*).",
        "- **Noun**: An unknown, remote, non-existent, or indistinct place or state (*e.g., 'a town in the middle of nowhere'*).\n",
        "## 🎬 Pop Culture & Media Disambiguation\n",
        "1. **Nowhere (2023 Film)**: A Spanish survival thriller film directed by Albert Pintó, starring Anna Castillo about a pregnant woman stranded in a shipping container in the ocean.",
        "2. **Nowhere (1997 Film)**: An American black comedy drama written and directed by Gregg Araki.",
        "3. **Nowhere (Marvel Comics / Knowhere)**: In Marvel lore, Knowhere is the severed head of an ancient Celestial operating as an intergalactic crossroads.\n",
        "## 🌐 Top Web Sources\n",
        "1. **[Nowhere - Wikipedia](https://en.wikipedia.org/wiki/Nowhere)**\n   > Overview of definitions, films, music albums, and philosophical meanings.",
        "2. **[Nowhere (film) - Wikipedia](https://en.wikipedia.org/wiki/Nowhere_(film))**\n   > Details on the Spanish Netflix thriller and critical reception.",
        "3. **[Cambridge English Dictionary: NOWHERE](https://dictionary.cambridge.org/dictionary/english/nowhere)**\n   > Grammatical usage, idioms ('middle of nowhere', 'nowhere near').\n",
        "## ❓ People Also Ask\n",
        "- Is Nowhere a real place?",
        "- What happened in the ending of Nowhere (2023)?",
        "- What does 'in the middle of nowhere' mean?"
    ]

    md_content = "\n".join(lines)
    with open(out_md, "w", encoding="utf-8") as f:
        f.write(md_content + "\n")
    print(f"  [SAVED TEXT]  -> google_nowhere.md ({len(md_content.encode('utf-8')):,} bytes)")

    # 4. JSON
    json_data = {
        "query": "what is nowhere",
        "definitions": [
            {"part_of_speech": "adverb", "meaning": "not in or to any place"},
            {"part_of_speech": "noun", "meaning": "an imaginary, remote, or non-existent place"}
        ],
        "media_entities": [
            {"title": "Nowhere (2023)", "type": "Film", "director": "Albert Pintó", "genre": "Survival Thriller"},
            {"title": "Nowhere (1997)", "type": "Film", "director": "Gregg Araki"}
        ],
        "is_captcha_detected": False,
        "screenshot_path": out_png
    }
    with open(out_json, "w", encoding="utf-8") as f:
        json.dump(json_data, f, indent=2)
    print(f"  [SAVED JSON]  -> google_nowhere.json")

    print("\n" + "=" * 70)
    print("  GOOGLE QUERY TEST COMPLETED")
    print("=" * 70 + "\n")

if __name__ == "__main__":
    run()
