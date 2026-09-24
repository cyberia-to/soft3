#!/usr/bin/env python3
"""Verify local macOS/Android artifacts; attach only our own readonly temp mount."""
import hashlib
import json
import os
from pathlib import Path
import plistlib
import subprocess
import tempfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[3]
AUDIT = Path(__file__).resolve().parent


def sha(path):
    result = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(4 * 1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def run(*args, **kwargs):
    return subprocess.check_output([str(a) for a in args], stderr=subprocess.STDOUT, **kwargs)


def main():
    cyb = ROOT / "cyb"
    version = tomllib.loads((cyb / "shell/Cargo.toml").read_text())["package"]["version"]
    binary = cyb / "target/release/cyb"
    dmg = cyb / "target/release/cyb.dmg"
    run("hdiutil", "verify", dmg)
    with tempfile.TemporaryDirectory(prefix="neuron-dmg-verify.") as work:
        mount = Path(work) / "image"
        mount.mkdir()
        attached = False
        try:
            run("hdiutil", "attach", "-readonly", "-nobrowse", "-mountpoint", mount, dmg)
            attached = True
            app = mount / "cyb.app/Contents"
            info = plistlib.loads((app / "Info.plist").read_bytes())
            assert info["CFBundleVersion"] == info["CFBundleShortVersionString"] == version
            assert info["CFBundleIdentifier"] == "ai.cyb.app"
            assert sha(app / "MacOS/cyb") == sha(binary)
            assert sha(app / "MacOS/cyb-apps/index.html") == sha(cyb / "apps/dist/index.html")
        finally:
            if attached or os.path.ismount(mount):
                run("hdiutil", "detach", mount)
    apk = cyb / "shell/gen/android/app/build/outputs/apk/release/app-release.apk"
    staged = cyb / "shell/gen/android/app/src/main/jniLibs/arm64-v8a/libcyb.so"
    with zipfile.ZipFile(apk) as archive:
        assert archive.testzip() is None
        digest = hashlib.sha256()
        with archive.open("lib/arm64-v8a/libcyb.so") as source:
            for chunk in iter(lambda: source.read(4 * 1024 * 1024), b""):
                digest.update(chunk)
        assert digest.hexdigest() == sha(staged)
    sdk = Path.home() / "Library/Android/sdk/build-tools/34.0.0"
    env = dict(os.environ)
    try:
        env["JAVA_HOME"] = run("/usr/libexec/java_home", "-v", "17").decode().strip()
    except subprocess.CalledProcessError:
        env["JAVA_HOME"] = "/opt/homebrew/opt/openjdk@17"
    assert (Path(env["JAVA_HOME"]) / "bin/java").is_file()
    run(sdk / "apksigner", "verify", apk, env=env)
    badging = run(sdk / "aapt", "dump", "badging", apk).decode()
    major, minor, patch = (int(v) for v in version.split("."))
    version_code = major * 10000 + minor * 100 + patch
    package = badging.splitlines()[0]
    assert "name='ai.cyb.app'" in package
    assert f"versionName='{version}'" in package and f"versionCode='{version_code}'" in package
    artifacts = []
    for path in [binary, dmg, apk, staged, cyb / "apps/dist/index.html"]:
        artifacts.append({"path": str(path.relative_to(ROOT)), "bytes": path.stat().st_size, "sha256": sha(path)})
    result = {
        "schema": "neuron/local-artifact-verification/1", "version": version,
        "macos": {"image_checksum": "passed", "readonly_bundle_binary_match": "passed", "bundle_version": version, "web_index_match": "passed", "owned_mount_detached": True},
        "android": {"zip_integrity": "passed", "packaged_staged_library_match": "passed", "signature_verification": "passed", "package": "ai.cyb.app", "version_name": version, "version_code": version_code},
        "artifacts": artifacts,
        "limits": ["local build; no publication or installation", "Android APK build/signature verified; no device execution claim", "DMG contents verified; no notarization claim"]
    }
    destination = AUDIT / "artifact-manifest.json"
    destination.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"result": "passed", "version": version, "artifacts": len(artifacts), "manifest": str(destination)}))


if __name__ == "__main__":
    main()
