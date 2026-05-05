# -*- coding: utf-8 -*-
"""从 ZTable/SceneTable.json 提取 Id -> Name，输出 JSON 到 scripts/output/SceneName.json。
若场景 Name 为空，则用 Id 查 ZTable/MapInfoTable.json 的 Name 填入。
最后应用 scripts/overrides/scene_name_overrides.json 里的名称覆盖。"""

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


def main():
    input_path = ZTABLE_DIR / "SceneTable.json"
    map_info_path = ZTABLE_DIR / "MapInfoTable.json"
    overrides_path = SCRIPT_DIR / "overrides" / "scene_name_overrides.json"
    output_path = OUTPUT_DIR / "SceneName.json"

    if not input_path.exists():
        print(f"错误: 未找到 {input_path}")
        return 1

    table = _load_json(input_path)
    map_info_table = _load_json(map_info_path)
    name_overrides = _load_json(overrides_path)

    result = {}
    for key, entry in table.items():
        if isinstance(entry, dict) and "Id" in entry and "Name" in entry:
            name = entry["Name"]
            if not name:
                id_str = str(entry["Id"])
                map_entry = map_info_table.get(id_str)
                if isinstance(map_entry, dict) and "Name" in map_entry:
                    name = map_entry["Name"]
            result[str(entry["Id"])] = name

    result.update({str(scene_id): name for scene_id, name in name_overrides.items()})

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=2, sort_keys=True)

    print(f"已生成: {output_path}，共 {len(result)} 条（{len(name_overrides)} 条由例外文件覆盖）")

if __name__ == "__main__":
    main()
