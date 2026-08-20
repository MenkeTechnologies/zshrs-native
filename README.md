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

## World first: a version control system compiled into the shell

Every Unix shell in the fifty-five years since the Bourne shell runs `git` the
same way: fork, exec, wait. `git` is a foreign binary that the shell knows
nothing about beyond its exit status. This package makes it a builtin — the
whole of git, executing inside the shell's own process, no `fork`, no `exec`,
no `PATH` lookup, no dynamic loader.

The implementation is real git, not a status helper: zvcs serves every
porcelain verb natively through its vendored gitoxide fork, and there is no
fallback to an external git binary anywhere in it.

Nothing else does this. The near misses, and why they are not the same thing:

| System | What it does | Why it is not this |
|---|---|---|
| BusyBox / toybox | Shell and utilities in one binary, no fork between them | No git applet — the applet set is coreutils-class |
| Nushell `nu_plugin_gstat` | Git status as structured data | Git *status* only, and plugins are separate child processes: Nu "launches them as needed and communicates with them over stdin and stdout or local sockets" |
| `git-shell` | Restricted login shell for SSH git access | Permits only server-side git verbs, and execs real git to serve them — the confusable name, the opposite architecture |
| bash `enable -f`, zsh `zmodload` | Load native code into the shell | Nobody has shipped a git through them; both bind to the shell's private build-tree headers with no stable ABI |
| Emacs + magit, posh-git, fish/zsh git plugins | Rich git integration in a shell or editor | Every one shells out to the git binary |

The same treatment extends to `arb` and `stryke`, so the binary hosts a VCS, a
pipeline TUI language, and a scripting language with no process boundary
between any of them and the shell.

`command git` still reaches whatever `git` is on `PATH`, so the escape hatch to
another implementation is unchanged.

### Status

Honest accounting of what is wired today:

| | State |
|---|---|
| All four runtimes link into one binary | Working — `zshrs -c` runs, one fusevm, one libsqlite3-sys |
| `git` / `arb` / `stryke` as builtins | **Not yet dispatched.** Each needs an argv entry point (`run_argv(&[String]) -> i32`); none of the three exposes one today — their CLI layers live in `main.rs`, which no other crate can call |
| `@ <code>` → stryke | **Handler registered, not reached.** `zsh::try_stryke_dispatch` is consulted only from `bins/zshrs.rs:process_line`, and interactive input never returns to the bin — it hands off at `bins/zshrs.rs:2642` to `zsh::ported::init::zsh_main()` in the library |

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
