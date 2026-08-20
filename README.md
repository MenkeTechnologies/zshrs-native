# zshrs-native

The [zshrs](https://github.com/MenkeTechnologies/zshrs) shell with three sibling
runtimes compiled into the same binary and dispatched as shell builtins:

| Command  | Runtime                                                      | What it replaces          |
|----------|--------------------------------------------------------------|---------------------------|
| `git`    | [zvcs](https://github.com/MenkeTechnologies/zvcs)            | git (via vendored gitoxide) |
| `arb`    | [arblang](https://github.com/MenkeTechnologies/arb)          | pipeline TUI / query language |
| `stryke` | [strykelang](https://github.com/MenkeTechnologies/strykelang) | Perl-superset scripting   |

The binary this package builds is named `zshrs`. It is the thin `zshrs` shell
plus these three — a drop-in replacement, not a different shell.

## No fork

zshrs already runs 23 coreutils and every shell builtin in-process. This
package extends that set: `git status`, `arb`, and `stryke` run inside the
shell process too. No `fork`, no `exec`, no dynamic loader, no `PATH` lookup.

`command git` still reaches the `git` on `PATH`, so the escape hatch to any
other implementation is unchanged.

## Build

```sh
git clone --recursive https://github.com/MenkeTechnologies/zshrs-native
cd zshrs-native
cargo build --release          # binary: target/release/zshrs
cargo install --path .         # installs as `zshrs`
```

`--recursive` is required — the four runtimes are `vendor/` submodules, and
zvcs carries its own vendored gitoxide under `src/ported`.

To move the pins forward to each runtime's current `main`:

```sh
git submodule update --remote
```

## Layout

```
src/main.rs        registers the runtimes, then runs the shell
vendor/zshrs       the shell
vendor/strykelang  stryke
vendor/arb         arb
vendor/zvcs        git
```

`src/main.rs` includes the shell's REPL (`vendor/zshrs/bins/zshrs.rs`) with
`#[path]`, because Cargo binary targets cannot be imported by another package.

## Constraints

Both of these are load-bearing; changing them breaks the build or the shell.

- **One fusevm.** fusevm exports 67 `#[no_mangle]` symbols across its
  `aot`/`jit`/`ffi` modules, and Cargo treats `0.x` minors as
  semver-incompatible. Two copies in the dependency graph is a duplicate-symbol
  link failure. Every runtime here must request the same fusevm minor. Note
  that `cargo check` does not link, so it will not catch a violation —
  `cargo build` will.
- **`panic = "unwind"`.** The zshrs package's own release profile uses
  `panic = "abort"`. Profile settings come from the root package, so this one
  governs the whole graph, and it deliberately differs: three large runtimes now
  share the shell's address space, and an abort in any of them would take an
  interactive shell down. Unwinding lets the dispatch boundary catch a panic and
  return an exit status.

Only one `#[no_mangle] fusevm_aot_register_builtins` may exist per binary. The
zshrs dependency takes `default-features = false` to drop its definition, so
strykelang's is the one that links.
