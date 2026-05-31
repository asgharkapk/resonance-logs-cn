# -*- coding: utf-8 -*-
"""Extract scene Id -> Name mappings into scripts/output.

All languages share the Chinese ZTable as the template source so every output
keeps the same keys. Language-specific override folders supply translated names
and other corrections such as dungeon difficulty.
"""

import argparse
import json
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
OUTPUT_DIR = SCRIPT_DIR / "output"
ZTABLE_DIR = PROJECT_ROOT / "ZTable"

LANG_CONFIGS = {
    "zh": {
        "overrides_dir": SCRIPT_DIR / "overrides",
        "output_name": "SceneName.json",
    },
    "en": {
        "overrides_dir": SCRIPT_DIR / "overrides_en",
        "output_name": "SceneName_en.json",
    },
    "jp": {
        "overrides_dir": SCRIPT_DIR / "overrides_jp",
        "output_name": "SceneName_jp.json",
    },
}

OVERRIDE_FILE_NAME = "scene_name_overrides.json"


def _load_json(path: Path) -> dict:
    if not path.exists():
        return {}
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _scene_name(scene_id: str, scene_table: dict, map_info_table: dict) -> str:
    entry = scene_table.get(scene_id)
    name = entry.get("Name") if isinstance(entry, dict) else None
    if name:
        return name

    map_entry = map_info_table.get(scene_id)
    if isinstance(map_entry, dict):
        return map_entry.get("Name") or ""

    return ""


def _source_scene_ids() -> list[str]:
    source_path = ZTABLE_DIR / "SceneTable.json"
    if not source_path.exists():
        raise FileNotFoundError(f"未找到中文模板源: {source_path}")

    source_table = _load_json(source_path)
    scene_ids = []
    for entry in source_table.values():
        if isinstance(entry, dict) and "Id" in entry:
            scene_ids.append(str(entry["Id"]))
    return scene_ids


def _build_scene_names(lang: str, scene_ids: list[str]) -> tuple[dict, int]:
    input_path = ZTABLE_DIR / "SceneTable.json"
    map_info_path = ZTABLE_DIR / "MapInfoTable.json"
    overrides_path = LANG_CONFIGS[lang]["overrides_dir"] / OVERRIDE_FILE_NAME

    if not input_path.exists():
        raise FileNotFoundError(f"未找到场景表: {input_path}")

    scene_table = _load_json(input_path)
    map_info_table = _load_json(map_info_path)
    name_overrides = _load_json(overrides_path)

    result = {
        scene_id: _scene_name(scene_id, scene_table, map_info_table)
        for scene_id in scene_ids
    }
    result.update({str(scene_id): name for scene_id, name in name_overrides.items()})

    return result, len(name_overrides)


def _write_scene_names(lang: str, result: dict, override_count: int) -> None:
    output_path = OUTPUT_DIR / LANG_CONFIGS[lang]["output_name"]
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=2, sort_keys=True)

    print(f"已生成: {output_path}，共 {len(result)} 条（{override_count} 条由例外文件覆盖）")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="从 ZTable/SceneTable.json 提取多语言场景 Id -> Name。"
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
        scene_ids = _source_scene_ids()
        for lang in langs:
            result, override_count = _build_scene_names(lang, scene_ids)
            _write_scene_names(lang, result, override_count)
    except FileNotFoundError as exc:
        print(f"错误: {exc}")
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
