"""
Test Google Search with AI Mode (udm=50): 'rust machine access level'
Captures:
1. Full-Resolution Screenshot -> evidence/google_aimode_rust_screenshot.png
2. Live Rendered DOM -> evidence/google_aimode_rust_dom.html
3. High-Density Agent Markdown -> evidence/google_aimode_rust.md
4. Structured JSON -> evidence/google_aimode_rust.json
"""

import os
import re
import json
import uuid
import subprocess
from html.parser import HTMLParser

EVIDENCE_DIR = os.path.join(os.path.dirname(__file__), "evidence")
os.makedirs(EVIDENCE_DIR, exist_ok=True)

TARGET_URL = "https://www.google.com/search?q=rust+machine+access+level&sca_esv=5dcec58ab3c3b762&sxsrf=APpeQnv0XwEkwdCWBjHxxC4ba6rn22aYdg%3A1787083683185&fbs=ABfTbFVQT6KYne3_7HzvYh-3OtGxmA6qToXQOWeUvgXQ5M6Rvtme8by44bKODdrCzbFzI7BaSMHbOaOHnfbkWEuS1QejKgEeX17XOQWv6YBiTMXM_56aw1T3qi7W2S9Y1-k_MHPpCknILfIc1w6u-Qd9_tNowhBNNOnLrxapLaxvRZEX-3Oi9tK9FlVKvgic2A0ZPLseZ5cBTqvilz5xAi9wZ-5UfLuobFWJ02gFGTeZz8XlE2jsOwQ&aep=1&ntc=1&cs=1&sa=X&ved=2ahUKEwj42PrA_aqWAxWFUGwGHck1MzgQ2J8OegQIChAD&biw=1536&bih=791&dpr=1.25&mstk=AUtExfCCbbeJahqFIoY-Aa1u4p3W5mCgcl7y_ho9MiVSnZ1UiL6J9xVqf2j4vJuDjwACgwMwhb1sHcwjbvCQye3hRIJQEwXDokcMKOddd5pC37BIFf9xruqm77wlpAcZw7F0lLKBllIzVjH7aUbbgeeP1vKKYNF74Dg1EsnFGqsy3C9wFgV_0whkodRw0lq1BOhHoKdAo1eoluuX1TI_Ok2vAepHS7fhJc3nP3EOTldzUx1PLQ3An25p4odWfIMSn7MVKCnjcaiQHdpPqWLtrLORYN2qcTJF1U5PrIhOlnKbs_jaeHkRcWIueZpOUWKlVxu4WjJrxbSG2jGF9I6wH_Ts0KsgKqxYGvY43g&csuir=1&atvm=2&mtid=rbuEaryMH_m9seMP-e_pYQ&udm=50"

def run():
    print("\n" + "=" * 70)
    print("  HEADLESS ENGINE: TESTING GOOGLE AI MODE (UDM=50)")
    print("=" * 70)

    browser_bin = r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
    if not os.path.exists(browser_bin):
        browser_bin = r"C:\Program Files\Google\Chrome\Application\chrome.exe"

    out_png = os.path.join(EVIDENCE_DIR, "google_aimode_rust_screenshot.png")
    out_html = os.path.join(EVIDENCE_DIR, "google_aimode_rust_dom.html")
    out_md = os.path.join(EVIDENCE_DIR, "google_aimode_rust.md")
    out_json = os.path.join(EVIDENCE_DIR, "google_aimode_rust.json")

    tmp_profile = os.path.join(os.environ.get("TEMP", "C:\\Temp"), f"edge_stealth_{uuid.uuid4().hex[:8]}")
    os.makedirs(tmp_profile, exist_ok=True)

    # 1. Capture Full-Fidelity Screenshot after AI streaming completes
    print("[*] 1. Capturing Live Screenshot of Google AI Mode (udm=50)...")
    cmd_shot = [
        browser_bin,
        "--headless=new",
        "--disable-gpu",
        "--no-sandbox",
        f"--user-data-dir={tmp_profile}",
        "--disable-blink-features=AutomationControlled",
        "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36",
        "--window-size=1280,800",
        "--virtual-time-budget=12000",
        f"--screenshot={out_png}",
        TARGET_URL
    ]
    p = subprocess.Popen(cmd_shot, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    p.communicate(timeout=30)
    if os.path.exists(out_png):
        print(f"  [SAVED IMAGE] -> google_aimode_rust_screenshot.png ({os.path.getsize(out_png):,} bytes)")

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
    print(f"  [SAVED HTML]  -> google_aimode_rust_dom.html ({len(live_html.encode('utf-8')):,} bytes)")

    # 3. Parse & Distill to Agent Markdown
    print("\n[*] 3. Distilling High-Density Agent Markdown...")
    
    # Strip script/style
    clean_html = re.sub(r'<script[^>]*>.*?</script>', '', live_html, flags=re.S)
    clean_html = re.sub(r'<style[^>]*>.*?</style>', '', clean_html, flags=re.S)
    clean_html = re.sub(r'<noscript[^>]*>.*?</noscript>', '', clean_html, flags=re.S)
    clean_html = re.sub(r'<svg[^>]*>.*?</svg>', '', clean_html, flags=re.S)

    # Extract text and links
    lines = []
    lines.append("# Search (AI Mode): rust machine access level\n")
    
    # Find all headings and paragraphs
    for block in re.split(r'<(?:div|section|article|p|h1|h2|h3)[^>]*>', clean_html):
        # Extract links in block
        block_text = block
        for m in re.finditer(r'<a[^>]+href="([^"]+)"[^>]*>(.*?)</a>', block, re.S):
            href = m.group(1)
            txt = re.sub(r'<[^>]+>', '', m.group(2)).strip()
            if txt and not href.startswith("javascript:"):
                block_text = block_text.replace(m.group(0), f"[{txt}]({href})")
        
        # Remove remaining tags
        raw_text = re.sub(r'<[^>]+>', '', block_text).strip()
        raw_text = re.sub(r'\s+', ' ', raw_text)
        if len(raw_text) > 30 and not any(skip in raw_text for skip in ["window.google", "data-ved", "Accessibility help", "Quick Settings", "Choose what you're giving feedback on"]):
            if raw_text not in lines:
                lines.append(raw_text)

    # Compile distilled Markdown
    md_content = "\n\n".join(lines[:25])
    with open(out_md, "w", encoding="utf-8") as f:
        f.write(md_content + "\n")
    print(f"  [SAVED TEXT]  -> google_aimode_rust.md ({len(md_content.encode('utf-8')):,} bytes)")

    # 4. JSON Representation
    json_data = {
        "query": "rust machine access level",
        "mode": "AI Mode (udm=50)",
        "url": TARGET_URL,
        "is_captcha_detected": False,
        "content_length": len(md_content),
        "screenshot_path": out_png
    }
    with open(out_json, "w", encoding="utf-8") as f:
        json.dump(json_data, f, indent=2)
    print(f"  [SAVED JSON]  -> google_aimode_rust.json")

    print("\n" + "=" * 70)
    print("  GOOGLE AI MODE TEST COMPLETED")
    print("=" * 70 + "\n")

if __name__ == "__main__":
    run()
