# -*- coding: utf-8 -*-
"""从 ZTable/DamageAttrTable.json 提取 Id -> Name，输出 JSON 到 scripts/output。
当 Name 为空时，按 DamageType 决定 TypeEnum 的优先查表顺序：
1 -> SkillTable，2 -> BuffTable，3 -> BulletTable；未命中时再查其他表
（优先 NameDesign，回退 Name）。
最后应用 scripts/overrides/damage_attr_id_name_overrides.json 里的名称覆盖。
"""

import json
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
ZTABLE_DIR = PROJECT_ROOT / "ZTable"
OUTPUT_DIR = SCRIPT_DIR / "output"


def _pick_name(entry: dict) -> str | None:
    """从一条记录里取名字：优先 NameDesign，其次 Name。"""
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
    """根据 DamageType 把最可能命中的表排到最前。"""
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
    """按给定顺序在多个表里用 TypeEnum 作为 key 查找名称。

    返回 (名称, 命中的表名)；都没命中则返回 (None, None)。
    """
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


def _load_json(path: Path) -> dict:
    if not path.exists():
        return {}
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def main():
    input_path = ZTABLE_DIR / "DamageAttrTable.json"
    skill_path = ZTABLE_DIR / "SkillTable.json"
    buff_path = ZTABLE_DIR / "BuffTable.json"
    bullet_path = ZTABLE_DIR / "BulletTable.json"
    overrides_path = SCRIPT_DIR / "overrides" / "damage_attr_id_name_overrides.json"
    output_path = OUTPUT_DIR / "DamageAttrIdName.json"

    if not input_path.exists():
        print(f"错误: 未找到 {input_path}")
        return 1

    table = _load_json(input_path)
    skill_table = _load_json(skill_path)
    buff_table = _load_json(buff_path)
    bullet_table = _load_json(bullet_path)
    name_overrides = _load_json(overrides_path)

    lookup_tables = [
        ("SkillTable", skill_table),
        ("BuffTable", buff_table),
        ("BulletTable", bullet_table),
    ]

    result = {}
    fallback_stats = {name: 0 for name, _ in lookup_tables}
    unknown_ids = []

    for entry in table.values():
        if not (isinstance(entry, dict) and "Id" in entry):
            continue

        attr_id = entry["Id"]
        name = entry.get("Name")
        if name:
            result[str(attr_id)] = name
            continue

        fallback, hit_table = _resolve_by_type_enum(
            entry.get("TypeEnum"), entry.get("DamageType"), lookup_tables
        )
        if fallback:
            result[str(attr_id)] = fallback
            fallback_stats[hit_table] += 1
        else:
            unknown_ids.append(attr_id)

    result.update({str(attr_id): name for attr_id, name in name_overrides.items()})

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=2, sort_keys=True)

    fallback_total = sum(fallback_stats.values())
    print(
        f"已生成: {output_path}，共 {len(result)} 条"
        f"（其中 {fallback_total} 条由 TypeEnum 补全："
        f"Skill={fallback_stats['SkillTable']}, "
        f"Buff={fallback_stats['BuffTable']}, "
        f"Bullet={fallback_stats['BulletTable']}；"
        f"{len(unknown_ids)} 条仍无名称；"
        f"{len(name_overrides)} 条由例外文件覆盖）"
    )
    if unknown_ids:
        print(f"仍无名称的前 10 条 ID: {unknown_ids[:10]}")

if __name__ == "__main__":
    main()
