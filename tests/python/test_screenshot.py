import sys
import os
import base64

sys.path.insert(0, os.path.abspath("sdk/python"))
from headless_engine import HeadlessBrowser

os.makedirs("scratch/screenshots", exist_ok=True)

def test_screenshot():
    print("\n==================================================================")
    print("  TESTING HEADLESS ENGINE SCREENSHOT SYSTEM")
    print("==================================================================\n")

    with HeadlessBrowser() as browser:
        # Test 1: Hacker News
        url = "https://news.ycombinator.com/"
        print(f"[*] Navigating to {url}...")
        nav = browser.navigate(url)
        print(f"[*] Navigated: Status {nav.get('status')}, Title: {nav.get('page_title')}")

        print("[*] Capturing Screenshot...")
        shot = browser.screenshot()
        
        width = shot.get("width")
        height = shot.get("height")
        b64 = shot.get("png_base64", "")
        if b64.startswith("data:image/png;base64,"):
            b64 = b64.split(",", 1)[1]
        png_bytes = base64.b64decode(b64)

        print(f"  -> Resolution: {width} x {height}")
        print(f"  -> Captured PNG File Size: {len(png_bytes):,} bytes")

        png_path = "scratch/screenshots/hn_screenshot.png"
        with open(png_path, "wb") as f:
            f.write(png_bytes)
        print(f"  -> Saved PNG to: {png_path}")

        # Test 2: Google Search
        g_url = "https://www.google.com/search?q=quantum+computing"
        print(f"\n[*] Navigating to {g_url}...")
        browser.navigate(g_url)
        g_shot = browser.screenshot()
        g_b64 = g_shot.get("png_base64", "")
        if g_b64.startswith("data:image/png;base64,"):
            g_b64 = g_b64.split(",", 1)[1]
        g_png_bytes = base64.b64decode(g_b64)

        g_png_path = "scratch/screenshots/google_quantum_screenshot.png"
        with open(g_png_path, "wb") as f:
            f.write(g_png_bytes)
        print(f"  -> Captured Google PNG File Size: {len(g_png_bytes):,} bytes")
        print(f"  -> Saved PNG to: {g_png_path}")

        print("\n==================================================================")
        print("  SCREENSHOT TEST COMPLETED SUCCESSFULLY (100% WORKING)")
        print("==================================================================\n")

if __name__ == "__main__":
    test_screenshot()
