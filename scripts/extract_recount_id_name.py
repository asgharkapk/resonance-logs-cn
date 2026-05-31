# -*- coding: utf-8 -*-
"""Extract localized RecountTable files into scripts/output.

The Chinese ZTable/RecountTable.json is the template source for all output keys
and full row structure. The "其他" catch-all row is skipped. English/Japanese
names are supplied by override files and only replace RecountName.
"""

import argparse
import json
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
OUTPUT_DIR = SCRIPT_DIR / "output"
ZH_RECOUNT_PATH = PROJECT_ROOT / "ZTable" / "RecountTable.json"

LANG_CONFIGS = {
    "zh": {
        "overrides_dir": SCRIPT_DIR / "overrides",
        "output_name": "RecountTable.json",
    },
    "en": {
        "overrides_dir": SCRIPT_DIR / "overrides_en",
        "output_name": "RecountTable_en.json",
    },
    "jp": {
        "overrides_dir": SCRIPT_DIR / "overrides_jp",
        "output_name": "RecountTable_jp.json",
    },
}

OVERRIDE_FILE_NAME = "recount_name_overrides.json"
SKIP_RECOUNT_NAMES = {"其他"}


def _load_json(path: Path, missing_ok: bool = False) -> dict:
    if not path.exists():
        if missing_ok:
            return {}
        raise FileNotFoundError(path)
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _write_json(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)


def _source_recount_table() -> dict:
    table = _load_json(ZH_RECOUNT_PATH)
    result = {}

    for key, entry in table.items():
        if not isinstance(entry, dict):
            continue

        source_key = str(entry.get("Id", key))
        recount_name = entry.get("RecountName") or ""
        if recount_name in SKIP_RECOUNT_NAMES:
            continue

        result[source_key] = dict(entry)

    return result


def _load_overrides(lang: str) -> dict:
    if lang == "zh":
        return {}
    return _load_json(
        LANG_CONFIGS[lang]["overrides_dir"] / OVERRIDE_FILE_NAME,
        missing_ok=True,
    )


def _build_recount_table(lang: str, source_table: dict) -> tuple[dict, int]:
    overrides = _load_overrides(lang)
    result = {key: dict(entry) for key, entry in source_table.items()}

    applied_count = 0
    for recount_id, name in overrides.items():
        entry = result.get(str(recount_id))
        if isinstance(entry, dict):
            entry["RecountName"] = name
            applied_count += 1

    return result, applied_count


def _write_recount_table(lang: str, result: dict, override_count: int) -> None:
    output_path = OUTPUT_DIR / LANG_CONFIGS[lang]["output_name"]
    _write_json(output_path, result)
    print(f"已生成: {output_path}，共 {len(result)} 条（{override_count} 条由例外文件覆盖）")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="从中文 RecountTable 模板和 override 文件提取多语言 RecountTable。"
    )
    parser.add_argument(
        "--lang",
        choices=sorted(LANG_CONFIGS.keys()),
        help="只生成指定语言；默认生成所有语言。",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    langs = [args.lang] if args.lang else list(LANG_CONFIGS.keys())

    try:
        source_table = _source_recount_table()
        for lang in langs:
            result, override_count = _build_recount_table(lang, source_table)
            _write_recount_table(lang, result, override_count)
    except FileNotFoundError as exc:
        print(f"错误: 未找到 {exc}")
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
