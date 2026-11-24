# Test Completo Tabelle GFM `0.29-gfm` (`§4.10`)

Paragrafo iniziale esterno alla tabella (posiziona qui il cursore per verificare il collasso live in `TableWidget`, oppure clicca una singola cella per aprirla sul punto esatto).

## 1. Allineamenti, Pipe Escapati `\|`, Code Span `` `a | b` ``, Inline Annidati e Math

| Allineamento Sinistra | Allineamento Centro | Allineamento Destra | Edge Case Inline, `\|` & Math |
| :--- | :---: | ---: | --- |
| Testo semplice &copy; | **Grassetto** | `1.250,00` &euro; | Pipe escapati: `a \| b` e fuori codice: A \| B |
| *Corsivo* e ~~barrato **bold**~~ | `codice_inline()` | `42,5%` | Code span con pipe: `a | b` e `` `x | y` `` |
| Annidati: **bold *italic*** e *em **bold*** | Formula: $E = mc^2$ | `999.999` &#8594; | Link [Voce Rust (wiki)](https://en.wikipedia.org/wiki/Rust_(programming_language)) e escape \*\*non bold\*\* |
| Sicurezza DOM (no XSS) | `<kbd>`: <kbd>Ctrl</kbd>+<kbd>L</kbd> | $100 - $250 | Tag grezzi neutralizzati: <script>alert(1)</script> <img src=x onerror=alert(1)> |

## 2. Conteggio Colonne Fissato da `TableHeader` (Celle Mancanti & Celle in Eccesso)

Colonna A (Default) | Colonna B (Centro) | Colonna C (Destra)
--- | :---: | ---:
Solo cella A (B e C completate vuote)
Cella 1 |  | Cella 3 (B vuota esplicita)
Uno | Due | Tre | Quattro (eccesso ignorato) | Cinque (ignorato)

## 3. Tabella Larga (`overflow-x: auto` senza sfondare la colonna `80ch`)

| Modulo | Livello | Responsabilità Principale | Parser / Motore | Stato Conformità | Note Tecniche Estese |
| :--- | :---: | :--- | :--- | :---: | :--- |
| `accent-core` | `L1` | Buffer `Rope` + AST unificato + statistiche UTF-16 | `comrak` (`0.29-gfm`) | Conforme | Nessuna dipendenza GUI o Tauri |
| `accent-export` | `L3` | Anteprima HTML unificata + esportatori PDF/DOCX/ODT/EPUB/LaTeX | `comrak` + `resvg` | Conforme | Tagfilter `§6.11` e scoping `.preview-prose` |
| `accent-ui` | `L5` | Shell desktop CSD + multi-tab + `TableWidget` caret-aware | Svelte 5 + CodeMirror 6 | Conforme | `StateField<DecorationSet>` con `eq()` e click-to-cell |

Paragrafo finale sotto tutte le tabelle per il test di uscita del cursore.
