# sheets-rs

A vim-inspired terminal spreadsheet editor, written in Rust. Based on [maaslalani/sheets](https://github.com/maaslalani/sheets).

## Install

```
cargo install --path .
```

## Usage

```
sheets [file] [cell...]
```

Open a CSV file interactively:

```
sheets data.csv
```

Query a cell without opening the TUI:

```
sheets data.csv B9
```

Set a cell value:

```
sheets data.csv B7=10
```

Read from stdin:

```
cat data.csv | sheets
```

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| h/j/k/l | Move left/down/up/right |
| gg | Go to top |
| G | Go to bottom |
| 0 / $ | First / last column |
| ^ | First non-blank column |
| H / M / L | Window top / middle / bottom |
| Ctrl-D / Ctrl-U | Half-page down / up |
| Ctrl-O / Ctrl-I | Jump back / forward |

### Editing

| Key | Action |
|-----|--------|
| i | Edit cell |
| I | Edit cell (cursor at start) |
| a / A | Append to cell |
| c | Clear and edit cell |
| o / O | Insert row below / above |
| u | Undo |
| Ctrl-R | Redo |
| . | Repeat last change |

### Selection and Clipboard

| Key | Action |
|-----|--------|
| v | Visual select |
| V | Visual select (row) |
| y / yy | Yank cell / row |
| d / dd | Delete cell / row |
| x | Cut |
| p | Paste |
| "a | Use register a |
| m{a-z} | Set mark |
| '{a-z} | Jump to mark |

### Search and Commands

| Key | Action |
|-----|--------|
| / | Search forward |
| ? | Search backward |
| n / N | Next / previous match |
| : | Command mode |

### Commands

| Command | Action |
|---------|--------|
| :w [path] | Save |
| :q | Quit |
| :wq | Save and quit |
| :e path | Open file |
| :goto A1 | Go to cell |

## Formulas

Cells starting with `=` are evaluated as formulas.

```
=A1+B1
=SUM(A1:A10)
=AVG(B1:B5)
=MIN(A1:A3)
=MAX(A1:A3)
=COUNT(A1:A10)
```

Supported operators: `+`, `-`, `*`, `/`, parentheses.

## License

MIT
