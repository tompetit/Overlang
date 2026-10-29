# Fixtures

Each `<name>.ov` has one of:

- `<name>.expect`: exact stdout of `ov run main`.
- `<name>.expect.err`: the trap `kind` expected on stderr (exit code 3), with
  `<name>.expect` holding any stdout before the trap.
- `<name>.expect.diag`: one diagnostic `code` per line that `ov check` must
  emit for the file, and nothing with severity error beyond them.
- `<name>.stdin`: optional stdin.
