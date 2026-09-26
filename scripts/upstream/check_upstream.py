#!/usr/bin/env python3
"""Report what upstream Codex++ has changed since we last reviewed it.

AetherCodex is a renamed, trimmed fork, so `git merge upstream/main` is not
usable: every path was renamed and some subsystems were deliberately dropped.
This reports instead, classifying each upstream commit by whether it touches
code we still carry.

    scripts/upstream/check_upstream.py               # markdown report
    scripts/upstream/check_upstream.py --json        # machine readable
    scripts/upstream/check_upstream.py --no-fetch    # use the local remote ref
    scripts/upstream/check_upstream.py --set-baseline <sha>

Exit status: 0 when there is nothing new, 1 when upstream has commits worth
reviewing, 2 on error. CI uses that to decide whether to raise a report.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / ".upstream-sync.json"
REMOTE = "upstream"


def run(*args: str, check: bool = True) -> str:
    result = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    )
    if check and result.returncode != 0:
        raise SystemExit(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout.strip()


def load_config() -> dict:
    if not CONFIG.exists():
        raise SystemExit(f"missing {CONFIG.relative_to(ROOT)}")
    return json.loads(CONFIG.read_text())


def ensure_remote(config: dict, fetch: bool) -> None:
    remotes = run("remote").splitlines()
    if REMOTE not in remotes:
        run("remote", "add", REMOTE, config["upstream_remote"])
    if fetch:
        run("fetch", "--no-tags", "--quiet", REMOTE)


def map_path(path: str, path_map: dict[str, str]) -> str | None:
    """Translate an upstream path to ours, or None when we do not carry it."""
    for upstream_prefix, ours in sorted(path_map.items(), key=lambda kv: -len(kv[0])):
        if path == upstream_prefix or path.startswith(upstream_prefix):
            mapped = ours + path[len(upstream_prefix) :]
            return mapped
    # Paths outside the map are carried unchanged when they exist here.
    return path


def classify(paths: list[str], config: dict) -> tuple[str, list[str]]:
    """Return (verdict, our paths affected).

    portable  - touches files we still have
    new       - only touches files we never had (an upstream feature to decide on)
    declined  - only touches things we removed on purpose
    """
    declined_prefixes = tuple(config.get("declined_paths", []))
    path_map = config.get("path_map", {})

    ours: list[str] = []
    saw_declined = False
    for path in paths:
        if path.startswith(declined_prefixes):
            saw_declined = True
            continue
        mapped = map_path(path, path_map)
        if mapped and (ROOT / mapped).exists():
            ours.append(mapped)

    if ours:
        return "portable", sorted(set(ours))
    if saw_declined:
        return "declined", []
    return "new", []


def collect(config: dict) -> dict:
    baseline = config["baseline"]
    branch = config.get("upstream_branch", "main")
    ref = f"{REMOTE}/{branch}"

    head = run("rev-parse", ref)
    if run("rev-parse", baseline) == head:
        return {"baseline": baseline, "head": head, "commits": []}

    log = run(
        "log", "--reverse", "--no-merges", "--format=%H%x1f%an%x1f%ad%x1f%s",
        "--date=short", f"{baseline}..{ref}",
    )
    commits = []
    for line in log.splitlines():
        if not line.strip():
            continue
        sha, author, date, subject = line.split("\x1f", 3)
        files = run("show", "--name-only", "--format=", sha).splitlines()
        files = [f for f in files if f.strip()]
        verdict, ours = classify(files, config)
        commits.append({
            "sha": sha,
            "short": sha[:9],
            "author": author,
            "date": date,
            "subject": subject,
            "verdict": verdict,
            "files": files,
            "our_files": ours,
        })
    return {"baseline": baseline, "head": head, "commits": commits}


def area_of(path: str) -> str:
    parts = path.split("/")
    return "/".join(parts[:2]) if len(parts) > 1 else path


def render(report: dict, config: dict) -> str:
    commits = report["commits"]
    if not commits:
        return (
            f"# Upstream sync\n\nUp to date with upstream "
            f"`{report['head'][:9]}`. Nothing new to review.\n"
        )

    portable = [c for c in commits if c["verdict"] == "portable"]
    new = [c for c in commits if c["verdict"] == "new"]
    declined = [c for c in commits if c["verdict"] == "declined"]

    areas: dict[str, int] = {}
    for commit in portable:
        for path in commit["our_files"]:
            areas[area_of(path)] = areas.get(area_of(path), 0) + 1

    lines = [
        "# Upstream sync",
        "",
        f"Reviewed up to `{report['baseline'][:9]}`, upstream is at "
        f"`{report['head'][:9]}`.",
        "",
        f"- **{len(portable)}** commits touch code AetherCodex still carries",
        f"- **{len(new)}** commits are in subsystems this fork does not have",
        f"- **{len(declined)}** commits only touch parts removed on purpose "
        "(ads, donations)",
        "",
    ]

    if areas:
        lines += ["## Where the portable changes land", ""]
        for area, count in sorted(areas.items(), key=lambda kv: -kv[1]):
            lines.append(f"- `{area}` — {count} file touches")
        lines.append("")

    if portable:
        lines += ["## Commits to review", ""]
        for commit in portable[-80:]:
            lines.append(
                f"- `{commit['short']}` {commit['date']} {commit['subject']}"
            )
        if len(portable) > 80:
            lines.append(f"- …and {len(portable) - 80} earlier commits")
        lines.append("")

    if new:
        subsystems: dict[str, int] = {}
        for commit in new:
            for path in commit["files"]:
                subsystems[area_of(path)] = subsystems.get(area_of(path), 0) + 1
        lines += [
            "## Upstream-only subsystems",
            "",
            "These are features this fork never had. Adopting one is a product "
            "decision, not a sync.",
            "",
        ]
        for area, count in sorted(subsystems.items(), key=lambda kv: -kv[1])[:15]:
            lines.append(f"- `{area}` — {count} file touches")
        lines.append("")

    lines += [
        "## After porting",
        "",
        "Record how far the review got so the next run starts from there:",
        "",
        "```sh",
        f"scripts/upstream/check_upstream.py --set-baseline {report['head'][:9]}",
        "```",
        "",
    ]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON")
    parser.add_argument("--no-fetch", action="store_true", help="skip git fetch")
    parser.add_argument("--set-baseline", metavar="SHA", help="record a new baseline")
    args = parser.parse_args()

    config = load_config()

    if args.set_baseline:
        ensure_remote(config, fetch=not args.no_fetch)
        resolved = run("rev-parse", args.set_baseline)
        config["baseline"] = resolved
        config["baseline_note"] = (
            "Reviewed up to this upstream commit; earlier changes are ported or "
            "consciously declined."
        )
        CONFIG.write_text(json.dumps(config, indent=2, ensure_ascii=False) + "\n")
        print(f"baseline set to {resolved}")
        return 0

    ensure_remote(config, fetch=not args.no_fetch)
    report = collect(config)

    if args.json:
        print(json.dumps(report, indent=2, ensure_ascii=False))
    else:
        print(render(report, config))

    actionable = [c for c in report["commits"] if c["verdict"] != "declined"]
    return 1 if actionable else 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except SystemExit:
        raise
    except Exception as error:  # noqa: BLE001 - surface any failure to CI
        print(f"error: {error}", file=sys.stderr)
        sys.exit(2)
