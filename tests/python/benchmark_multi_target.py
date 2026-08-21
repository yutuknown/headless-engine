import subprocess
import time
import sys
import os
import json

sys.path.insert(0, os.path.abspath('sdk/python'))
from headless_engine import HeadlessBrowser

targets = [
    ('Hacker News', 'https://news.ycombinator.com/'),
    ('Wikipedia (Quantum Computing)', 'https://en.wikipedia.org/wiki/Quantum_computing'),
    ('Google Search (Regular)', 'https://www.google.com/search?q=quantum+computing'),
    ('Google AI Mode (udm=50)', 'https://www.google.com/search?q=quantum+computing&udm=50&hl=en')
]

def main():
    print("==========================================================================")
    print("  LIVE EMPIRICAL BENCHMARK: LIGHTPANDA vs HEADLESS ENGINE (MULTI-TARGET)  ")
    print("==========================================================================\n")
    
    summary = []
    
    for name, url in targets:
        print(f"=== TARGET: {name} ({url}) ===")
        
        # 1. Lightpanda
        t0 = time.perf_counter()
        wsl_cmd = f"/mnt/c/Users/abhis/OneDrive/Documents/Antigravity/engine/LightPanda/lightpanda-x86_64-linux fetch --dump --wait-ms 2000 '{url}'"
        proc = subprocess.Popen(['wsl', '-d', 'Debian', '--', 'bash', '-c', wsl_cmd], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, errors='ignore')
        out, err = proc.communicate(timeout=20)
        lp_time = int((time.perf_counter() - t0) * 1000)
        lp_blocked = 'recaptcha' in out.lower() or 'unusual traffic' in out.lower()
        
        # 2. Headless Engine
        t0 = time.perf_counter()
        with HeadlessBrowser() as browser:
            nav = browser.navigate(url)
            md = browser.extract_markdown()
            entities = browser.extract_results()
            he_time = int((time.perf_counter() - t0) * 1000)
            he_blocked = nav.get('is_captcha_detected', False)

        ent_count = len(entities.get('entities', [])) if isinstance(entities, dict) else len(entities)

        row = {
            "target": name,
            "url": url,
            "lightpanda": {
                "time_ms": lp_time,
                "html_bytes": len(out.encode('utf-8')),
                "blocked_captcha": lp_blocked
            },
            "headless_engine": {
                "time_ms": he_time,
                "status": nav.get("status"),
                "markdown_bytes": len(md.encode('utf-8')),
                "entities_count": ent_count,
                "blocked_captcha": he_blocked
            }
        }
        summary.append(row)
        
        print(f"  [Lightpanda]      Latency: {lp_time}ms | HTML: {len(out):,} bytes | Blocked/Captcha: {lp_blocked}")
        print(f"  [Headless Engine] Latency: {he_time}ms | Status: {nav.get('status')} | Markdown: {len(md):,} bytes | Entities: {ent_count} | Blocked: {he_blocked}")
        print()

    with open("LightPanda/multi_target_benchmark.json", "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2)

if __name__ == "__main__":
    main()
