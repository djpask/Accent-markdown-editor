# Accent — A Multiplatform Markdown Editor and Viewer

**Accent** è un editor e visualizzatore Markdown multipiattaforma progettato per unire pulizia visiva, rigorosa conformità agli standard e affidabilità assoluta dei dati locali.

Di seguito sono riassunti i pilastri concettuali e architetturali di Accent:

1. **Architettura Backend Rust + Desktop Shell Tauri v2 + Svelte 5 + CodeMirror 6**: Il cuore applicativo (`accent-core`, `accent-storage`, `accent-mermaid`, `accent-export`, `accent-session`) è realizzato in **Rust** puro attorno a un'unica pipeline AST `comrak`, mentre l'interfaccia desktop utilizza **Tauri v2**, **Svelte 5** e **CodeMirror 6** con rendering nativo dei caratteri di sistema e quattro modalità di luminosità (*Light*, *Sepia*, *Semi-Dark* e *Dark*) abbinate a palette cromatiche ergonomiche.

2. **Modalità di Preview e *View-while-typing* (Live Preview)**: Oltre alle modalità di anteprima affiancata (*split a destra*, *split sotto*) e *solo lettura*, Accent integra la modalità **View-while-typing** sensibile alla posizione del cursore. Man mano che l'utente digita, l'editor interpreta e formatta il testo in tempo reale: ad esempio, completando `**parola**`, i delimitatori vengono nascosti mostrando **parola** in grassetto e riappaiono istantaneamente non appena il cursore entra nello span per consentirne la modifica.

3. **Larghezza riga e margini configurabili**: Selettore rapido nella barra superiore e nelle Preferenze per scegliere tra:
   - **Compatta**: ~80 caratteri per riga
   - **Media**: 110 caratteri per riga
   - **Ampia**: 140+ caratteri per riga
   - **Larghezza intera (Fluid / Full-Width)**: adattamento dinamico alla finestra con margine laterale in percentuale regolabile tramite slider.

4. **Tipografia curata per la lettura a schermo e gestione Font nativa**: Il font della prosa (*Body/Prose*) è configurabile separatamente dal font del codice (*Inline code* / *Code block*). Attraverso il crate Rust **`fontdb`**, le Preferenze permettono di selezionare qualsiasi famiglia tipografica installata nel sistema operativo e di regolare con precisione dimensione del testo e interlinea.

5. **Configurazione Markdown a due livelli (`MarkdownConfig`)**: Attraverso la pipeline AST unificata basata su `comrak`, Accent supporta:
   - **Grammatica Base**: **GFM Standard (`0.29-gfm`)** (tabelle `§4.10`, task list `§5.3`, barrato `§6.5`, autolink estesi `§6.9` e filtro HTML `§6.11`) oppure **CommonMark puro (`0.29`)**.
   - **Estensioni Modulari** (attivabili singolarmente su GFM Standard): formule matematiche KaTeX, diagrammi Mermaid, blocchi di avviso (*Alerts/Callouts*), note a piè di pagina e frontmatter YAML.

6. **Supporto Mermaid integrato (100% Offline)**: Creazione e visualizzazione di diagrammi direttamente nel documento. I blocchi di codice `mermaid` vengono convertiti in SVG tematici in puro Rust (`accent-mermaid`) e rasterizzati tramite **`resvg`** per le esportazioni, senza alcuna dipendenza cloud.

7. **Esportazione 100% Rust Embedded (Zero dipendenze esterne)**: Esportazione diretta del documento in **PDF**, **Word (`.docx`)**, **LibreOffice (`.odt`)**, **LaTeX (`.tex`)**, **EPUB** e **HTML5** senza richiedere software esterno installato sul sistema.

8. **Affidabilità senza compromessi e Crash Recovery**: Progettato secondo il principio *zero data loss*:
   - **Journaling *Save-while-typing* (`< 300ms`)**: salvataggio continuo su journal WAL locale con ripristino automatico al riavvio successivo.
   - **Salvataggio Atomico (*Write-Temp -> Fsync -> Atomic Rename*)**: scrittura su file temporaneo, sincronizzazione fisica (`fsync`) e rinomina atomica.

9. **Internazionalizzazione (i18n)**: Supporto multilingua integrato in Rust per **italiano**, **inglese** e **spagnolo**.
