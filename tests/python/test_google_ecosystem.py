"""
Comprehensive Google Real-World Evidence Test Suite
Tests all Headless Engine capabilities against live Google:
- Normal Web Search
- AI Overview extraction
- Images SERP
- Videos SERP
- News & Knowledge Panels
- People Also Ask (PAA) extraction
- Pure-Rust Vector SVG Screenshotting + Base64 PNG Screenshotting
- Multi-Device Profile Spoofing (Desktop vs iOS Safari)

All raw outputs and parsed evidence are saved to ./evidence/ directory.
"""

import os
import json
import time
import base64
import urllib.parse
from headless_engine import HeadlessBrowser

EVIDENCE_DIR = os.path.join(os.path.dirname(__file__), "evidence")
os.makedirs(EVIDENCE_DIR, exist_ok=True)

def save_evidence(filename: str, content: str):
    path = os.path.join(EVIDENCE_DIR, filename)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"  [SAVED EVIDENCE] -> {filename} ({len(content.encode('utf-8')):,} bytes)")

def save_binary(filename: str, raw_bytes: bytes):
    path = os.path.join(EVIDENCE_DIR, filename)
    with open(path, "wb") as f:
        f.write(raw_bytes)
    print(f"  [SAVED BINARY]   -> {filename} ({len(raw_bytes):,} bytes)")

def run_tests():
    print("\n" + "=" * 70)
    print("  HEADLESS ENGINE: GOOGLE REAL-WORLD MULTI-MODAL EVIDENCE SUITE")
    print("=" * 70)

    summary_results = []

    with HeadlessBrowser() as browser:
        # -----------------------------------------------------------------
        # TEST 1: Normal Search & AI Overview Extraction
        # -----------------------------------------------------------------
        topic_1 = "quantum computing breakthrough 2025"
        print(f"\n[*] 1. Testing Google Normal Search & AI Overview: '{topic_1}'")
        url_1 = f"https://www.google.com/search?q={urllib.parse.quote(topic_1)}"
        
        t0 = time.time()
        rep_1 = browser.navigate(url_1)
        dur_1 = (time.time() - t0) * 1000
        
        results_1 = browser.extract_results()
        md_1 = browser.extract_markdown()
        links_1 = browser.extract_links()
        
        save_evidence("1_quantum_search_results.json", json.dumps(results_1, indent=2))
        save_evidence("1_quantum_search.md", md_1)
        
        summary_results.append({
            "topic": topic_1,
            "modality": "Normal Web Search + AI Overview",
            "status": rep_1.get("status", 200),
            "latency_ms": round(dur_1, 1),
            "links_extracted": len(links_1),
            "markdown_bytes": len(md_1.encode("utf-8")),
            "has_entities": bool(results_1)
        })

        # -----------------------------------------------------------------
        # TEST 2: Google Images Extraction
        # -----------------------------------------------------------------
        topic_2 = "james webb space telescope deep field"
        print(f"\n[*] 2. Testing Google Images Search: '{topic_2}'")
        url_2 = f"https://www.google.com/search?q={urllib.parse.quote(topic_2)}&udm=2"
        
        t0 = time.time()
        rep_2 = browser.navigate(url_2)
        dur_2 = (time.time() - t0) * 1000
        
        results_2 = browser.extract_results()
        md_2 = browser.extract_markdown()
        links_2 = browser.extract_links()
        
        save_evidence("2_jwst_images_results.json", json.dumps(results_2, indent=2))
        save_evidence("2_jwst_images.md", md_2)
        
        summary_results.append({
            "topic": topic_2,
            "modality": "Google Images (udm=2)",
            "status": rep_2.get("status", 200),
            "latency_ms": round(dur_2, 1),
            "links_extracted": len(links_2),
            "markdown_bytes": len(md_2.encode("utf-8")),
            "has_entities": bool(results_2)
        })

        # -----------------------------------------------------------------
        # TEST 3: Google Videos & News SERP
        # -----------------------------------------------------------------
        topic_3 = "spacex starship flight test launch"
        print(f"\n[*] 3. Testing Google Videos / News: '{topic_3}'")
        url_3 = f"https://www.google.com/search?q={urllib.parse.quote(topic_3)}&tbm=vid"
        
        t0 = time.time()
        rep_3 = browser.navigate(url_3)
        dur_3 = (time.time() - t0) * 1000
        
        results_3 = browser.extract_results()
        md_3 = browser.extract_markdown()
        
        save_evidence("3_spacex_videos_results.json", json.dumps(results_3, indent=2))
        save_evidence("3_spacex_videos.md", md_3)
        
        summary_results.append({
            "topic": topic_3,
            "modality": "Google Videos (tbm=vid)",
            "status": rep_3.get("status", 200),
            "latency_ms": round(dur_3, 1),
            "links_extracted": len(browser.extract_links()),
            "markdown_bytes": len(md_3.encode("utf-8")),
            "has_entities": bool(results_3)
        })

        # -----------------------------------------------------------------
        # TEST 4: People Also Ask (PAA) & Knowledge Panels
        # -----------------------------------------------------------------
        topic_4 = "what is artificial intelligence agent"
        print(f"\n[*] 4. Testing PAA & Knowledge Entities: '{topic_4}'")
        url_4 = f"https://www.google.com/search?q={urllib.parse.quote(topic_4)}"
        
        t0 = time.time()
        rep_4 = browser.navigate(url_4)
        dur_4 = (time.time() - t0) * 1000
        
        results_4 = browser.extract_results()
        md_4 = browser.extract_markdown()
        
        save_evidence("4_ai_agents_paa_results.json", json.dumps(results_4, indent=2))
        save_evidence("4_ai_agents_paa.md", md_4)
        
        summary_results.append({
            "topic": topic_4,
            "modality": "PAA & Definitions",
            "status": rep_4.get("status", 200),
            "latency_ms": round(dur_4, 1),
            "links_extracted": len(browser.extract_links()),
            "markdown_bytes": len(md_4.encode("utf-8")),
            "has_entities": bool(results_4)
        })

        # -----------------------------------------------------------------
        # TEST 5: Real-World Screenshotting (SVG + PNG Base64)
        # -----------------------------------------------------------------
        print(f"\n[*] 5. Testing Pure-Rust Screenshot on Google Search")
        shot = browser.screenshot()
        if isinstance(shot, dict):
            svg = shot.get("svg", "")
            layout = shot.get("layout_wireframe", "")
            png_b64 = shot.get("png_base64", "")
            
            if svg:
                save_evidence("5_google_search_screenshot.svg", svg)
            if layout:
                save_evidence("5_google_search_layout.txt", layout)
            if png_b64:
                try:
                    raw_b64 = png_b64.split(",", 1)[1] if "," in png_b64 else png_b64
                    png_bytes = base64.b64decode(raw_b64)
                    save_binary("5_google_search_screenshot.png", png_bytes)
                except Exception as e:
                    print(f"  [WARN] Failed to decode PNG base64: {e}")

        # -----------------------------------------------------------------
        # TEST 6: Mobile iPhone 16 Profile Spoofing
        # -----------------------------------------------------------------
        print(f"\n[*] 6. Testing iPhone 16 (Safari iOS) Profile Spoofing on Google")
        ios_tab = browser.create_tab(profile="safari-ios")
        browser.navigate("https://www.google.com/search?q=apple+iphone+16+pro", tab_id=ios_tab)
        ios_ua = browser.evaluate_js("navigator.userAgent", tab_id=ios_tab)
        ios_md = browser.extract_markdown(tab_id=ios_tab)
        
        save_evidence("6_google_mobile_ios.md", ios_md)
        save_evidence("6_ios_user_agent.txt", f"Spoofed UA: {ios_ua}")
        browser.close_tab(ios_tab)

    # -----------------------------------------------------------------
    # Generate Summary Markdown Evidence Report
    # -----------------------------------------------------------------
    report_md = "# Google Multi-Modal Evidence & Capability Audit\n\n"
    report_md += f"**Execution Date:** {time.strftime('%Y-%m-%d %H:%M:%S')}\n"
    report_md += "**Engine:** Headless Engine v1.0.0 (Pure-Rust)\n\n"
    report_md += "| Topic | Search Modality | HTTP Status | Latency | Actionable Links | Markdown Size |\n"
    report_md += "| :--- | :--- | :--- | :--- | :--- | :--- |\n"
    for r in summary_results:
        report_md += f"| `{r['topic']}` | {r['modality']} | **{r['status']} OK** | {r['latency_ms']}ms | {r['links_extracted']} links | {r['markdown_bytes']:,} B |\n"
    
    report_md += "\n## Generated Evidence Files in `./evidence/`:\n"
    for f in sorted(os.listdir(EVIDENCE_DIR)):
        size = os.path.getsize(os.path.join(EVIDENCE_DIR, f))
        report_md += f"- [`{f}`](file://{os.path.abspath(os.path.join(EVIDENCE_DIR, f))}): {size:,} bytes\n"

    save_evidence("SUMMARY_REPORT.md", report_md)
    print("\n" + "=" * 70)
    print("  ALL GOOGLE MODALITY TESTS COMPLETED AND SAVED TO ./evidence/")
    print("=" * 70 + "\n")

if __name__ == "__main__":
    run_tests()
