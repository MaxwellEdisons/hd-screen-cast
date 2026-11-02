"""Read-only integrity and privacy checks for generated release archives."""
import gzip
import hashlib
import json
from pathlib import Path
import zipfile

root = Path(__file__).resolve().parents[1]
manifest = json.loads((root / "packages-manifest.json").read_text(encoding="utf-8"))
for path in sorted((root / "dist").glob("*.zip")):
    with zipfile.ZipFile(path) as archive:
        assert archive.testzip() is None
        names = set(archive.namelist())
        assert {"HD-Screen-Cast.exe", "README.md", "README.zh-CN.md", "LICENSE", "THIRD_PARTY_NOTICES.md"}.issubset(names)
        assert not any("credentials" in name.lower() or "sunshine_state" in name.lower() or "preferences.json" in name.lower() for name in names)
        if "Offline" in path.name:
            for item in manifest["files"]:
                assert hashlib.sha256(archive.read(item["path"])).hexdigest() == item["sha256"]
        print("Verified:", path.name, len(names), "entries", flush=True)
source = next((root / "dist").glob("*.tar.gz"))
with gzip.open(source, "rb") as stream:
    while stream.read(1024 * 1024):
        pass
print("Verified source gzip integrity", flush=True)
