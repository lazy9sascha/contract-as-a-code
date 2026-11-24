#!/usr/bin/env python3
"""Run one CAAC intake and evidence round on public GitHub Trending repositories."""

import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import fcntl
from hashlib import sha1, sha256
from html.parser import HTMLParser
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
from tempfile import TemporaryDirectory
from urllib.parse import quote
from urllib.request import Request, urlopen


PROJECT = Path(__file__).resolve().parents[1]
COHORT_SIZE = 20
MAX_BLOB = 128 * 1024
REPO_NAME = re.compile(r"[A-Za-z0-9-]+/[A-Za-z0-9_.-]+")
GIT_SHA = re.compile(r"[0-9a-f]{40}")
SOURCE_TYPES = {".rs", ".py"}
OTHER_SOURCE_TYPES = {".c", ".cpp", ".go", ".java", ".js", ".kt", ".rb", ".sh", ".swift", ".ts", ".tsx"}
EXCLUDED = {".github", "node_modules", "target", "vendor", "fixtures", "tests", "test"}


class Rankings(HTMLParser):
    def __init__(self):
        super().__init__()
        self.row = False
        self.heading = False
        self.repos = []

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "article":
            self.row = "Box-row" in attrs.get("class", "").split()
        elif tag == "h2" and self.row:
            self.heading = True
        elif tag == "a" and self.heading:
            href = attrs.get("href", "")
            if href.startswith("/") and REPO_NAME.fullmatch(href[1:]):
                self.repos.append(href[1:])
                self.heading = False

    def handle_endtag(self, tag):
        if tag == "article":
            self.row = False
            self.heading = False
        elif tag == "h2":
            self.heading = False


def download(url, limit):
    request = Request(url, headers={"User-Agent": "caac-trending-smoke"})
    with urlopen(request, timeout=20) as response:
        payload = response.read(limit + 1)
    if len(payload) > limit:
        raise ValueError("Response exceeded its byte limit")
    return payload


def cohort():
    selected = {}
    ranking_counts = {}
    for period in ("daily", "weekly", "monthly"):
        rankings = Rankings()
        rankings.feed(download(f"https://github.com/trending?since={period}", 2 * 1024 * 1024).decode())
        if not rankings.repos:
            raise ValueError(f"GitHub {period} Trending contained no recognized ranking rows")
        ranking_counts[period] = len(rankings.repos)
        for rank, repo in enumerate(rankings.repos, start=1):
            selected.setdefault(repo.casefold(), {"repo": repo, "period": period, "rank": rank})
            if len(selected) == COHORT_SIZE:
                return list(selected.values()), ranking_counts
    return list(selected.values()), ranking_counts


def checked_tree(repo):
    remote = subprocess.run(
        ["/usr/bin/git", "ls-remote", f"https://github.com/{repo}.git", "HEAD"],
        env={
            "PATH": "/usr/bin:/bin",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_TERMINAL_PROMPT": "0",
        },
        cwd="/",
        capture_output=True,
        text=True,
        timeout=30,
        check=True,
    )
    fields = remote.stdout.split()
    if len(fields) != 2 or not GIT_SHA.fullmatch(fields[0]) or fields[1] != "HEAD":
        raise ValueError("GitHub did not return a pinned HEAD commit")
    commit = fields[0]
    url = f"https://api.github.com/repos/{repo}/git/trees/{commit}?recursive=1"
    return commit, json.loads(download(url, 12 * 1024 * 1024))


def select_files(tree):
    sources, other_sources, documents = [], [], []
    for entry in tree:
        value = entry["path"]
        path = PurePosixPath(value)
        if (
            entry.get("mode") not in ("100644", "100755")
            or not 0 < entry.get("size", 0) <= MAX_BLOB
            or path.as_posix() != value
            or path.is_absolute()
            or any(part.startswith(".") or part in EXCLUDED for part in path.parts)
            or any(ord(character) < 32 for character in value)
        ):
            continue
        if path.suffix in SOURCE_TYPES:
            sources.append(entry)
        elif path.suffix in OTHER_SOURCE_TYPES:
            other_sources.append(entry)
        elif len(path.parts) == 1 and path.name.casefold() in ("readme.md", "contract.md"):
            documents.append(entry)

    def order(entry):
        return (entry["path"].count("/"), entry["path"])

    return sorted(sources, key=order)[:8] or sorted(other_sources, key=order)[:1], sorted(documents, key=order)[:2]


def retain_blob(repo, commit, entry, snapshot):
    relative = entry["path"]
    url = f"https://raw.githubusercontent.com/{repo}/{commit}/{quote(relative, safe='/')}"
    data = download(url, MAX_BLOB)
    git_bytes = f"blob {len(data)}\0".encode() + data
    if len(data) != entry["size"] or sha1(git_bytes).hexdigest() != entry["sha"]:
        raise ValueError(f"Blob does not match the pinned tree: {relative}")
    path = snapshot / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as file:
        file.write(data)
    return path, {"path": relative, "sha256": sha256(data).hexdigest()}


def run_step(binary, row, stage, *arguments):
    result = subprocess.run(
        [str(binary), *map(str, arguments)],
        cwd="/",
        capture_output=True,
        timeout=45,
    )
    row["stages"][stage] = result.returncode
    if result.returncode:
        raise ValueError(f"{stage} exited {result.returncode}: {result.stderr.decode(errors='replace')[:1000]}")
    return result.stdout


def test_repository(ranking, binary, scratch):
    repo = ranking["repo"]
    row = {**ranking, "stages": {}, "sources": [], "documents": []}
    try:
        commit, tree = checked_tree(repo)
        row.update(commit=commit, tree_truncated=tree["truncated"])
        source_entries, document_entries = select_files(tree["tree"])
        trial = scratch / f"{ranking['period']}-{ranking['rank']}"
        snapshot = trial / "snapshot"
        snapshot.mkdir(parents=True)
        evidence = []
        source_arguments = []
        for entry in source_entries:
            path, identity = retain_blob(repo, commit, entry, snapshot)
            source_arguments.extend(("--source", path))
            row["sources"].append(identity)
        for entry in document_entries:
            path, identity = retain_blob(repo, commit, entry, snapshot)
            evidence.extend(("--contract", path))
            row["documents"].append(identity)

        run_step(binary, row, "discover", "discover", "--directory", snapshot, "--depth", 4)
        if source_arguments:
            intake = trial / "intake"
            baseline = intake / "baseline.json"
            run_step(binary, row, "plugin", "plugin", intake, "--title", repo, *source_arguments)
            run_step(binary, row, "baseline-check", "baseline", "check", baseline)
            baseline_files = json.loads(baseline.read_bytes())["files"]
            row["syntax_statuses"] = dict(Counter(file["analysis"]["status"] for file in baseline_files))
            evidence.extend(("--baseline", baseline))
        if not evidence:
            row["result"] = "no-selected-input"
            return row

        collection = trial / "collection.json"
        packet = trial / "packet.json"
        run_step(binary, row, "collect", "collect", *evidence, "--output", collection)
        run_step(binary, row, "review-packet", "review-packet", "--collection", collection, *evidence, "--output", packet)
        row["collection_sha256"] = sha256(collection.read_bytes()).hexdigest()
        row["packet_sha256"] = sha256(packet.read_bytes()).hexdigest()
        row["result"] = "passed"
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        row.update(result="failed", error=str(error)[:1500])
    return row


def record(path, report):
    temporary = path.with_suffix(".new")
    temporary.write_text(json.dumps(report, ensure_ascii=True, indent=2) + "\n", encoding="utf-8")
    temporary.replace(path)


def run_round(state, cargo):
    report = {
        "started_at": datetime.now(timezone.utc).isoformat(),
        "requested_repositories": COHORT_SIZE,
        "scope": "selected source and document intake, baseline verification, collection, review packet",
        "repositories": [],
    }
    try:
        ranking, counts = cohort()
        report["ranking_counts"] = counts
        report["cohort"] = ranking
        record(state / "progress.json", report)
        build = subprocess.run(
            [cargo, "build", "--offline", "--locked", "--bin", "caac", "--target-dir", str(PROJECT / "target")],
            cwd=PROJECT,
            capture_output=True,
            text=True,
            timeout=180,
        )
        if build.returncode:
            raise ValueError(f"CAAC build exited {build.returncode}: {build.stderr[-2000:]}")
        with TemporaryDirectory(prefix="round-", dir=state) as directory:
            scratch = Path(directory)
            binary = scratch / "caac"
            shutil.copy2(PROJECT / "target" / "debug" / "caac", binary)
            report["caac_sha256"] = sha256(binary.read_bytes()).hexdigest()
            with ThreadPoolExecutor(max_workers=4) as pool:
                tasks = [pool.submit(test_repository, item, binary, scratch) for item in ranking]
                for task in tasks:
                    row = task.result()
                    report["repositories"].append(row)
                    record(state / "progress.json", report)
        report["results"] = dict(Counter(row["result"] for row in report["repositories"]))
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        report["error"] = str(error)[:2000]
    report["finished_at"] = datetime.now(timezone.utc).isoformat()
    record(state / "latest.json", report)
    if not (state / "first.json").exists():
        record(state / "first.json", report)
    record(state / "progress.json", report)
    print(json.dumps({key: report[key] for key in ("finished_at", "results", "error") if key in report}))
    return int(report.get("results") != {"passed": COHORT_SIZE})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state-dir", type=Path, default=PROJECT / "target" / "trending-smoke")
    parser.add_argument("--cargo", default="cargo")
    args = parser.parse_args()
    state = args.state_dir.resolve()
    state.mkdir(parents=True, exist_ok=True)
    with (state / "round.lock").open("a") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print("A CAAC Trending round is already running.")
            return 0
        return run_round(state, args.cargo)


if __name__ == "__main__":
    raise SystemExit(main())
