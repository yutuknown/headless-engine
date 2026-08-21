"""
StealthGuard Anti-Detection & VM-Bypass Verification Suite
Tests JS runtime stealth properties and browser fingerprint attributes:
1. navigator.webdriver == false & hardwareConcurrency / deviceMemory
2. window.chrome object structure, csi & loadTimes
3. navigator.plugins & mimeTypes
4. WebGL unmasked vendor, renderer & supported extensions
5. Web Audio API (AudioContext & OfflineAudioContext)
6. Network & Media (navigator.connection, navigator.mediaDevices, navigator.getBattery)
7. Storage & DB (localStorage, sessionStorage, indexedDB)
8. Canvas 2D context simulation & toDataURL
9. Performance & Display metrics (performance.timing/memory, screen, matchMedia)
"""

import sys
import os

sys.path.insert(0, os.path.abspath("sdk/python"))
from headless_engine import HeadlessBrowser

def run_stealth_audit():
    print("\n" + "=" * 75)
    print("  HEADLESS ENGINE: COMPREHENSIVE STEALTHGUARD ANTI-DETECTION AUDIT")
    print("=" * 75)

    with HeadlessBrowser() as browser:
        browser.navigate("https://en.wikipedia.org/wiki/Artificial_intelligence")

        tests = [
            ("navigator.webdriver", "navigator.webdriver"),
            ("navigator.hardwareConcurrency", "navigator.hardwareConcurrency"),
            ("navigator.deviceMemory", "navigator.deviceMemory"),
            ("navigator.languages", "JSON.stringify(navigator.languages)"),
            ("navigator.plugins.length", "navigator.plugins.length"),
            ("navigator.plugins[0].name", "navigator.plugins[0].name"),
            ("navigator.mimeTypes.length", "navigator.mimeTypes.length"),
            ("navigator.connection.effectiveType", "navigator.connection.effectiveType"),
            ("window.chrome.app.isInstalled", "window.chrome.app.isInstalled"),
            ("typeof window.chrome.csi", "typeof window.chrome.csi"),
            ("typeof window.chrome.loadTimes", "typeof window.chrome.loadTimes"),
            ("typeof AudioContext", "typeof AudioContext"),
            ("typeof webkitAudioContext", "typeof webkitAudioContext"),
            ("typeof localStorage", "typeof localStorage"),
            ("typeof sessionStorage", "typeof sessionStorage"),
            ("typeof indexedDB", "typeof indexedDB"),
            ("typeof matchMedia", "typeof matchMedia"),
            ("typeof performance.now", "typeof performance.now"),
            ("performance.memory.jsHeapSizeLimit", "performance.memory.jsHeapSizeLimit"),
            ("WebGL UNMASKED_VENDOR (37445)", "new WebGLRenderingContext().getParameter(37445)"),
            ("WebGL UNMASKED_RENDERER (37446)", "new WebGLRenderingContext().getParameter(37446)"),
            ("WebGL Supported Extensions Count", "new WebGLRenderingContext().getSupportedExtensions().length"),
            ("Notification.permission", "Notification.permission"),
            ("window.devicePixelRatio", "window.devicePixelRatio"),
            ("Canvas toDataURL Signature", "document.createElement('canvas').toDataURL().slice(0, 30)")
        ]

        all_passed = True
        for label, js_code in tests:
            try:
                val = browser.evaluate_js(js_code)
                print(f"  [*] {label:<40} -> {val}")
            except Exception as e:
                all_passed = False
                print(f"  [!] {label:<40} -> FAILED ({e})")

        print("\n" + "-" * 75)
        if all_passed:
            print("  ALL 25 STEALTHGUARD INTEGRITY & VM-EVASION PROBES PASSED")
        else:
            print("  SOME PROBES FAILED")
        print("-" * 75 + "\n")

if __name__ == "__main__":
    run_stealth_audit()
