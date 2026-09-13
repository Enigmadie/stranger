# Stranger

Stranger is a terminal file manager built around Miller columns, Vim-style navigation, file previews, search, marks, bookmarks, and safe file operations.

It opens in the current working directory.

## Install

Stranger currently requires a Rust toolchain and a terminal supported by Crossterm.

```sh
cargo build --release --locked
./target/release/stranger
```

To install the binary into `/usr/local/bin`:

```sh
make install
```

The Makefile uses `sudo` for installation. Override `PREFIX` to change the install prefix.

## Usage

```text
Usage: stranger [OPTIONS]

Options:
      --editor <EDITOR>
      --config-path <CONFIG_PATH>
  -h, --help
```

Editor commands may contain arguments and quoted values. Stranger appends the selected file path as a separate argument:

```sh
stranger --editor 'nvim -f'
stranger --config-path ~/.config/stranger/work.toml
```

## Configuration

The config path is selected in this order:

1. `--config-path PATH`
2. `$XDG_CONFIG_HOME/stranger/config.toml`
3. macOS: `~/Library/Application Support/stranger/config.toml`
4. Other Unix systems: `~/.config/stranger/config.toml`
5. `./config.toml` when no home or platform config directory is available

Example:

```toml
[common]
editor = "nvim"

[bookmarks]
work = "/home/user/work"
config = "/home/user/.config/example.toml"
```

The editor is selected from `--editor`, the config file, `$VISUAL`, `$EDITOR`, then `nvim`. Bookmark aliases retain their configured order and must be unique.

Bookmark changes are saved atomically. Existing config permissions are retained, config-file symlinks are followed, and malformed config files are never overwritten.

## Key Bindings

### Navigation

| Key | Action |
|---|---|
| `j`, `Down` | Move down |
| `k`, `Up` | Move up |
| `h`, `Left` | Open parent directory |
| `l`, `Right` | Open directory or edit file |
| `Ctrl-h` | Toggle hidden files |
| `Ctrl-d`, `Ctrl-u` | Move down or up 25 entries |
| `Space` | Toggle mark and move down |
| `v` | Start visual range selection |
| `Esc` | Clear marks and leave search |
| `q` | Quit |

### Files And Search

| Key | Action |
|---|---|
| `a` | Add a file; end the name with `/` to add a directory |
| `r` | Rename the selected item |
| `y` | Copy selected or marked items |
| `p` | Paste copied or cut items |
| `/` | Search by filename prefix |
| `n`, `N` | Next or previous search match |

Input prompts support normal text editing, cursor movement, deletion, selection, undo, and redo through `tui-textarea`. Press `Enter` to commit or `Esc` to cancel. Input is limited to 255 UTF-8 bytes.

### Visual Mode

| Key | Action |
|---|---|
| `j`, `Down` | Extend or contract the selected range downward |
| `k`, `Up` | Extend or contract the selected range upward |
| `v`, `Esc`, `Ctrl-[` | Return to normal mode |

### Bookmarks

Press `b` to open the bookmark menu.

| Key | Action |
|---|---|
| `b` | Open bookmark list |
| `a` | Add the selected item with a new alias |
| `q`, `Esc` | Close the menu |

In the bookmark list, use `j`/`k` or the arrow keys to navigate, `l` or `Enter` to open, `d` to delete, and `q` or `Esc` to return.

### Cut And Delete

Press `d` to open the file-operation menu.

| Key | Action |
|---|---|
| `d` | Cut selected or marked items for a later paste |
| `D` | Move selected or marked items to the OS trash |
| `x` | Permanently delete selected or marked items |
| `q`, `Esc` | Cancel |

### Exit Menu

Press `z` or `Z` to open the exit menu.

| Key | Action |
|---|---|
| `z`, `Z` | Replace Stranger with an interactive shell in the displayed directory |
| `q`, `Q` | Exit in the original directory |
| `Esc` | Cancel |

The current-directory action replaces Stranger with `$SHELL -l -i`; it cannot change the working directory of its parent shell.

## Safety

- Add and rename accept one file name, not an absolute or nested path.
- Add and rename never overwrite an existing item.
- Paste resolves name collisions with a unique destination name.
- A directory cannot be copied or moved into itself or a descendant.
- Same-filesystem moves use an atomic no-replace rename. Cross-device moves delete the source only after a successful copy.
- Items that fail during a partial paste remain in the internal clipboard for retry.
- Symlinks are copied, moved, and deleted as links rather than followed.
- Special files such as FIFOs and devices are not copied or previewed.
- Trash failure never falls back to permanent deletion.
- Permanent deletion is recursive and irreversible. It requires the `d`, then `x` key sequence.
- The current directory and its ancestors cannot be deleted.
- Terminal state is restored after normal exit, setup errors, panics, and external editor failures.

## Platform Support

- Linux is tested in CI.
- macOS is supported but not currently tested in CI. Moving files to Trash may trigger a file-access permission prompt.
- Other Unix systems may work but are not tested.
- Windows is not currently supported.

## Development

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```
