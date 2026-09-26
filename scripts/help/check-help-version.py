#!/usr/bin/env python3
"""Fail when the user manual has not been updated for the current version.

The manual (`apps/aethercodex-manager/public/help.html`) carries a version
meta tag and a visible version line. Both must match the version in
`crates/aethercodex-core/src/version.rs`, so a release cannot ship with a
manual that still describes the previous one.

    scripts/help/check-help-version.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
VERSION_RS = ROOT / "crates/aethercodex-core/src/version.rs"
HELP = ROOT / "apps/aethercodex-manager/public/help.html"


def main() -> int:
    version_match = re.search(r'pub const VERSION: &str = "([^"]+)"', VERSION_RS.read_text())
    if not version_match:
        print(f"error: could not read VERSION from {VERSION_RS}", file=sys.stderr)
        return 2
    version = version_match.group(1)

    if not HELP.exists():
        print(f"error: missing user manual at {HELP}", file=sys.stderr)
        return 2
    help_text = HELP.read_text()

    problems = []

    meta = re.search(r'<meta name="aethercodex-version" content="([^"]+)">', help_text)
    if not meta:
        problems.append('missing <meta name="aethercodex-version" content="...">')
    elif meta.group(1) != version:
        problems.append(
            f'meta version is "{meta.group(1)}" but the app is "{version}"'
        )

    if f"版本 {version}" not in help_text:
        problems.append(f'the manual does not show "版本 {version}" to the reader')

    if problems:
        print(f"The user manual is out of date for {version}:", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        print(
            "\nUpdate apps/aethercodex-manager/public/help.html: bump both version\n"
            "markers and describe whatever changed in this release.",
            file=sys.stderr,
        )
        return 1

    print(f"user manual is current for {version}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
