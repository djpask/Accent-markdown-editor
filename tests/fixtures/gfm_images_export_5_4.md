# Phase 5.4 — Immagini Locali (`§6.7`) & Esportatori AST

Questo documento verifica la risoluzione sicura dei path relativi rispetto a `current_file_path` (sia in **Anteprima in-line •** con `ImageWidget` caret-aware sia nello **Split Preview Pane** via protocollo `asset:` / `data:` sanitizzato) e la fedeltà degli esportatori Rust.

![Diagramma Pipeline Accent 5.4](./assets/accent_pipeline_5_4.svg "Architettura Unificata L1–L5")

Icona locale PNG risolta con path relativo `./assets/accent_logo_5_4.png` e `../../img/brand/accent_logo.png`: ![Logo Locale Assets](./assets/accent_logo_5_4.png "Logo Locale PNG in assets/") integrata nel paragrafo insieme a ![Logo Brand](../../img/brand/accent_logo.png "Logo Ufficiale Accent").

Fallback eleganti senza icone *broken image* grezze: ![Asset Locale Mancante](./assets/non_esistente_404.png "File assente") oppure ![Immagine Remota Offline](https://example.com/cloud-banner.png "Bloccata dalla CSP offline-first").

---

## Tabella Ricca (`§4.10`) ed Elenchi Annidati (`§5.2`–`§5.4`)

| Modulo | Allineamento Centro | Esportazione Destra |
|:---|:---:|---:|
| **accent-core** (`L1`) | *AST unificato* & ~~split-brain~~ | `PreviewBlock` completo |
| **accent-export** (`L3`) | Immagini incorporate<br>su **6 formati** | [Specifica GFM](https://github.github.com/gfm/ "GFM 0.29") |

> [!TIP]
> Citazione multi-blocco con **Alert GFM** e hard line break:\
> Seconda riga dopo `\` con codice `asset://localhost`.
>
> 3. [x] Risoluzione path relativi (`./`, `../`, `%20`)
>    - Incorporamento reale in PDF, DOCX, ODT, EPUB e HTML standalone
> 4. [ ] Verifica interattiva click-to-expand sul widget immagine
