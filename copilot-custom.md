# `copilot-custom.md` \- Documentazione impostazioni Copilot

**Generato automaticamente da** `copilot-custom.json`. Non modificare manualmente questo file.

## Scopo
Descrivere in italiano le impostazioni definite nel file `copilot-custom.json` che influenzano il comportamento dell'assistente per questo progetto.

## Dove salvarlo
Posiziona questo file nella root del repository (accanto a `Cargo.toml`) insieme a `copilot-custom.json`.

## Panoramica del file JSON
- `version`: versione della struttura di configurazione.
- `defaults`: valori predefiniti per il comportamento dell'assistente.
- `prompts`: messaggi/prompts personalizzati usati come sistema o titoli.

## Descrizione dei campi principali

### `version`
- Indica la versione del formato di configurazione (es. `1`).

### `defaults`
- `always_generate_rust`: `true`
- `avoid_redundant_intro`: `true`
- `brief_explanations_only`: `true`
- `comment_language`: `en`
- `doc_comment_style`: `///`
- `doc_comments_language`: `en`
- `escape_markdown_chars_in_text`: `true`
- `explain_code`: `true`
- `force_comments_english`: `true`
- `force_doc_comments_english`: `true`
- `force_preferred_code_language`: `true`
- `format_response`: `markdown`
- `include_code_blocks`: `true`
- `include_tests`: `true`
- `language`: `it`
- `max_response_lines`: `40`
- `prefer_rust_examples`: `true`
- `preferred_code_language`: `rust`
- `sign_off`: `false`
- `test_framework`: `cargo`
- `tone`: `conciso`
- `use_unix_line_endings`: `false`
- `verbosity`: `bassa`

### `prompts`
- `code_block_title`: Esempio di codice (Rust)
- `default_system`: Se richiesto, fornisci risposte concise in italiano. Quando generi codice, usa esclusivamente Rust. Tutti i commenti e i blocchi di documentazione nel codice devono essere in inglese ("doc comments" con stile `///`). Genera test `cargo` semplici quando applicabile.
- `test_block_title`: Test automatici (cargo)

## Note pratiche
- Mantieni `copilot-custom.json` come fonte primaria; questo file viene rigenerato automaticamente.
- Aggiungi uno step CI o un pre\-commit hook che esegua la rigenerazione e fallisca il commit se ci sono diff.
