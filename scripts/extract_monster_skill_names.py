# -*- coding: utf-8 -*-
"""Extract monster skill Id -> Name mappings into scripts/output.

Scope is limited to the skill ids referenced by MonsterTable.SkillIds so the
output stays a monster-skill table instead of the full (mostly player) skill
table. All languages share the Chinese ZTable as the template source so every
output keeps the same keys. Language-specific override folders supply
translated names.
"""

import argparse
import json
import re
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
OUTPUT_DIR = SCRIPT_DIR / "output"
ZTABLE_DIR = PROJECT_ROOT / "ZTable"

LANG_CONFIGS = {
    "zh": {
        "overrides_dir": SCRIPT_DIR / "overrides",
        "output_name": "MonsterSkillName.json",
    },
    "en": {
        "overrides_dir": SCRIPT_DIR / "overrides_en",
        "output_name": "MonsterSkillName_en.json",
    },
    "jp": {
        "overrides_dir": SCRIPT_DIR / "overrides_jp",
        "output_name": "MonsterSkillName_jp.json",
    },
}

OVERRIDE_FILE_NAME = "monster_skill_name_overrides.json"

# SkillTable.Name is a shared placeholder ("场地标记01") for most monster
# skills; NameDesign carries the planner-facing label instead.
PLACEHOLDER_NAME = "场地标记01"

# Latin letters / digits / separators mark planner-internal labels such as
# "塔塔ATK_01" or "P2缓慢回血技能".
LATIN_PATTERN = re.compile(r"[A-Za-z0-9_\-]")


def _load_json(path: Path) -> dict:
    if not path.exists():
        return {}
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _usable_name(entry: dict, field: str) -> str:
    value = entry.get(field)
    if not isinstance(value, str):
        return ""
    value = value.strip()
    if not value or value == PLACEHOLDER_NAME:
        return ""
    return value


def _monster_skill_name(entry: dict) -> str:
    """NameDesign first; fall back to Name only when NameDesign carries a
    planner-internal latin label while Name is a proper localized name."""
    name_design = _usable_name(entry, "NameDesign")
    name = _usable_name(entry, "Name")
    if name and (not name_design or LATIN_PATTERN.search(name_design)):
        return name
    return name_design or name


def _source_monster_skill_ids() -> list[str]:
    source_path = ZTABLE_DIR / "MonsterTable.json"
    if not source_path.exists():
        raise FileNotFoundError(f"未找到怪物表: {source_path}")

    monster_table = _load_json(source_path)
    skill_ids: set[int] = set()
    for entry in monster_table.values():
        if not isinstance(entry, dict):
            continue
        for item in entry.get("SkillIds") or []:
            if isinstance(item, list):
                skill_ids.update(x for x in item if isinstance(x, int))
            elif isinstance(item, int):
                skill_ids.add(item)
    return [str(skill_id) for skill_id in sorted(skill_ids)]


def _build_monster_skill_names(lang: str, skill_ids: list[str]) -> tuple[dict, int]:
    input_path = ZTABLE_DIR / "SkillTable.json"
    overrides_path = LANG_CONFIGS[lang]["overrides_dir"] / OVERRIDE_FILE_NAME

    if not input_path.exists():
        raise FileNotFoundError(f"未找到技能表: {input_path}")

    skill_table = _load_json(input_path)
    name_overrides = _load_json(overrides_path)

    result = {}
    for skill_id in skill_ids:
        entry = skill_table.get(skill_id)
        if not isinstance(entry, dict):
            continue
        name = _monster_skill_name(entry)
        if name:
            result[skill_id] = name
    result.update({str(skill_id): name for skill_id, name in name_overrides.items()})

    return result, len(name_overrides)


def _write_monster_skill_names(lang: str, result: dict, override_count: int) -> None:
    output_path = OUTPUT_DIR / LANG_CONFIGS[lang]["output_name"]
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(result, f, ensure_ascii=False, indent=2, sort_keys=True)
        f.write("\n")

    print(f"已生成: {output_path}，共 {len(result)} 条（{override_count} 条由例外文件覆盖）")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="从 ZTable/SkillTable.json + MonsterTable.json 提取多语言怪物技能 Id -> Name。"
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
        skill_ids = _source_monster_skill_ids()
        for lang in langs:
            result, override_count = _build_monster_skill_names(lang, skill_ids)
            _write_monster_skill_names(lang, result, override_count)
    except FileNotFoundError as exc:
        print(f"错误: {exc}")
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
