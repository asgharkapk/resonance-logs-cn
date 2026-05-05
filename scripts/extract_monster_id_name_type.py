# -*- coding: utf-8 -*-
"""从 ZTable/MonsterTable.json 提取 Id -> {Name, MonsterType}，输出 JSON 到 scripts/output。
最后应用 scripts/overrides/monster_id_name_type_overrides.json 里的覆盖。"""

import json
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
ZTABLE_DIR = PROJECT_ROOT / "ZTable"
OUTPUT_DIR = SCRIPT_DIR / "output"


def _load_json(path: Path) -> dict:
    if not path.exists():
        return {}
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _apply_overrides(result: dict, overrides: dict) -> None:
    for monster_id, override in overrides.items():
        key = str(monster_id)
        if isinstance(override, str):
            if key in result:
                result[key]["Name"] = override
            else:
                result[key] = {"Name": override}
            continue

        if not isinstance(override, dict):
            continue

        target = result.setdefault(key, {})
        if "Name" in override:
            target["Name"] = override["Name"]
        if "MonsterType" in override:
            target["MonsterType"] = override["MonsterType"]


def _sort_by_key(data: dict) -> dict:
    return {key: data[key] for key in sorted(data)}


def main():
    input_path = ZTABLE_DIR / "MonsterTable.json"
    overrides_path = SCRIPT_DIR / "overrides" / "monster_id_name_type_overrides.json"
    name_type_output_path = OUTPUT_DIR / "MonsterIdNameType.json"
    type_output_path = OUTPUT_DIR / "MonsterIdType.json"

    if not input_path.exists():
        print(f"错误: 未找到 {input_path}")
        return 1

    table = _load_json(input_path)
    overrides = _load_json(overrides_path)

    result = {}
    for entry in table.values():
        if not isinstance(entry, dict):
            continue
        if "Id" not in entry or "Name" not in entry or "MonsterType" not in entry:
            continue

        try:
            eid = int(entry["Id"])
        except (TypeError, ValueError):
            eid = entry["Id"]

        result[str(entry["Id"])] = {
            "Name": entry["Name"],
            "MonsterType": entry["MonsterType"],
        }

    _apply_overrides(result, overrides)
    result = _sort_by_key(result)
    type_result = _sort_by_key(
        {
            monster_id: entry["MonsterType"]
            for monster_id, entry in result.items()
            if isinstance(entry, dict) and "MonsterType" in entry
        }
    )

    name_type_output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(name_type_output_path, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=2)
    with open(type_output_path, "w", encoding="utf-8") as f:
        json.dump(type_result, f, ensure_ascii=False, indent=2)

    print(
        f"已生成: {name_type_output_path}，共 {len(result)} 条"
        f"（{len(overrides)} 条由例外文件覆盖）"
    )
    print(f"已生成: {type_output_path}，共 {len(type_result)} 条")


if __name__ == "__main__":
    main()
