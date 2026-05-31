# -*- coding: utf-8 -*-
"""从 ZTable/DamageAttrTable.json 提取 Id -> Name，输出 JSON 到 scripts/output。

所有语言以中文 ZTable 为模板源，保证各语言输出键一致。
各语言 override 目录中的 JSON 用于覆盖翻译名称。
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
        "output_name": "DamageAttrIdName.json",
    },
    "en": {
        "overrides_dir": SCRIPT_DIR / "overrides_en",
        "output_name": "DamageAttrIdName_en.json",
    },
    "jp": {
        "overrides_dir": SCRIPT_DIR / "overrides_jp",
        "output_name": "DamageAttrIdName_jp.json",
    },
}

OVERRIDE_FILE_NAME = "damage_attr_id_name_overrides.json"


def _load_json(path: Path) -> dict:
    if not path.exists():
        return {}
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _pick_name(entry: dict) -> str | None:
    if not isinstance(entry, dict):
        return None
    for field in ("NameDesign", "Name"):
        value = entry.get(field)
        if value:
            return value
    return None


def _ordered_lookup_tables(
    damage_type, lookup_tables: list[tuple[str, dict]]
) -> list[tuple[str, dict]]:
    preferred_by_damage_type = {
        1: "SkillTable",
        2: "BuffTable",
        3: "BulletTable",
    }
    preferred = preferred_by_damage_type.get(damage_type)
    if not preferred:
        return lookup_tables
    return sorted(lookup_tables, key=lambda item: item[0] != preferred)


def _resolve_by_type_enum(
    type_enum, damage_type, lookup_tables: list[tuple[str, dict]]
) -> tuple[str | None, str | None]:
    if type_enum is None:
        return None, None
    key = str(type_enum)
    for table_name, table in _ordered_lookup_tables(damage_type, lookup_tables):
        if not table:
            continue
        entry = table.get(key)
        name = _pick_name(entry)
        if name:
            return name, table_name
    return None, None


def _build_base_names(table: dict, lookup_tables: list[tuple[str, dict]]) -> dict[str, str]:
    result: dict[str, str] = {}
    for entry in table.values():
        if not (isinstance(entry, dict) and "Id" in entry):
            continue

        attr_id = str(entry["Id"])
        name = entry.get("Name")
        if name:
            result[attr_id] = name
            continue

        fallback, _ = _resolve_by_type_enum(
            entry.get("TypeEnum"), entry.get("DamageType"), lookup_tables
        )
        if fallback:
            result[attr_id] = fallback

    return result


def _build_damage_attr_names(lang: str, table: dict) -> tuple[dict, int]:
    skill_table = _load_json(ZTABLE_DIR / "SkillTable.json")
    buff_table = _load_json(ZTABLE_DIR / "BuffTable.json")
    bullet_table = _load_json(ZTABLE_DIR / "BulletTable.json")
    overrides_path = LANG_CONFIGS[lang]["overrides_dir"] / OVERRIDE_FILE_NAME
    name_overrides = _load_json(overrides_path)

    lookup_tables = [
        ("SkillTable", skill_table),
        ("BuffTable", buff_table),
        ("BulletTable", bullet_table),
    ]

    result = _build_base_names(table, lookup_tables)
    result.update({str(attr_id): name for attr_id, name in name_overrides.items()})

    return result, len(name_overrides)


def _write_damage_attr_names(lang: str, result: dict, override_count: int) -> None:
    output_path = OUTPUT_DIR / LANG_CONFIGS[lang]["output_name"]
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=2, sort_keys=True)

    print(f"已生成: {output_path}，共 {len(result)} 条（{override_count} 条由例外文件覆盖）")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="从 ZTable/DamageAttrTable.json 提取多语言伤害属性 Id -> Name。"
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

    input_path = ZTABLE_DIR / "DamageAttrTable.json"
    if not input_path.exists():
        print(f"错误: 未找到伤害属性表: {input_path}")
        return 1

    table = _load_json(input_path)

    zh_result, _ = _build_damage_attr_names("zh", table)
    canonical_keys = set(zh_result.keys())

    for lang in langs:
        result, override_count = _build_damage_attr_names(lang, table)
        result = {key: result[key] for key in sorted(canonical_keys) if key in result}
        _write_damage_attr_names(lang, result, override_count)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
