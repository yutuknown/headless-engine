from headless_engine import HeadlessBrowser

print("Testing HeadlessBrowser Python SDK from PyPI...")
try:
    with HeadlessBrowser() as browser:
        print("Browser spawned successfully!")
        res = browser.navigate("https://news.ycombinator.com")
        print("Page title:", res.get("title") or res.get("url"))
        md = browser.extract_markdown()
        print("Extracted Markdown preview:\n", md[:300])
        print("\nPython SDK Test: SUCCESS!")
except Exception as e:
    print(f"Encountered: {e}")
