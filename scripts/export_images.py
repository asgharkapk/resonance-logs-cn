"""
Batch-export Texture2D/Sprite PNGs from Unity bundles via UnityPy.

Optional --manifest: main process writes JSONL after workers return ManifestRow tuples
(source bundle, Unity container/name/path_id, out_rel relative to task out_base, plus out_rel_output).
"""

import argparse
import json
from concurrent.futures import ProcessPoolExecutor, as_completed
from dataclasses import dataclass
import os
import re
import sys
from typing import Optional

import UnityPy

DEFAULT_EXTS = (".ab", ".unity3d", ".bundle", ".assets")
DEFAULT_TYPES = ("Texture2D", "Sprite")
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)


@dataclass(frozen=True)
class ManifestRow:
    """One exported PNG; written to JSONL by the main process only."""

    task_index: int
    source_rel: str
    asset_type: str
    container_path: str
    unity_name: str
    path_id: int
    out_rel: str


@dataclass(frozen=True)
class ExportTask:
    index: int
    src: str
    rel_src: str
    out_base: str
    asset_types: tuple[str, ...]
    flat: bool
    quiet: bool
    png_compress_level: Optional[int]


@dataclass(frozen=True)
class ExportResult:
    index: int
    rel_src: str
    exported: int = 0
    error: Optional[str] = None
    manifest_rows: tuple[ManifestRow, ...] = ()


def non_negative_int(value: str) -> int:
    try:
        parsed = int(value)
    except ValueError as e:
        raise argparse.ArgumentTypeError("must be an integer") from e
    if parsed < 0:
        raise argparse.ArgumentTypeError("must be >= 0")
    return parsed


def png_compress_level(value: str) -> int:
    try:
        parsed = int(value)
    except ValueError as e:
        raise argparse.ArgumentTypeError("must be an integer") from e
    if parsed < 0 or parsed > 9:
        raise argparse.ArgumentTypeError("must be between 0 and 9")
    return parsed


def resolve_worker_count(requested: int, task_count: int) -> int:
    if task_count <= 0:
        return 1
    if requested == 0:
        requested = max(1, (os.cpu_count() or 2) - 1)
    return max(1, min(requested, task_count))


def resolve_input_dir(path: str) -> str:
    """Resolve relative input paths from CWD, then project root if the CWD path is missing."""
    if os.path.isabs(path):
        return os.path.abspath(path)

    cwd_candidate = os.path.abspath(path)
    if os.path.isdir(cwd_candidate):
        return cwd_candidate

    root_candidate = os.path.abspath(os.path.join(PROJECT_ROOT, path))
    if os.path.isdir(root_candidate):
        return root_candidate

    return cwd_candidate


def safe_part(s: str) -> str:
    s = (s or "noname").strip()
    # Windows invalid chars + control chars
    s = re.sub(r'[<>:"/\\|?*\x00-\x1F]', "_", s)
    s = re.sub(r"\s+", " ", s)
    s = s.strip(" .")
    if not s:
        s = "noname"
    return s[:180]


def split_any_path(p: str) -> list[str]:
    # split on both / and \
    return [x for x in re.split(r"[\\/]+", p) if x]


def join_safe(base_dir: str, rel_like_path: str) -> str:
    parts = split_any_path(rel_like_path)
    safe_parts = [safe_part(x) for x in parts]
    return os.path.join(base_dir, *safe_parts)


def ensure_unique(path: str) -> str:
    root, ext = os.path.splitext(path)
    i = 0
    while True:
        candidate = path if i == 0 else f"{root}_{i}{ext}"
        flags = os.O_CREAT | os.O_EXCL | os.O_WRONLY
        if hasattr(os, "O_BINARY"):
            flags |= os.O_BINARY
        try:
            fd = os.open(candidate, flags)
        except FileExistsError:
            i += 1
        else:
            os.close(fd)
            return candidate


def get_unity_name(data, obj=None) -> str:
    """
    Best-effort Unity object name.
    Prefer m_Name (Unity serialized field), then name (UnityPy convenience),
    then obj.name. Always returns a non-empty safe string.
    """
    raw = None
    for attr in ("m_Name", "name", "Name"):
        raw = getattr(data, attr, None)
        if raw:
            break
    if not raw and obj is not None:
        raw = getattr(obj, "name", None)
    return safe_part(raw)


def best_rel_path_no_ext(container_path: Optional[str], unity_name: str) -> str:
    """
    Keep container folder structure when useful, but ensure the final filename
    comes from unity_name (m_Name) when container path is missing/unhelpful.
    Returns a relative path WITHOUT extension.
    """
    if not container_path:
        return unity_name

    is_dir_like = container_path.endswith(("/", "\\"))
    parts = split_any_path(container_path)
    if not parts:
        return unity_name

    if is_dir_like:
        parts.append(unity_name)
    else:
        last = parts[-1]
        last_no_ext = os.path.splitext(last)[0]
        # Some bundles use CAB hashes / ids / empty-ish names as the "path".
        bad_last = (
            (not last_no_ext)
            or last_no_ext.isdigit()
            or bool(re.match(r"^cab-[0-9a-f]{16,}$", last_no_ext, re.I))
        )
        if bad_last and unity_name != "noname":
            parts[-1] = unity_name
        else:
            parts[-1] = last_no_ext or unity_name

    return os.path.join(*parts)


def export_env_images(
    env: "UnityPy.Environment",
    out_base: str,
    asset_types: set[str],
    flat: bool,
    quiet: bool,
    png_compress_level: Optional[int],
    task_index: int,
    source_rel: str,
) -> tuple[int, list[ManifestRow]]:
    exported = 0
    rows: list[ManifestRow] = []
    made_dirs: set[str] = set()
    save_kwargs = {}
    if png_compress_level is not None:
        save_kwargs["compress_level"] = png_compress_level

    def export_one(
        image,
        rel_path_no_ext: str,
        *,
        asset_type: str,
        container_path: str,
        unity_name: str,
        path_id: int,
    ) -> None:
        nonlocal exported
        if flat:
            name = rel_path_no_ext.replace("\\", "_").replace("/", "_")
            out_path = os.path.join(out_base, safe_part(name) + ".png")
        else:
            out_path = join_safe(out_base, rel_path_no_ext + ".png")
        out_dir = os.path.dirname(out_path)
        if out_dir not in made_dirs:
            os.makedirs(out_dir, exist_ok=True)
            made_dirs.add(out_dir)
        out_path = ensure_unique(out_path)
        try:
            image.save(out_path, **save_kwargs)
        except Exception:
            try:
                os.remove(out_path)
            except OSError:
                pass
            raise
        exported += 1
        out_rel = os.path.relpath(out_path, out_base)
        rows.append(
            ManifestRow(
                task_index=task_index,
                source_rel=source_rel,
                asset_type=asset_type,
                container_path=container_path,
                unity_name=unity_name,
                path_id=path_id,
                out_rel=out_rel,
            )
        )
        if not quiet:
            print("  +", out_rel)

    # Prefer container paths (more meaningful, often includes folders)
    if getattr(env, "container", None):
        for container_path, obj in env.container.items():
            if obj.type.name not in asset_types:
                continue
            data = obj.read()
            img = getattr(data, "image", None)
            if img is None:
                continue
            unity_name = get_unity_name(data, obj=obj)
            rel = best_rel_path_no_ext(container_path, unity_name)
            export_one(
                img,
                rel,
                asset_type=obj.type.name,
                container_path=str(container_path),
                unity_name=unity_name,
                path_id=int(obj.path_id),
            )

    # Fallback: scan all objects (names may be less structured)
    if exported == 0:
        for obj in env.objects:
            if obj.type.name not in asset_types:
                continue
            data = obj.read()
            img = getattr(data, "image", None)
            if img is None:
                continue
            name = get_unity_name(data, obj=obj)
            rel = os.path.join("__no_container", obj.type.name, f"{name}_{obj.path_id}")
            export_one(
                img,
                rel,
                asset_type=obj.type.name,
                container_path="",
                unity_name=name,
                path_id=int(obj.path_id),
            )

    return exported, rows


def process_source_file(task: ExportTask) -> ExportResult:
    try:
        env = UnityPy.load(task.src)
        exported, rows = export_env_images(
            env=env,
            out_base=task.out_base,
            asset_types=set(task.asset_types),
            flat=task.flat,
            quiet=task.quiet,
            png_compress_level=task.png_compress_level,
            task_index=task.index,
            source_rel=task.rel_src,
        )
        return ExportResult(
            index=task.index,
            rel_src=task.rel_src,
            exported=exported,
            manifest_rows=tuple(rows),
        )
    except Exception as e:
        return ExportResult(index=task.index, rel_src=task.rel_src, error=str(e))


def write_manifest_jsonl(
    path: str,
    rows: list[ManifestRow],
    tasks_by_index: dict[int, ExportTask],
    out_dir: str,
) -> None:
    parent = os.path.dirname(os.path.abspath(path))
    if parent:
        os.makedirs(parent, exist_ok=True)
    sorted_rows = sorted(rows, key=lambda r: (r.task_index, r.source_rel, r.path_id, r.out_rel))
    with open(path, "w", encoding="utf-8") as f:
        for row in sorted_rows:
            task = tasks_by_index[row.task_index]
            out_rel_output = os.path.normpath(
                os.path.join(os.path.relpath(task.out_base, out_dir), row.out_rel)
            )
            rec = {
                "task_index": row.task_index,
                "source_rel": row.source_rel,
                "asset_type": row.asset_type,
                "container_path": row.container_path,
                "unity_name": row.unity_name,
                "path_id": row.path_id,
                "out_rel": row.out_rel,
                "out_rel_output": out_rel_output,
            }
            f.write(json.dumps(rec, ensure_ascii=False) + "\n")


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(
        description="Batch export Unity images (Texture2D/Sprite) from .ab/.unity3d using UnityPy"
    )
    p.add_argument(
        "-i",
        "--input",
        required=True,
        help="Input directory containing .ab/.unity3d/.bundle/.assets files",
    )
    p.add_argument(
        "-o",
        "--output",
        default="exported_images",
        help="Output directory (default: exported_images)",
    )
    p.add_argument(
        "--ext",
        default=",".join(DEFAULT_EXTS),
        help='Comma-separated extensions to include (default: ".ab,.unity3d,.bundle,.assets")',
    )
    p.add_argument(
        "--types",
        default=",".join(DEFAULT_TYPES),
        help='Comma-separated asset types to export (default: "Texture2D,Sprite")',
    )
    p.add_argument(
        "--flat",
        action="store_true",
        help="Flatten output into one folder (no subdirectories)",
    )
    p.add_argument(
        "--by-source",
        action="store_true",
        help="Create a subfolder per source file under output/",
    )
    p.add_argument(
        "--quiet",
        action="store_true",
        help="Less verbose output",
    )
    p.add_argument(
        "--workers",
        type=non_negative_int,
        default=0,
        help="Worker processes to use; 0 auto-selects, 1 runs serially (default: 0)",
    )
    p.add_argument(
        "--png-compress-level",
        type=png_compress_level,
        default=None,
        help="PNG compression level 0-9; lower is faster but larger (default: Pillow default)",
    )
    p.add_argument(
        "--manifest",
        nargs="?",
        const="",
        default=None,
        metavar="PATH",
        help="Write JSONL manifest after export; PATH defaults to OUTPUT/export_manifest.jsonl when flag is used without a path",
    )

    args = p.parse_args(argv)

    in_dir = resolve_input_dir(args.input)
    out_dir = os.path.abspath(args.output)
    exts = tuple(x.strip().lower() for x in args.ext.split(",") if x.strip())
    asset_types = {x.strip() for x in args.types.split(",") if x.strip()}

    if not os.path.isdir(in_dir):
        print(f"Input directory not found: {in_dir}", file=sys.stderr)
        return 2

    os.makedirs(out_dir, exist_ok=True)

    source_items: list[tuple[int, str, str, str]] = []

    for root, _, files in os.walk(in_dir):
        for fn in files:
            if not fn.lower().endswith(exts):
                continue
            index = len(source_items) + 1
            src = os.path.join(root, fn)
            rel = os.path.relpath(src, in_dir)

            # Choose output base for this source file
            if args.by_source:
                rel_no_ext = os.path.splitext(rel)[0]
                out_base = join_safe(out_dir, rel_no_ext)
            else:
                out_base = out_dir

            source_items.append((index, src, rel, out_base))

    total_files = len(source_items)
    workers = resolve_worker_count(args.workers, total_files)
    if workers > 1 and not args.by_source:
        print(
            "Warning: --by-source is recommended with --workers > 1 to keep outputs easier to compare.",
            file=sys.stderr,
        )
    if not args.quiet:
        print(f"Using workers={workers} for scanned_files={total_files}")

    task_quiet = args.quiet or workers > 1
    tasks = [
        ExportTask(
            index=index,
            src=src,
            rel_src=rel,
            out_base=out_base,
            asset_types=tuple(asset_types),
            flat=args.flat,
            quiet=task_quiet,
            png_compress_level=args.png_compress_level,
        )
        for index, src, rel, out_base in source_items
    ]

    total_exported = 0
    failed = 0
    all_manifest_rows: list[ManifestRow] = []

    def handle_result(result: ExportResult) -> None:
        nonlocal total_exported, failed
        if result.error:
            failed += 1
            print(f"[{result.index}/{total_files}] {result.rel_src}", file=sys.stderr)
            print(f"  !! FAIL: {result.error}", file=sys.stderr)
            return
        total_exported += result.exported
        all_manifest_rows.extend(result.manifest_rows)
        if not args.quiet:
            print(f"[{result.index}/{total_files}] {result.rel_src}")
            print(f"  => exported: {result.exported}")

    if workers == 1:
        for task in tasks:
            result = process_source_file(task)
            handle_result(result)
    else:
        with ProcessPoolExecutor(max_workers=workers) as executor:
            future_to_task = {
                executor.submit(process_source_file, task): task for task in tasks
            }
            for future in as_completed(future_to_task):
                task = future_to_task[future]
                try:
                    result = future.result()
                except Exception as e:
                    result = ExportResult(
                        index=task.index,
                        rel_src=task.rel_src,
                        error=str(e),
                    )
                handle_result(result)

    manifest_path: Optional[str] = None
    if args.manifest is not None:
        manifest_path = (
            os.path.join(out_dir, "export_manifest.jsonl")
            if args.manifest == ""
            else os.path.abspath(args.manifest)
        )

    if manifest_path is not None:
        if failed > 0:
            print(
                "Warning: manifest lists only successfully exported images; "
                "some source files failed.",
                file=sys.stderr,
            )
        tasks_by_index = {t.index: t for t in tasks}
        write_manifest_jsonl(manifest_path, all_manifest_rows, tasks_by_index, out_dir)
        if not args.quiet:
            print(f"Manifest: {manifest_path} ({len(all_manifest_rows)} rows)")

    print("")
    print(f"Done. scanned_files={total_files}, exported_images={total_exported}, failed_files={failed}")
    print(f"Output: {out_dir}")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

