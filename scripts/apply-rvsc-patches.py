#!/usr/bin/env python3
"""Apply RvsC dependency patches to Cargo.toml.

Usage:
    python3 scripts/apply-rvsc-patches.py [remote|local|clean]
    PATCH_MODE=local python3 scripts/apply-rvsc-patches.py

Markers expected in Cargo.toml:
    # ==RVSC_CRATES_IO_START==   / # ==RVSC_CRATES_IO_END==

The script replaces content between each pair of markers with the
corresponding entries from rvsc-patches-{mode}.toml.
Running 'clean' empties all markers without writing any entries.
"""

import sys
import re
import os

# Maps marker names (found in Cargo.toml) -> TOML section headers (found in patch files)
MARKER_TO_SECTION = {
    "CRATES_IO": "[patch.crates-io]",
}


def parse_patch_file(path: str) -> dict:
    """Parse a rvsc-patches-*.toml file into {section_header: [lines]}."""
    sections: dict = {}
    current = None
    with open(path) as f:
        for raw_line in f:
            line = raw_line.rstrip()
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            if stripped.startswith("[patch."):
                current = stripped
                sections[current] = []
            elif current is not None:
                sections[current].append(line)
    return sections


def find_markers(content: str) -> list:
    """Return all marker names present in the Cargo.toml content."""
    return re.findall(r"# ==RVSC_(\w+)_START==", content)


def apply_marker(content: str, marker_name: str, replacement_lines: list) -> str:
    """Replace the block between START/END markers with replacement_lines."""
    start = f"# ==RVSC_{marker_name}_START=="
    end = f"# ==RVSC_{marker_name}_END=="
    inner = ("\n".join(replacement_lines) + "\n") if replacement_lines else ""
    pattern = re.compile(
        rf"({re.escape(start)})\n(.*?)({re.escape(end)})",
        re.DOTALL,
    )
    new_content, n = pattern.subn(f"\\1\n{inner}\\3", content)
    if n == 0:
        print(f"WARNING: Markers not found — {start}", file=sys.stderr)
    return new_content


def main() -> None:
    mode = (
        sys.argv[1]
        if len(sys.argv) > 1
        else os.environ.get("PATCH_MODE", "remote")
    )
    if mode not in ("remote", "local", "clean"):
        print(f"Usage: {sys.argv[0]} [remote|local|clean]", file=sys.stderr)
        sys.exit(1)

    cargo_file = "Cargo.toml"
    with open(cargo_file) as f:
        content = f.read()

    markers = find_markers(content)
    if not markers:
        print("WARNING: No RVSC markers found in Cargo.toml", file=sys.stderr)

    if mode == "clean":
        for m in markers:
            content = apply_marker(content, m, [])
    else:
        patch_file = f"rvsc-patches-{mode}.toml"
        if not os.path.exists(patch_file):
            print(f"ERROR: Patch file not found: {patch_file}", file=sys.stderr)
            sys.exit(1)
        sections = parse_patch_file(patch_file)
        for m in markers:
            section_header = MARKER_TO_SECTION.get(m)
            lines = sections.get(section_header, []) if section_header else []
            content = apply_marker(content, m, lines)

    with open(cargo_file, "w") as f:
        f.write(content)

    print(f"✓ Applied '{mode}' patches to {cargo_file}")


if __name__ == "__main__":
    main()
