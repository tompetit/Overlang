# Fixtures

Each `<name>.ov` has one of:

- `<name>.expect`: exact stdout of `ov run main`.
- `<name>.expect.err`: the trap `kind` expected on stderr (exit code 3), with
  `<name>.expect` holding any stdout before the trap.
- `<name>.expect.diag`: every diagnostic `code`, errors and warnings, that
  `ov check` emits for the file, one per line in the order it emits them.
- `<name>.expect.json`: optionally, next to `.expect.diag`, the exact error
  diagnostics as JSON lines, as `ov check` prints them.
- `<name>.stdin`: optional stdin.
