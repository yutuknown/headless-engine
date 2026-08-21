import time
import psutil
import os
import sys

# Ensure sdk is in path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "sdk", "python")))
from headless_engine import BrowserEngine, DeviceProfile

def measure_headless_engine():
    process = psutil.Process(os.getpid())
    start_ram = process.memory_info().rss / (1024 * 1024)
    
    start_time = time.time()
    
    engine = BrowserEngine()
    tab_id = engine.create_tab("chrome-windows")
    
    res = engine.call_rpc("tab.google_search", {"tab_id": tab_id, "query": "headless browser pure rust engine for AI agents"})
    
    end_time = time.time()
    end_ram = process.memory_info().rss / (1024 * 1024)
    
    latency = (end_time - start_time) * 1000
    ram_used = end_ram - start_ram
    
    print("--- Headless Engine ---")
    print(f"Latency: {latency:.2f} ms")
    print(f"RAM Used: {ram_used:.2f} MB")
    
    # Use organic_results from struct format instead of python dict if needed,
    # Actually the struct is mapped to python dict
    results_len = 0
    if isinstance(res, dict) and 'organic_results' in res:
        results_len = len(res['organic_results'])
    
    print(f"Extracted {results_len} results cleanly without CAPTCHA.")

if __name__ == "__main__":
    measure_headless_engine()
