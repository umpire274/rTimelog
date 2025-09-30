# python
#!/usr/bin/env python3
import json
from pathlib import Path
import argparse
import sys

def generate_md(data):
    defaults = data.get("defaults", {})
    prompts = data.get("prompts", {})
    version = data.get("version", "")

    use_unix = bool(defaults.get("use_unix_line_endings", False))
    nl = "\n" if use_unix else "\r\n"

    lines = []
    lines.append("# `copilot-custom.md` \\- Documentazione impostazioni Copilot")
    lines.append("")
    lines.append("**Generato automaticamente da** `copilot-custom.json`. Non modificare manualmente questo file.")
    lines.append("")
    lines.append("## Scopo")
    lines.append("Descrivere in italiano le impostazioni definite nel file `copilot-custom.json` che influenzano il comportamento dell'assistente per questo progetto.")
    lines.append("")
    lines.append("## Dove salvarlo")
    lines.append("Posiziona questo file nella root del repository (accanto a `Cargo.toml`) insieme a `copilot-custom.json`.")
    lines.append("")
    lines.append("## Panoramica del file JSON")
    lines.append("- `version`: versione della struttura di configurazione.")
    lines.append("- `defaults`: valori predefiniti per il comportamento dell'assistente.")
    lines.append("- `prompts`: messaggi/prompts personalizzati usati come sistema o titoli.")
    lines.append("")
    lines.append("## Descrizione dei campi principali")
    lines.append("")
    lines.append("### `version`")
    lines.append(f"- Indica la versione del formato di configurazione (es. `{version}`).")
    lines.append("")
    lines.append("### `defaults`")
    for k in sorted(defaults.keys()):
        v = defaults[k]
        if isinstance(v, str):
            display = v
        else:
            display = json.dumps(v, ensure_ascii=False)
        lines.append(f"- `{k}`: `{display}`")
    lines.append("")
    lines.append("### `prompts`")
    for k in sorted(prompts.keys()):
        v = prompts[k]
        display = v.replace("\r", "\\r").replace("\n", "\\n")
        lines.append(f"- `{k}`: {display}")
    lines.append("")
    lines.append("## Note pratiche")
    lines.append("- Mantieni `copilot-custom.json` come fonte primaria; questo file viene rigenerato automaticamente.")
    lines.append("- Aggiungi uno step CI o un pre\\-commit hook che esegua la rigenerazione e fallisca il commit se ci sono diff.")
    lines.append("")

    return nl.join(lines)

def main():
    p = argparse.ArgumentParser(description="Generate copilot-custom.md from copilot-custom.json")
    p.add_argument("--input", "-i", default="copilot-custom.json")
    p.add_argument("--output", "-o", default="copilot-custom.md")
    args = p.parse_args()

    in_path = Path(args.input)
    out_path = Path(args.output)

    if not in_path.exists():
        print(f"Errore: `{in_path}` non trovato.", file=sys.stderr)
        sys.exit(2)

    try:
        data = json.loads(in_path.read_text(encoding="utf-8"))
    except Exception as e:
        print(f"Errore durante la lettura di `{in_path}`: {e}", file=sys.stderr)
        sys.exit(2)

    md = generate_md(data)
    out_path.write_text(md, encoding="utf-8")
    # Avoid complex inline f-string expression that can confuse Python parsing
    defaults = data.get('defaults', {})
    use_unix = bool(defaults.get('use_unix_line_endings', False))
    print(f"Generato `{out_path}` (use_unix_line_endings={use_unix})")

if __name__ == "__main__":
    main()