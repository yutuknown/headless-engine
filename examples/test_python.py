import sys
import os

# Add sdk/python to sys.path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "sdk", "python")))

from headless_engine import HeadlessBrowser

def main():
    print(">>> Initializing Headless Engine Python SDK...")
    with HeadlessBrowser() as browser:
        print("[1] Navigating to Hacker News...")
        report = browser.navigate("https://news.ycombinator.com/")
        print(f"  * Status: {report['status']}")
        print(f"  * Title:  {report['page_title']}")

        print("\n[2] Extracting clean LLM Markdown...")
        md = browser.extract_markdown()
        print(f"  * Markdown Size: {len(md)} bytes")
        print(f"  * Markdown Preview:\n{md[:300]}...")

        print("\n[3] Extracting Actionable Links...")
        links = browser.extract_links()
        print(f"  * Total Links Found: {len(links)}")
        for i, link in enumerate(links[:3]):
            print(f"    [{i+1}] {link['text']} -> {link['href']}")

        print("\n>>> Python SDK Test Passed Successfully!")

if __name__ == "__main__":
    main()
