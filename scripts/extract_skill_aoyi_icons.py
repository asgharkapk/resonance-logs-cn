#!/usr/bin/env python3
"""
从 SkillTable.json 中提取 Icon 以 ui/textures/skill_aoyi/skill_aoyi_skill_icon 为前缀的技能，
仅记录 4 位数的 skillId（1000–9999）。
输出 JSON 到 scripts/output：保留 id、NameDesign、Icon；若 SkillType==5 则增加 maxCharges（取 MaxEnergyChargeNum）。

用法:
  python extract_skill_aoyi_icons.py
  python extract_skill_aoyi_icons.py -o custom_output.json
"""

import argparse
import json
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
ZTABLE_DIR = PROJECT_ROOT / "ZTable"
OUTPUT_DIR = SCRIPT_DIR / "output"
SKILL_TABLE_PATH = ZTABLE_DIR / "SkillTable.json"
ICON_PREFIX = "ui/textures/skill_aoyi/skill_aoyi_skill_icon"


def extract_icon_filename(icon_path: str) -> str:
    """从 Icon 路径取最后一段作为文件名，并加上 .png。"""
    if not icon_path:
        return ""
    name = icon_path.replace("\\", "/").split("/")[-1]
    return f"{name}.png"


def main():
    parser = argparse.ArgumentParser(description="提取 skill_aoyi 技能图标信息")
    parser.add_argument(
        "-o", "--output",
        default=str(OUTPUT_DIR / "skill_aoyi_icons.json"),
        help="输出 JSON 路径",
    )
    args = parser.parse_args()

    if not SKILL_TABLE_PATH.exists():
        print(f"错误: 未找到 {SKILL_TABLE_PATH}")
        return 1

    with open(SKILL_TABLE_PATH, "r", encoding="utf-8") as f:
        skill_table = json.load(f)

    results = []
    for key, entry in skill_table.items():
        if not isinstance(entry, dict):
            continue
        icon = entry.get("Icon")
        if not icon or not str(icon).startswith(ICON_PREFIX):
            continue
        skill_id = entry.get("Id")
        if skill_id is None:
            skill_id = key
        skill_id = int(skill_id)
        # 只记录 4 位数的 skillId（1000–9999）
        if not (1000 <= skill_id <= 9999):
            continue
        item = {
            "id": skill_id,
            "NameDesign": entry.get("NameDesign", ""),
            "Icon": extract_icon_filename(icon),
        }
        if entry.get("SkillType") == 5:
            item["maxCharges"] = entry.get("MaxEnergyChargeNum", 0)
        results.append(item)

    results.sort(key=lambda x: (x["id"], x["Icon"]))

    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(results, f, ensure_ascii=False, indent=2)

    print(f"已提取 {len(results)} 条记录 -> {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
