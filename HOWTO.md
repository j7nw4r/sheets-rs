# How To Use Sheets

## Open a file

```
sheets data.csv
```

TSV files are detected automatically by extension:

```
sheets data.tsv
```

Pipe data from another command:

```
cat data.csv | sheets
```

Start with an empty spreadsheet:

```
sheets
```

## Navigate the grid

Move with arrow keys or vim-style `h` `j` `k` `l`. Prefix a number to repeat: `5j` moves down 5 rows.

Jump to the top of the file with `gg` and the bottom with `G`.

Jump to the first column with `0`, last column with `$`, or the first non-empty column with `^`.

Scroll half a page with `Ctrl-D` (down) and `Ctrl-U` (up).

Go directly to a cell by typing `:` then a cell reference like `B12` and pressing Enter.

## Edit a cell

Press `i` to edit the current cell. The existing value is loaded and the cursor is placed at the end.

Press `c` to clear the cell and start typing from scratch.

Press `a` or `A` to append to the end of the current value.

Press `I` to edit with the cursor at the beginning.

When editing:

- Type normally to insert text.
- `Backspace` deletes the character before the cursor.
- `Ctrl-A` / `Ctrl-E` jump to the start / end of the value.
- `Ctrl-K` deletes from the cursor to the end.
- `Ctrl-U` deletes from the cursor to the beginning.
- `Ctrl-W` deletes the previous word.

Finish editing with:

- `Esc` -- save and stay on the current cell.
- `Enter` -- save and move down.
- `Tab` -- save and move right.

## Insert and delete rows

Press `o` to insert a blank row below the current row and begin editing.

Press `O` to insert a blank row above.

Press `dd` to delete the current row. The row is saved to the clipboard and can be pasted elsewhere.

## Use formulas

Type a value starting with `=` to create a formula:

| Formula | Result |
|---------|--------|
| `=A1+B1` | Sum of two cells |
| `=A1*2` | Multiply a cell by 2 |
| `=SUM(A1:A10)` | Sum a range |
| `=AVG(B1:B5)` | Average of a range |
| `=MIN(A1:A3)` | Minimum value |
| `=MAX(A1:A3)` | Maximum value |
| `=COUNT(A1:A10)` | Count of numeric values |

Formulas update automatically when referenced cells change. Circular references are detected and display an error.

## Select a range

Press `v` to start a visual selection. Move with `h` `j` `k` `l` to extend the selection rectangle.

Press `V` to select entire rows instead.

With a selection active:

- `y` copies the selection.
- `x` or `d` cuts the selection.
- `Esc` cancels the selection.

## Copy, cut, and paste

Copy the current cell by selecting it with `v` then pressing `y`.

Copy the current row with `yy`.

Cut the current cell with `x`.

Paste with `p`. The clipboard contents are placed starting at the current cell.

Formulas are automatically adjusted when pasted to a different position. For example, `=A1+A2` pasted one column to the right becomes `=B1+B2`.

## Use named registers

Prefix a copy or paste command with `"` followed by a letter to use a named register.

For example, `"a` then `yy` copies the current row into register `a`. Later, `"a` then `p` pastes from register `a`. This lets you hold multiple clipboard values at once.

## Undo and redo

Press `u` to undo the last change.

Press `Ctrl-R` to redo.

## Repeat the last edit

Press `.` to repeat the last insert-mode edit on the current cell. This is useful for applying the same value to multiple cells: edit one cell, move to the next, press `.`.

## Search

Press `/` and type a search term to search forward. Press `?` to search backward.

Press `n` to jump to the next match. Press `N` to jump to the previous match. Search wraps around the entire grid.

## Set bookmarks

Press `m` followed by a letter to bookmark the current cell. For example, `ma` sets mark `a`.

Press `'` followed by the letter to jump back. `'a` returns to the bookmarked cell.

## Save and quit

| Command | Action |
|---------|--------|
| `:w` | Save to the current file |
| `:w path.csv` | Save to a specific file |
| `:q` | Quit |
| `:wq` | Save and quit |
| `q` | Quit (in normal mode) |

## Use from the command line

Query a cell value without opening the editor:

```
sheets data.csv B9
```

Set a cell value and save:

```
sheets data.csv B7=10
```

Query multiple cells:

```
sheets data.csv A1 B2 C3
```
