# Smoke Test Completo Specifica GFM `0.29-gfm`

Questo documento verifica visivamente in **Accent** (`./target/debug/accent-ui tests/fixtures/gfm_spec_smoke.md`) ciascuna sezione della specifica ufficiale [GitHub Flavored Markdown `0.29-gfm`](https://github.github.com/gfm/) (`§4.1`–`§4.10`, `§5.1`–`§5.4`, `§6.1`–`§6.14`), inclusi i principali casi limite per:
- **Editor CodeMirror 6** (*Modifica Sorgente* vs *Anteprima in-line •* con cursore fuori e dentro gli span);
- **Pannello Anteprima** (*Split destra*, *Split sotto*, *Solo lettura*) ed **Esportazione HTML/PDF/DOCX/ODT/LaTeX/EPUB**.

---

## Capitolo 4 — Leaf Blocks (`§4.1`–`§4.10`)

### §4.1 Thematic breaks

Tre varianti canoniche (`---`, `***`, `___`) e variante spaziata (` - - - `):

---

***

___

 - - - 

### §4.2 ATX headings

# Intestazione ATX Livello 1
## Intestazione ATX Livello 2
### Intestazione ATX Livello 3 con sequenza di chiusura ###
#### Intestazione ATX Livello 4
##### Intestazione ATX Livello 5
###### Intestazione ATX Livello 6

Edge case — intestazione ATX vuota (valida in GFM §4.2 Ex. 36) seguita da falso heading con 7 `#`:

###

####### Sette cancelletti: deve restare un paragrafo normale, non H7.

### §4.3 Setext headings

Intestazione Setext di Livello 1
================================

Intestazione Setext di Livello 2
su due righe consecutive
------------------------

### §4.4 Indented code blocks

Blocco di codice indentato di 4 spazi (le righe seguenti con `#` e `-` **non** devono diventare heading o liste):

    # Non è una intestazione ATX ma codice indentato
    - [x] Non è una task list ma codice indentato
    fn somma(a: i32, b: i32) -> i32 {
        a + b
    }

### §4.5 Fenced code blocks

Blocco fenced con tripli backtick e info string `rust`:

```rust
pub fn verifica_gfm(input: &str) -> bool {
    let valido = !input.is_empty();
    println!("Input GFM ({} byte): {valido}", input.len());
    valido
}
```

Blocco fenced con tilde (`~~~json`):

~~~json
{
  "spec": "0.29-gfm",
  "extensions": ["table", "tasklist", "strikethrough", "autolink", "tagfilter"],
  "compliant": true
}
~~~

Edge case — blocco a 4 backtick contenente tripli backtick interni e finta chiusura con testo residuo (§4.5 Ex. 97):

````markdown
Dentro un fence a 4 backtick posso mostrare un esempio fenced:
```python
print("fenced interno")
```
````

### §4.6 HTML blocks

<div class="nota-html-block" data-section="4.6">
  <strong>Blocco HTML Tipo 6 (§4.6):</strong> questo contenuto è racchiuso in un elemento <code>&lt;div&gt;</code> di blocco.
</div>

<details>
<summary>Blocco interattivo HTML5 <code>&lt;details&gt;</code> / <code>&lt;summary&gt;</code> (clicca per espandere in Preview)</summary>
Contenuto interno del blocco HTML espandibile.
</details>

<!--
  Blocco HTML Tipo 2 (§4.6): commento HTML multi-riga.
  Non deve produrre testo visibile nel pannello di anteprima né nell'HTML esportato.
-->

### §4.7 Link reference definitions

Le definizioni seguenti non devono apparire come testo renderizzato in Preview/Export, ma risolvere i link nella sezione `§6.6`:

[gfm-spec]: https://github.github.com/gfm/ "GitHub Flavored Markdown Spec 0.29-gfm"
[rust-home]: <https://www.rust-lang.org> 'Sito Ufficiale Rust'
[definizione-multiriga]:
  https://commonmark.org/help/
  (Guida Rapida CommonMark su riga successiva)

### §4.8 Paragraphs

Primo paragrafo standard su più righe consecutive: l'andata a capo semplice
viene trattata come continuazione fluida dello stesso blocco di prosa senza
creare un nuovo paragrafo.

Secondo paragrafo separato dal precedente, immediatamente interrotto da una citazione senza riga vuota:
> Citazione che interrompe il paragrafo senza linea vuota intermedia.

### §4.9 Blank lines

Due paragrafi separati da righe vuote multiple (tutte collassate a un singolo confine di blocco nell'output HTML):



Paragrafo dopo tre righe vuote consecutive.

### §4.10 Tables (Estensione GFM #1)

Tabella principale con **allineamenti** (`:---` sinistra, `:---:` centro, `---:` destra, `---` default), **formattazione inline nelle celle** e **pipe escapati `\|`**:

| Allineamento Sinistra | Allineamento Centro | Allineamento Destra | Edge Case Inline & Pipe Escapati |
| :--- | :---: | ---: | --- |
| Testo semplice &copy; | **Grassetto** | `1.250,00 €` | Pipe escapato: `a \| b` e fuori codice: A \| B |
| *Corsivo* e ~~barrato **bold**~~ | `codice_inline()` | `42,5%` | Link in cella: [Spec GFM](https://github.github.com/gfm/) |
| Combinazione **bold *italic*** | Formula $E = mc^2$ 🦀 | `999.999` &#8594; | Code span con pipe: `a | b` e `` `x | y` `` |

Edge case GFM §4.10 — tabella compatta **senza pipe esterni** e con numero variabile di celle nelle righe dati (celle mancanti completate vuote, celle in eccesso ignorate):

Colonna A | Colonna B | Colonna C
--- | :---: | ---:
Solo A
 Uno | Due | Tre | Quattro (cella in eccesso ignorata da GFM)

---

## Capitolo 5 — Container Blocks (`§5.1`–`§5.4`)

### §5.1 Block quotes

Citazione singola, citazione **annidata** (`> >`) e *lazy continuation line* (riga finale senza `>`):

> Prima riga della citazione di primo livello.
> Continuazione con *lazy line* sulla riga seguente senza simbolo di maggiore:
questa riga appartiene comunque alla stessa citazione secondo CommonMark/GFM §5.1.
>
> > Citazione annidata di secondo livello con **grassetto** e `codice`.
> >
> > > Terzo livello di annidamento.

### §5.2 List items

Elementi di lista con paragrafi multipli e blocco di codice indentato interno (regola d'indentazione §5.2):

- Primo elemento con due paragrafi interni.

  Secondo paragrafo allineato dentro il primo elemento della lista.

      // Codice indentato dentro il primo list item (indentazione = 2 + 4 spazi)
      let indent_ok = true;

- Secondo elemento con citazione interna:
  > Citazione annidata dentro l'elemento d'elenco.

### §5.3 Task list items (Estensione GFM #2)

Task list canonica (`[ ]`, `[x]`, `[X]` maiuscola) e sotto-task annidati:

- [ ] Attività da completare (verifica click su checkbox in Editor e in Preview)
- [x] Attività completata con `x` minuscola e **formattazione inline**
- [X] Attività completata con `X` maiuscola (valida in GFM §5.3)
  - [ ] Sotto-attività annidata non completata
  - [x] Sotto-attività annidata completata

Edge case GFM §5.3 — task list su **elenco numerato** (`1. [x]`) e dentro **blockquote** (`> - [ ]`):

1. [x] Primo passo numerato completato
2. [ ] Secondo passo numerato da eseguire

> - [x] Task completato dentro una citazione
> - [ ] Task aperto dentro una citazione

### §5.4 Lists

Separazione automatica delle liste al cambio di marcatore (`-` vs `*`, `.` vs `)`) e attributo `start` diverso da `1`:

- Lista puntata con trattino `-` (tight list)
- Secondo elemento `-`

* Nuova lista puntata separata perché usa asterisco `*`

3. Lista ordinata che parte da **3** (`<ol start="3">`)
4. Elemento successivo numerato **4**

1) Nuova lista ordinata con parentesi chiusa `)`

---

## Capitolo 6 — Inlines (`§6.1`–`§6.14`)

### §6.1 Backslash escapes

Tutti i seguenti marcatori sono preceduti da `\` e **non** devono attivare formattazione né in Live Preview né in Preview HTML:

\*non corsivo\* · \*\*non grassetto\*\* · \~\~non barrato\~\~ · \`non codice\` · \[non link\](https://example.com) · \!\[non immagine\](logo.png) · \# non titolo · \\ backslash singolo

Backslash prima di un carattere non di punteggiatura (resta visibile `\a`): \a\b\c.

### §6.2 Entity and numeric character references

- **Entità con nome HTML5:** `&copy;` = &copy; · `&reg;` = &reg; · `&mdash;` = &mdash; · `&euro;` = &euro; · `&nbsp;` = [spazio&nbsp;inseparabile] · `&Lambda;` = &Lambda;
- **Riferimenti numerici decimali:** `&#35;` = &#35; · `&#169;` = &#169; · `&#8594;` = &#8594;
- **Riferimenti numerici esadecimali:** `&#x22;` = &#x22; · `&#x03BB;` = &#x03BB; · `&#x1F980;` = &#x1F980;

### §6.3 Code spans

- Singolo backtick: `let x = 42;`
- Doppio backtick per includere un backtick letterale (§6.3 Ex. 339): ``usa `codice` qui``
- Doppio backtick con spazi di guardia rimossi (§6.3 Ex. 330): `` `racchiuso da backtick` ``
- Non-escape dentro code span: `` `\*\*letterale\*\*` ``

### §6.4 Emphasis and strong emphasis

- Base: *corsivo con asterischi*, _corsivo con underscore_, **grassetto con asterischi**, __grassetto con underscore__.
- Combinati e annidati (edge case collisione intervalli): ***grassetto e corsivo insieme***, **grassetto con *corsivo interno* e `codice`**, *corsivo con **grassetto interno***.
- Intraword underscores (regola CommonMark/GFM §6.4: **mai** corsivo/grassetto): `campo_identificativo_documento__c` e campo_data_registrazione__c_finale_attivo.

### §6.5 Strikethrough (Estensione GFM #3)

- Doppia tilde standard: ~~testo barrato con doppia tilde~~.
- Singola tilde (valida in GFM `0.29-gfm` §6.5 Ex. 492, non pedice): ~testo barrato con singola tilde~.
- Tripla tilde (in GFM §6.5 **non** deve barrare il testo): ~~~tre tilde restano letterali~~~.
- Annidamento con enfasi: ~~barrato contenente **grassetto** e *corsivo*~~.

### §6.6 Links

- **Link inline con titolo:** [Specifica Ufficiale GFM](https://github.github.com/gfm/ "Titolo del collegamento")
- **Edge case parentesi bilanciate nell'URL (§6.6 Ex. 497):** [Voce Rust (linguaggio)](https://en.wikipedia.org/wiki/Rust_(programming_language))
- **URL racchiuso tra `<...>` con spazi:** [Percorso con spazi](<https://example.com/percorso con spazi/doc.md>)
- **Link con formattazione nel testo:** [**Link in grassetto** con `codice`](https://www.rust-lang.org)
- **Reference links (definiti in §4.7):** completo [Specifica GFM][gfm-spec], collassato [rust-home][], scorciatoia [definizione-multiriga].

### §6.7 Images

Immagine di blocco con **path relativo locale reale** nel repository (`../../img/brand/accent_logo.png`):

![Logo Ufficiale Accent PNG](../../img/brand/accent_logo.png "Logo Accent locale")

Immagine SVG locale con path relativo (`../../img/brand/accent_logo.svg`):

![Logo Ufficiale Accent SVG](../../img/brand/accent_logo.svg)

Edge case — immagine **inline** immersa dentro un paragrafo di testo: ecco l'icona ![mini-logo](../../img/brand/accent_logo.png) integrata nella stessa riga di prosa.

### §6.8 Autolinks (CommonMark standard)

- URL tra parentesi angolari: <https://github.github.com/gfm/>
- Indirizzo email tra parentesi angolari: <sviluppo@accent-editor.example>
- Schema `mailto:` esplicito: <mailto:info@accent-editor.example>

### §6.9 Autolinks (Estensione GFM #4 — Extended Autolinks)

- Dominio `www.` senza schema (deve puntare a `http://`): www.commonmark.org/help
- URL `https://` nudo nel testo: Visita https://github.github.com/gfm/ per i dettagli.
- **Edge case punteggiatura finale esclusa (§6.9 Ex. 622–626):** Controlla (https://github.com/rust-lang/rust), oppure https://github.github.com/gfm/?!.,:*_~ fine frase.
- **Email nuda ed `xmpp:`:** contatta supporto.tecnico@accent-editor.example oppure xmpp:nodo@accent-editor.example/risorsa.

### §6.10 Raw HTML (Inline)

Scorciatoia con tag `<kbd>`: premi <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>P</kbd> per aprire l'anteprima; formula chimica H<sub>2</sub>O e potenza x<sup>2</sup>; <!-- commento inline invisibile --> testo dopo il commento.

### §6.11 Disallowed Raw HTML (Estensione GFM #5)

In GFM `0.29-gfm` `§6.11`, i seguenti **9 tag** devono avere il `<` iniziale convertito in `&lt;` nell'HTML generato (senza essere interpretati dal browser), mentre gli altri tag HTML (`<kbd>`, `<span>`, `<div>`) restano attivi:

1. `<title>`: <title>Sonda GFM 6.11 title</title>
2. `<textarea>`: <textarea rows="1" cols="34">Sonda GFM 6.11 textarea non filtrata</textarea>
3. `<style>` (se non filtrato, disegna un bordo rosso tratteggiato sulla sonda qui sotto):
<style>.gfm-611-style-probe { outline: 2px dashed #d94646; padding: 2px 6px; }</style>
<span class="gfm-611-style-probe">Sonda visiva §6.11: se vedi un bordo rosso tratteggiato, `&lt;style&gt;` NON è stato filtrato!</span>
4. `<xmp>`: <xmp>Sonda GFM 6.11 <xmp></xmp>
5. `<iframe>`: <iframe src="about:blank" width="90" height="24" title="Sonda 6.11 iframe"></iframe>
6. `<noembed>`: <noembed>Sonda GFM 6.11 noembed</noembed>
7. `<noframes>`: <noframes>Sonda GFM 6.11 noframes</noframes>
8. `<script>`: <script>/* Sonda GFM 6.11 script */</script>
9. `<plaintext>`: testato in coda assoluta al documento (*Appendice §6.11-bis*) poiché in HTML5 `<plaintext>` non ammette tag di chiusura e assorbe tutto il DOM successivo.

### §6.12 Hard line breaks

Prima riga terminata con backslash finale (`\`): \
Seconda riga terminata con due spazi finali (`␣␣`):  
Terza riga dopo due *hard line breaks* consecutivi (`<br />`).

### §6.13 Soft line breaks

Questa è la prima riga di un paragrafo con *soft line break*
e questa è la seconda riga separata da un singolo `\n` (in HTML diventa uno spazio singolo, senza `<br />`).

### §6.14 Textual content

- **Testo multilingue Unicode:** Italiano (*àèéìòù*), Español (*¿Pingüino?*), Ελληνικά (*Αλφάβητο*), 日本語 (*日本語テキスト*).
- **Sequenze Emoji & ZWJ:** 🦀 Rust · 👨‍💻 Sviluppatore · 🚀 Razzo · `:rocket:` shortcode :rocket:.
- **Edge case simboli di valuta `$` (verifica falsi positivi KaTeX in GFM):** Il piano base costa $100 al mese mentre il piano enterprise costa $250 all'anno.

---

#### Appendice §6.11-bis — Sonda finale `<plaintext>`

Se il filtro GFM `§6.11` è attivo, la riga seguente appare come testo `&lt;plaintext&gt;...` normale; se non è filtrato, il parser HTML5 entra in modalità raw plaintext fino a fine documento:

<plaintext>Sonda finale GFM §6.11 <plaintext> — fine del documento smoke test.
