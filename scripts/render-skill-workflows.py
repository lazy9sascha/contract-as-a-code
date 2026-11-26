import argparse
import json
from pathlib import Path


def render(data, source):
    chart = data["flowchart"]
    node_ids = {node["id"] for node in chart["nodes"]}
    if len(node_ids) != len(chart["nodes"]) or chart["start"] not in node_ids:
        raise ValueError("Use unique flowchart node IDs and an existing start node.")

    lines = [
        f"# {data['title']}",
        "",
        f"{data['summary']} [JSON data]({source.name})",
        "",
        "## Feature map",
        "",
        "| User goal | Skill | Result |",
        "| --- | --- | --- |",
    ]
    for feature in data["features"]:
        if feature["id"] not in node_ids:
            raise ValueError(f"Add a flowchart node for feature {feature['id']}.")
        if not (source.parent / feature["path"]).is_file():
            raise ValueError(f"Skill file missing at {feature['path']}.")
        lines.append(
            "| {request} | [`{skill}`]({path}) | {result} |".format(**feature)
        )

    lines += ["", "## Contract-first implementation", "", "| Role | Responsibility |", "| --- | --- |"]
    for role in data["implementation_roles"]:
        lines.append("| {role} | {responsibility} |".format(**role))

    mcp = data["mcp_review"]
    lines += ["", "## Local MCP review", "", mcp["scope"], "", mcp["input"], "", mcp["result"], ""]
    lines += [mcp["verification"], "", "```json", json.dumps(mcp["client_configuration"], indent=2), "```", ""]
    lines += [mcp["follow_up"], ""]
    lines += ["## Flowchart", "", "```mermaid", "flowchart TD"]
    for node in chart["nodes"]:
        label = f'"{node["label"]}"'
        shape = f"{{{label}}}" if node.get("shape") == "decision" else f"[{label}]"
        lines.append(f"    {node['id']}{shape}")
    for edge in chart["edges"]:
        if edge["from"] not in node_ids or edge["to"] not in node_ids:
            raise ValueError(f"Use existing nodes for {edge['from']} -> {edge['to']}.")
        connector = f" -->|{edge['label']}| " if "label" in edge else " --> "
        lines.append(f"    {edge['from']}{connector}{edge['to']}")
    lines += ["```", ""]
    return "\n".join(lines).encode("utf-8")


def main():
    parser = argparse.ArgumentParser(description="Render the skill workflow guide from JSON.")
    parser.add_argument("--check", action="store_true", help="fail if Markdown needs regeneration")
    args = parser.parse_args()
    source = Path(__file__).resolve().parents[1] / "docs/skill-workflows.json"
    output = source.with_suffix(".md")
    try:
        content = render(json.loads(source.read_text(encoding="utf-8")), source)
        if not args.check:
            output.write_bytes(content)
        elif not output.is_file() or output.read_bytes() != content:
            parser.exit(
                1,
                "docs/skill-workflows.md is out of date. "
                "Run python3 scripts/render-skill-workflows.py.\n",
            )
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f"docs/skill-workflows.json: {error}\n")


if __name__ == "__main__":
    main()
