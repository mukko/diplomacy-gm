#!/usr/bin/env python3
"""Collect the licenses of the third-party code built into the program.

Rust crates linked into the server for TARGET (normal dependencies only) and the JavaScript
packages bundled into the GUI (non-dev entries of web/package-lock.json), with each package's
license files. Used by scripts/package.sh.

Usage: python3 -I scripts/third_party_licenses.py TARGET OUTPUT
"""

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LICENSE_PREFIXES = ("license", "licence", "copying", "notice", "unlicense", "copyright")


def license_files(directory: Path) -> list[Path]:
    if not directory.is_dir():
        return []
    return sorted(
        f for f in directory.iterdir()
        if f.is_file() and f.name.lower().startswith(LICENSE_PREFIXES)
    )


def rust_packages(target: str) -> list[dict]:
    meta = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--locked", "--filter-platform", target,
         "--manifest-path", str(ROOT / "server" / "Cargo.toml")],
    ))
    packages = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    root = meta["resolve"]["root"]
    seen, todo = set(), [root]
    while todo:
        pid = todo.pop()
        for dep in nodes[pid]["deps"]:
            normal = any(k["kind"] is None for k in dep["dep_kinds"])
            if normal and dep["pkg"] not in seen:
                seen.add(dep["pkg"])
                todo.append(dep["pkg"])
    out = []
    for pid in seen:
        p = packages[pid]
        out.append({
            "name": p["name"],
            "version": p["version"],
            "license": p.get("license") or "(see files)",
            "dir": Path(p["manifest_path"]).parent,
        })
    return sorted(out, key=lambda p: (p["name"], p["version"]))


def js_packages() -> list[dict]:
    lock = json.loads((ROOT / "web" / "package-lock.json").read_text())
    out = []
    for path, info in lock["packages"].items():
        if not path or info.get("dev") or info.get("devOptional"):
            continue
        out.append({
            "name": path.split("node_modules/")[-1],
            "version": info.get("version", "?"),
            "license": info.get("license", "(see files)"),
            "dir": ROOT / "web" / path,
        })
    return sorted(out, key=lambda p: p["name"])


def section(title: str, packages: list[dict]) -> str:
    parts = [f"{'=' * 78}\n{title}\n{'=' * 78}\n"]
    for p in packages:
        parts.append(f"\n{'-' * 78}\n{p['name']} {p['version']} — {p['license']}\n{'-' * 78}\n")
        files = license_files(p["dir"])
        if not files:
            parts.append(f"(No license file in the package; licensed under {p['license']}.)\n")
        for f in files:
            parts.append(f"\n[{f.name}]\n\n{f.read_text(errors='replace').strip()}\n")
    return "".join(parts)


def main() -> int:
    target, output = sys.argv[1], Path(sys.argv[2])
    rust, js = rust_packages(target), js_packages()
    missing_js = [p["name"] for p in js if not p["dir"].is_dir()]
    if missing_js:
        sys.exit(f"web dependencies not installed (run npm ci in web/): {', '.join(missing_js)}")
    text = (
        "Third-party software built into diplomacy-gm\n"
        "diplomacy-gm itself is licensed under the MIT License (see LICENSE); the board map under\n"
        "CC BY-SA 3.0 (see CREDITS.md). The components below keep their own licenses.\n\n"
        + section(f"Rust crates ({len(rust)}) — target {target}", rust)
        + "\n\n"
        + section(f"JavaScript packages ({len(js)}) — bundled into the GUI", js)
    )
    output.write_text(text)
    print(f"{output}: {len(rust)} Rust crates, {len(js)} JavaScript packages")
    return 0


if __name__ == "__main__":
    sys.exit(main())
