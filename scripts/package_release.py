"""Create release ZIPs and corresponding-source archive; never build or install."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def git(source: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-c", "core.longpaths=true", *args], cwd=source, text=True, encoding="utf-8").strip()


def collect_licenses(destination: Path) -> None:
    shutil.copytree(ROOT / "licenses", destination, dirs_exist_ok=True)
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", "x86_64-pc-windows-msvc"], cwd=ROOT))
    notices = []
    for package in metadata["packages"]:
        if package["name"] == "hd-screen-cast":
            continue
        source = Path(package["manifest_path"]).parent
        names = [item for item in source.iterdir() if item.is_file() and (item.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE")) or item.name.upper() == "UNLICENSE")]
        folder = destination / "rust" / f'{package["name"]}-{package["version"]}'
        folder.mkdir(parents=True, exist_ok=True)
        for item in names:
            shutil.copy2(item, folder / item.name)
        if package.get("license_file"):
            item = source / package["license_file"]
            if item.is_file():
                shutil.copy2(item, folder / item.name)
        notices.append({"name": package["name"], "version": package["version"], "license": package["license"], "repository": package["repository"]})
    (destination / "RUST-DEPENDENCIES.json").write_text(json.dumps(notices, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def archive_source(source: Path, output: Path, expected_tag: str) -> None:
    expected = git(source, "rev-list", "-n", "1", expected_tag)
    if git(source, "rev-parse", "HEAD") != expected:
        raise RuntimeError("Sunshine source does not match the pinned tag")
    modules = git(source, "submodule", "status", "--recursive")
    if any(line.startswith(("-", "+", "U")) for line in modules.splitlines()):
        raise RuntimeError("Sunshine submodules are incomplete or do not match pinned revisions")
    revision = source / "SOURCE-REVISION.txt"
    revision.write_text(f"Upstream: https://github.com/LizardByte/Sunshine\nTag: {expected_tag}\nCommit: {expected}\n\nRecursive submodules:\n{modules}\n", encoding="utf-8")
    with tarfile.open(output, "w:gz", compresslevel=6) as archive:
        for path in sorted(source.rglob("*")):
            if ".git" in path.relative_to(source).parts:
                continue
            if path.is_file() or path.is_symlink():
                archive.add(path, arcname=str(Path("Sunshine") / path.relative_to(source)), recursive=False)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sunshine-source", type=Path, default=ROOT / ".cache" / "sunshine-source")
    args = parser.parse_args()
    version = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
    executable = ROOT / "target" / "release" / "hd-screen-cast.exe"
    if not executable.is_file():
        raise RuntimeError("Build the release executable first")
    manifest = json.loads((ROOT / "packages-manifest.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        if sha256(ROOT / item["path"]) != item["sha256"]:
            raise RuntimeError(f'Package verification failed: {item["path"]}')
    if not (args.sunshine_source / ".git").exists():
        raise RuntimeError("Provide the pinned recursive Sunshine source using --sunshine-source, or prepare_packages.py --with-sources")
    dist = ROOT / "dist"
    dist.mkdir(exist_ok=True)
    staging = ROOT / ".cache" / f"release-{version}"
    staging.mkdir(parents=True, exist_ok=True)
    collect_licenses(staging / "licenses")
    source_archive = dist / f'Sunshine-{manifest["sunshine_version"]}-corresponding-source.tar.gz'
    archive_source(args.sunshine_source, source_archive, manifest["sunshine_version"])
    common = [(executable, "HD-Screen-Cast.exe")]
    for name in ("README.md", "README.zh-CN.md", "LICENSE", "THIRD_PARTY_NOTICES.md", "packages-manifest.json"):
        common.append((ROOT / name, name))
    common.extend((path, str(Path("licenses") / path.relative_to(staging / "licenses"))) for path in (staging / "licenses").rglob("*") if path.is_file())
    for edition in ("Controller", "Offline"):
        filename = f"HD-Screen-Cast-v{version}-windows-x64-{edition}.zip"
        with zipfile.ZipFile(dist / filename, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
            for source, relative in common:
                archive.write(source, relative)
            if edition == "Offline":
                for item in manifest["files"]:
                    archive.write(ROOT / item["path"], item["path"])
        print(filename, flush=True)
    files = [path for path in dist.iterdir() if path.suffix in {".zip", ".gz"}]
    (dist / "SHA256SUMS.txt").write_text("".join(f"{sha256(path)}  {path.name}\n" for path in sorted(files)), encoding="utf-8")
    print("Release artifacts:")
    for path in sorted(dist.iterdir()):
        print(path.name, path.stat().st_size)


if __name__ == "__main__":
    main()
