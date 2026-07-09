# bestls

A user-friendly alternative to `ls` with human-readable output.

## Features

- **Human-readable file sizes** — displays sizes in B, KB, MB, GB, or TB (1 decimal place for values ≥ 1 KB)
- **Relative timestamps** — shows modification times as relative phrases (`just now`, `3 days ago`, etc.)
- **Colored table output** — columns are color-coded for easy scanning
- **JSON mode** — `--json` flag for machine-readable output

## Usage

```
bestls [PATH] [--json]
```

If no `PATH` is given, the current directory is listed.

## Output columns

| Column    | Description                                          |
|-----------|------------------------------------------------------|
| Name      | File or directory name                               |
| Type      | `File` or `Dir`                                      |
| Size      | Human-readable size (B, KB, MB, GB, TB)              |
| Modified  | Relative modification time                           |

### Size formatting

- **Bytes**: displayed as whole numbers (e.g. `42 B`)
- **≥ 1 KB**: displayed with 1 decimal place (e.g. `1.5 KB`, `3.2 MB`)

### Time formatting

- < 60 seconds: `just now`
- < 60 minutes: `X minute(s) ago`
- < 24 hours: `X hour(s) ago`
- < 30 days: `X day(s) ago`
- < 365 days: `X month(s) ago`
- ≥ 365 days: `X year(s) ago`

## Building

```sh
cargo build --release
```

## Example

```
$ bestls
╭───────┬──────┬─────────┬──────────────╮
│ Name  │ Type │ Size    │ Modified     │
├───────┼──────┼─────────┼──────────────┤
│ src   │ Dir  │ 4.1 KB  │ 2 hours ago  │
│ Cargo │ File │ 315 B   │ just now     │
╰───────┴──────┴─────────┴──────────────╯
```
