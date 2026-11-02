"""Download pinned, unmodified upstream installers. Never install software."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--with-sources", action="store_true", help="Also clone the pinned Sunshine source and recursive submodules")
    args = parser.parse_args()
    manifest = json.loads((ROOT / "packages-manifest.json").read_text(encoding="utf-8"))
    folder = ROOT / "packages"
    folder.mkdir(exist_ok=True)
    for item in manifest["downloads"]:
        path = folder / item["asset"]
        if path.exists() and digest(path) == item["sha256"]:
            print(f"Verified existing {path.name}", flush=True)
            continue
        url = item.get("url") or f'https://github.com/{item["repo"]}/releases/download/{item["tag"]}/{item["asset"]}'
        temporary = path.with_suffix(path.suffix + ".download")
        print(f"Downloading {path.name}", flush=True)
        request = urllib.request.Request(url, headers={"User-Agent": "HD-Screen-Cast-release-preparation"})
        with urllib.request.urlopen(request, timeout=120) as response, temporary.open("wb") as output:
            while chunk := response.read(1024 * 1024):
                output.write(chunk)
        if digest(temporary) != item["sha256"]:
            raise RuntimeError(f"SHA-256 mismatch: {item['asset']}. No installer has been run. The upstream URL may have changed; audit it before updating the manifest.")
        temporary.replace(path)
    with zipfile.ZipFile(folder / "VirtualDisplayDriver-x86.Driver.Only.zip") as archive:
        target = folder / "virtual-display-driver"
        target.mkdir(exist_ok=True)
        for name in ("MttVDD.inf", "mttvdd.cat", "MttVDD.dll", "vdd_settings.xml"):
            (target / name).write_bytes(archive.read("VirtualDisplayDriver/" + name))
    for item in manifest["files"]:
        if digest(ROOT / item["path"]) != item["sha256"]:
            raise RuntimeError(f'Extracted file verification failed: {item["path"]}')
    if args.with_sources:
        source = ROOT / ".cache" / "sunshine-source"
        if not source.exists():
            source.parent.mkdir(exist_ok=True)
            subprocess.run(["git", "-c", "core.longpaths=true", "clone", "--depth", "1", "--branch", manifest["sunshine_version"], "--recurse-submodules", "--shallow-submodules", "https://github.com/LizardByte/Sunshine.git", str(source)], check=True)
        subprocess.run(["git", "-c", "core.longpaths=true", "submodule", "update", "--init", "--recursive", "--depth", "1"], cwd=source, check=True)
    print("All installers verified. No software was installed.")


if __name__ == "__main__":
    main()
