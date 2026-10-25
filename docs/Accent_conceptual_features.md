# Accent — A Multiplatform Markdown Editor and Viewer

Premesso che Apostrophe è un editor Markdown per GNOME che consente di effettuare la *preview* del testo digitato dall'utente in un *frame* laterale o in una finestra separata, **Accent** nasce come evoluzione multipiattaforma ispirata alla pulizia visiva e alle funzionalità di Apostrophe, superandone i limiti strutturali.

Di seguito sono definite le specifiche funzionali e architetturali di Accent:

1. **Architettura 100% Rust + `egui` Multipiattaforma**: Accent è scritto interamente in **Rust** utilizzando **`eframe` / `egui`** e distribuito come singolo binario autocontenuto senza dipendenze di sistema esterne. La *GUI* riprende l'eleganza minimalista e le palette cromatiche di Apostrophe (es. temi *Light*, *Sepia* e *Dark*), migliorandole con toni, contrasti e spaziature studiati per favorire la concentrazione profonda (*focus mode*) e la leggibilità prolungata a schermo.

2. **Modalità di Preview e *View-while-typing* (Live Preview)**: Oltre a tutte le modalità di *preview* classiche previste da Apostrophe — visualizzabili in un pannello affiancato (*split view*) oppure in una **finestra indipendente del sistema operativo** tramite le API *Multi-Viewport* native di `egui` — Accent integra la modalità **View-while-typing** (in stile Typora/Obsidian). Man mano che l'utente digita, l'editor interpreta e formatta il testo in tempo reale. Ad esempio, non appena l'utente completa la digitazione di `**parola**`, Accent nasconde gli asterischi e mostra direttamente il termine **parola** in grassetto; quando il cursore di testo (*caret*) entra nuovamente all'interno della parola (tramite frecce direzionali o click del mouse), l'editor rivela istantaneamente il markup sorgente `**parola**` per consentirne la modifica, tornando alla versione renderizzata non appena il cursore esce dallo span.

3. **Larghezza riga e margini configurabili**: Accent supera il limite rigido di Apostrophe (~70 caratteri per riga) offrendo un selettore rapido nella barra superiore e nelle Preferenze che permette di scegliere tra:
   - **Compatta**: ~80 caratteri per riga
   - **Media**: 100–120 caratteri per riga
   - **Ampia**: 140+ caratteri per riga
   - **Larghezza intera (Fluid / Full-Width)**: adattamento dinamico alla finestra con margine laterale (in percentuale o in pixel) liberamente regolabile tramite slider.

4. **Tipografia curata per la lettura a schermo e gestione Font nativa**: In Accent il font della prosa (*Body/Prose*) è configurabile separatamente dal font del codice (*Inline code* / *Code block*). I preset predefiniti includono font ad alta leggibilità a schermo (*Inter* / *Cantarell* per il Sans-Serif, *Literata* / *Charter* per la lettura prolungata in Serif, e *JetBrains Mono* per i blocchi di codice o la modalità sorgente). Nel pannello nativo delle Preferenze (`egui`), un selettore alimentato dal crate Rust **`fontdb`** consente di esplorare e selezionare qualsiasi font installato nel sistema operativo, oltre a regolare con precisione la dimensione del testo (*Font Size*) e l'altezza dell'interlinea (*Line Height* tramite `egui::TextFormat`).

5. **Supporto nativo ai dialetti Markdown**: Attraverso una pipeline di parsing interamente scritta in Rust (basata su `comrak` / `pulldown-cmark` con estensioni AST), Accent supporta nativamente:
   - **Standard CommonMark**
   - **GFM (GitHub Flavored Markdown)**: tabelle, task list, strikethrough, autolink e alert
   - **GLFM (GitLab Flavored Markdown)**: estensioni avanzate compatibili con la pipeline nativa Rust (blocchi matematici, note a piè di pagina, frontmatter YAML/TOML).

6. **Supporto Mermaid integrato (Zero-WebView)**: Supporto nativo per la creazione e visualizzazione di diagrammi e grafici di flusso direttamente nel documento. I blocchi di codice `mermaid` vengono convertiti in SVG in puro Rust e rasterizzati su texture GPU tramite **`resvg`**, garantendo un rendering nitido su schermi HiDPI senza ricorrere a browser esterni o componenti WebView.

7. **Esportazione 100% Rust Embedded (Zero dipendenze esterne)**: Accent consente di esportare direttamente il documento in **PDF**, **Word (`.docx`)** ed **EPUB** senza richiedere l'installazione di alcun software esterno sul sistema dell'utente. L'intera pipeline di generazione dei documenti è compilata staticamente all'interno dell'eseguibile tramite librerie Rust dedicate (es. `krilla` / `typst` per PDF, `docx-rs` per Word, `epub-builder` per EPUB).

8. **Affidabilità senza compromessi e Crash Recovery**: Accent è progettato secondo il principio *zero data loss*: in nessun caso è ammessa la perdita di dati o la corruzione dei file dell'utente, nemmeno a fronte di un improvviso *blackout* elettrico o di un *crash* di sistema. L'applicazione implementa:
   - **Journaling *Save-while-typing***: ad ogni modifica (con *debounce* inferiore a 300ms), lo stato dell'editor viene salvato in un journal di ripristino locale dedicato, consentendo al successivo riavvio di riprendere il lavoro esattamente dall'ultima parola digitata senza aver mai alterato a metà il file originale.
   - **Salvataggio Atomico (*Write-Temp $\rightarrow$ Fsync $\rightarrow$ Atomic Rename*)**: ogni salvataggio sul file `.md` di destinazione scrive prima su un file temporaneo nella medesima partizione, forza il flush fisico dei buffer su disco (`sync_all`) ed esegue una rinomina atomica a livello di file system.

9. **Internazionalizzazione (i18n)**: Accent è progettato con un sistema di localizzazione nativo in Rust (es. `fluent` / `rust-i18n`) per facilitare l'aggiunta di nuove traduzioni. Sin dalla prima *release* sono supportate integralmente tre lingue: **inglese**, **italiano** e **spagnolo**.

___________ OLD
# Accent - a multiplatform Markdown editor and viewer

Premesso che Apostrophe è un editor markdown per Gnome che consente di effettuare la *preview* del testo *markdown* digitato dall'utente in un frame o in una finestra separata, Accent dovrebbe essere un replica fedele di Apostrophe, sia delle funzionalità che dell'aspetto.
Inoltre, dovrebbe avere ulteriori funzionalità che Apostrophe non possiede:

1. Accent è scritto in **Rust + egui** ed è **multipiattaforma**: la *GUI* imita il più possibile quella di Apostrophe anche dal punto di visto delle palette di colori utilizzati. Accent migliora ulteriormente l'aspetto e utilizza toni e colori che favoriscono la concentrazione e la leggibilità del testo. 

2. **View while typing**: oltre a tutte le modalità di *preview* previste da Apostrophe in un *frame* o una finestra separata da quella in cui l'utente scrive il testo markdown, Accent dovrebbe essere dotato anche della modalità di *view-while-typing* propria di Typora (un altro famoso editor/viewer markdown): man mano che l'utente scrive testo markdown, la app dovrebbe interpretare automaticamente il testo markup. Ad esempio, non appena l'utente abbia completato di scrivere il testo "**parola**", Accent dovrebbe visualizzare il termine "parola" in grassetto senza gli asterischi: quando l'utente punta il mouse sul testo in grassetto "parola", l'editor mostra immediatamente il markup "**parola**" consentendogli di apportare eventuali modifiche.

3. **Larghezza riga e margini configurabili**: bisogna superare il limite di Apostrophe fissato a ~70 caratteri per ciascuna riga con un'opzione nelle Preferenze (e/o nel menu rapido in alto) che permette di scegliere tra: **Compatta** (riga di 80 caratteri), **Media** (riga di 100-120 caratteri), **Ampia** (riga di 140+ caratteri) o **Larghezza intera** (Fluid / Full-Width) con un semplice margine percentuale o in pixel personalizzabile tramite slider.

4. **Tipografia curata per la lettura a schermo + Scelta Font**: in Accent il Font del testo (Prosa) può essere diverso dal Font del codice (Inline code / Code block). I font di default o preset rapidi devono essere font progettati appositamente per la massima leggibilità a schermo (es. Inter / Cantarell per il Sans-Serif, Literata / Charter per la lettura prolungata Serif, e JetBrains Mono / Adwaita Mono solo per i blocchi di codice o se l'utente desidera esplicitamente il monospace).

5. **Molteplici dialetti markdown**: Accent supporta lo Standard CommonMark, GFM (GitHub Flavored Markdown), GitLab Flavored Markdown (GLFM) e Pandoc Markdown.

6. **Supporto Mermaid**: Supporto nativo per la creazione di diagrammi e grafici di flusso direttamente nel testo.

7. **Export in vari formati**: Accent consente di esportare il testo scritto in PDF, Word e EPUB.

8. **Affidabilità senza compromessi**: Accent deve essere progettato per essere estremamente affidabile: in nessun caso è ammessa la perdita dei dati o la generazione di *file* corrotti. In particolare, Accent implementa un meccanismo di *save-while-typing* che consente di recuperare il lavoro in caso di *crash* o di altri eventi anomali, come ad esempio un improvviso *blackout* elettrico: al successivo riavvio Accent deve essere in grado di riprendere l'editing del documento dall'ultima parola scritta.

9. **Internazionalizzazione**: Accent è progettato per essere facilmente tradotto in altre lingue, con pieno supporto i18n. Sin dalla prima *release* sono supportati l'inglese, l'italiano e lo spagnolo.