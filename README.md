```
███╗   ██╗ █████╗ ████████╗██╗██╗   ██╗███████╗
████╗  ██║██╔══██╗╚══██╔══╝██║██║   ██║██╔════╝
██╔██╗ ██║███████║   ██║   ██║██║   ██║█████╗  
██║╚██╗██║██╔══██║   ██║   ██║╚██╗ ██╔╝██╔══╝  
██║ ╚████║██║  ██║   ██║   ██║ ╚████╔╝ ███████╗
╚═╝  ╚═══╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═══╝  ╚══════╝
```

![Rust](https://img.shields.io/badge/Rust-2021-05d9e8?style=flat-square)
![license](https://img.shields.io/badge/license-MIT-ff2a6d?style=flat-square)
![status](https://img.shields.io/badge/status-active%20%C2%B7%20in%20development-9b5de5?style=flat-square)
![fork](https://img.shields.io/badge/fork%20count-0-00e5ff?style=flat-square)

### `[THE SHELL THAT IS ALSO GIT]`

> *"The last binary you exec is the shell."*

**zshrs-native** is [`zshrs`](https://github.com/MenkeTechnologies/zshrs) — the
first compiled Unix shell — with three sibling runtimes linked into the same
address space and dispatched as shell builtins: [`zvcs`](https://github.com/MenkeTechnologies/zvcs)
serves `git`, [`arblang`](https://github.com/MenkeTechnologies/arb) serves `arb`
and its fzf-compatible finder, [`strykelang`](https://github.com/MenkeTechnologies/strykelang)
serves `stryke` and the `@` prefix. Two of those are world firsts: **no shell has
ever compiled a version control system into itself**, and **no shell has ever
compiled in an fzf-compatible finder**. Everything here runs with no `fork`, no
`exec`, no `PATH` lookup and no dynamic loader — the same treatment zshrs already
gives `cat`, `sort` and `find`. The binary it builds is named `zshrs`: a drop-in
superset of the thin shell, not a different one.

### [`zshrs`](https://github.com/MenkeTechnologies/zshrs) &middot; [`zvcs`](https://github.com/MenkeTechnologies/zvcs) &middot; [`arb`](https://github.com/MenkeTechnologies/arb) &middot; [`strykelang`](https://github.com/MenkeTechnologies/strykelang) &middot; [`fusevm`](https://github.com/MenkeTechnologies/fusevm)

---

## Table of Contents

- [\[0x00\] Overview](#0x00-overview)
- [\[0x01\] Install](#0x01-install)
- [\[0x02\] World First: A VCS In The Shell](#0x02-world-first-a-vcs-in-the-shell)
- [\[0x03\] World First: An fzf Engine In The Shell](#0x03-world-first-an-fzf-engine-in-the-shell)
- [\[0x04\] No-Fork Dispatch](#0x04-no-fork-dispatch)
- [\[0x05\] Why A Separate Package](#0x05-why-a-separate-package)
- [\[0x06\] Link Constraints](#0x06-link-constraints)
- [\[0x07\] Status](#0x07-status)
- [\[0xFF\] License](#0xff-license)

---

## [0x00] OVERVIEW

| Command | Runtime | What it displaces |
|---------|---------|-------------------|
| `git` | [zvcs](https://github.com/MenkeTechnologies/zvcs) — vendored gitoxide, every porcelain verb native | `/usr/bin/git` |
| `arb` | [arblang](https://github.com/MenkeTechnologies/arb) — pipeline TUI, query engine, fzf finder | `fzf`, `jq`, `yq` |
| `stryke` · `st` · `s` | [strykelang](https://github.com/MenkeTechnologies/strykelang) — Perl-superset scripting; all three names it ships, each dispatching on its own `argv[0]` | `perl`, `awk` |
| `@ <code>` | strykelang, inline at the prompt | — |

```
┌─────────────────────────────────────────────────────────────┐
│                   ONE PROCESS, ONE BINARY                   │
├───────────────┬───────────────┬──────────────┬──────────────┤
│     zshrs     │     zvcs      │   arblang    │  strykelang  │
│  shell + ZLE  │      git      │  arb + fzf   │    stryke    │
├───────────────┴───────────────┴──────────────┴──────────────┤
│                    FUSEVM EXECUTION CORE                    │
│            one VM · one JIT · one JIT disk cache            │
└─────────────────────────────────────────────────────────────┘
```

---

## [0x01] INSTALL

```sh
git clone --recursive https://github.com/MenkeTechnologies/zshrs-native
cd zshrs-native
cargo build --release          # binary: target/release/zshrs
cargo install --path .         # installs as `zshrs`
```

`--recursive` is mandatory — the four runtimes are `vendor/` submodules, and
zvcs carries its own vendored gitoxide under `src/ported`.

```sh
git submodule update --remote  # move every pin to its runtime's current main
```

Or from the tap:

```sh
brew install MenkeTechnologies/menketech/zshrs-native      # the shell
brew install MenkeTechnologies/menketech/zshrs-native-all  # + zd, recorder, daemon
```

`zshrs-native-all` is the daily-driver package: the same four binaries
`zshrs-all` ships — `zshrs`, `zd`, `zshrs-recorder`, `zshrs-daemon` — with the
fat shell in place of the thin one. Each of these installs a binary named
`zshrs` (and `zd`), so they conflict with each other and with `zshrs` /
`zshrs-all` / `zshrs-daemon`; `brew uninstall zshrs` first if you have it.

`zd` and `zshrs-recorder` are bin targets of the zshrs package behind its
`zd` / `recorder` features. A path dependency is not a workspace member, so
neither `cargo build -p zshrs --features zshrs/zd` (refused: "cannot specify
features for packages outside of workspace") nor re-exporting the features here
can reach them — the release builds them through the submodule's own manifest,
at the cost of compiling the zshrs lib a second time under that feature set.
`zshrs-daemon` is a separate package and needs no features, so `-p` reaches it
from the root build.

The formula is generated by `.github/workflows/release.yml` on a `v*` tag: the
build matrix produces one tarball per target, the release job attaches them, and
the tap job writes `Formula/zshrs-native.rb` with the SHA256 of each. A target
that failed to build simply has no block in the formula rather than one with an
empty checksum — `v0.1.0` ships macOS arm64/x86_64 and Linux gnu arm64/x86_64,
and no musl: the musl link picks up glibc's `libtinfo.a`, whose fortified
`__fprintf_chk` / `__sprintf_chk` musl does not provide.

---

## [0x02] WORLD FIRST: A VCS IN THE SHELL

Every Unix shell in the fifty-five years since the Bourne shell runs `git` the
same way: fork, exec, wait. `git` is a foreign binary the shell knows nothing
about beyond an exit status. Here it is a builtin — the whole of git, in the
shell's own process. Not a status helper: zvcs serves every porcelain verb
natively and has no fallback to an external git anywhere in it.

| Prior art | What it does | Why it is not this |
|-----------|--------------|--------------------|
| BusyBox / toybox | Shell + utilities in one binary, no fork between them | No git applet — the set is coreutils-class |
| Nushell `nu_plugin_gstat` | Git status as structured data | Status only, and plugins are separate child processes: Nu "launches them as needed and communicates with them over stdin and stdout or local sockets" |
| `git-shell` | Restricted login shell for SSH git access | Permits only server-side verbs and execs real git to serve them — confusable name, opposite architecture |
| bash `enable -f`, zsh `zmodload` | Load native code into a shell | Nobody has shipped a git through either; both bind to private build-tree headers with no stable ABI |
| magit, posh-git, every shell git plugin | Rich git integration | All shell out to the git binary |

---

## [0x03] WORLD FIRST: AN FZF ENGINE IN THE SHELL

Every fzf integration a shell has ever had spawns the fzf binary — zsh's
`CTRL-T`/`CTRL-R` bindings, fzf-tab, fish's fzf.fish, PSFzf. The finder is a
foreign process the shell pipes into and reads back. arb's is linked in.

`arb --fzf` is a drop-in for the binary. It honors `FZF_DEFAULT_OPTS`,
`FZF_DEFAULT_OPTS_FILE`, and fzf's flag surface:

```
--preview  --preview-window  --bind      --expect     --nth      --with-nth
--delimiter --ansi           --tac       --tiebreak   --height   --layout
--reverse  --border          --color     --header     --header-lines
--info     --print-query     --exact     --no-sort    --filter   --cycle
--marker   --pointer         --ellipsis  --scroll-off --with-shell
```

So `ZPWR_FZF='arb --fzf'` keeps its prompt, layout, colors and key bindings, and
an existing config drops in untouched. Scoring is in `vendor/arb/src/tui.rs`
(`fuzzy_score`, `exact_score`, `score_line`); the flag, theme and preview layer
is `vendor/arb/src/fzf.rs`.

| Prior art | What it does | Why it is not this |
|-----------|--------------|--------------------|
| **Elvish** | The nearest miss by a distance — real in-process fuzzy filtering for command history (histlist) and directory jumping (location mode), "a mini-fzf" | Shell-internal UI modes, not a finder: cannot filter an arbitrary pipeline, honors none of fzf's CLI or env surface |
| Nushell `explore` | Interactive TUI pager over structured data | A viewer for nu values, not an fzf-compatible line filter |
| zsh fzf bindings, fzf-tab, fzf.fish, PSFzf | Deep fzf integration | Every one spawns the fzf binary |
| skim | The one fuzzy finder published as an embeddable Rust library rather than binary-only | The capability has been open to any Rust shell for years; none ships it in-process |

---

## [0x04] NO-FORK DISPATCH

| Operation | Every other shell | zshrs-native |
|-----------|-------------------|--------------|
| `git status` | fork + exec + ld.so + libc init | **Builtin** — zero fork |
| `… \| fzf` | fork + exec the fzf binary | **Builtin** — zero fork |
| `stryke -ne '…'` | fork + exec | **Builtin** — zero fork |
| `@ <code>` | not possible | Inline stryke at the prompt |

### How a name reaches a runtime

The shell library owns a registry of *host-registered native commands*
(`vendor/zshrs/src/extensions/native_cmds.rs`). It is the third builtin
registry in the shell and the only one the binary writes rather than the
library: `EXT_BUILTIN_NAMES` and the daemon's `ZSHRS_BUILTIN_NAMES` are `const`
arrays owned by the zshrs crate, and the three runtimes cannot be dependencies
of that crate — zvcs depends on its vendored gitoxide by path, which makes any
dependent unpublishable. So `src/main.rs` registers them before the shell reads
a line:

```rust
zsh::register_native_command("git", zvcs::run_argv);
zsh::register_native_command("arb", arb::cli::run_argv);
for name in ["stryke", "st", "s"] {
    zsh::register_native_command(name, stryke::cli::run_argv);
}
```

Every name a runtime ships a binary under is registered, not only the headline
one: strykelang installs `stryke`, `st` and `s`, three identical entry points
that differ by the `argv[0]` `stryke::cli` reads for itself.

Each runtime exposes one `run_argv(&[String]) -> i32` taking the whole command
line, `argv[0]` included, and each wraps it in its own `hosted::run`. That
wrapper is what makes a program written to own its process safe to call inside
one it does not: an `exit` from deep in a rendering loop unwinds back instead
of taking the shell down, a panic becomes an exit status, and a `git -C <dir>`
that moved the working directory is undone on the way out.

One thing the wrapper cannot undo is a child git spawns on purpose. git forks a
`git` child for a handful of jobs — `status` asks one for the submodule
summary, `submodule update` fetches through one, `rebase` drives `am` and
`commit` through them — and each site spawns "itself", which under a host is
the host: `Command::new(current_exe()).args(["submodule", "summary"])` ran *the
shell* and answered `zshrs: can't open input file: submodule`. zvcs resolves
the spawn target through `hosted::git_exe()` instead — `$ZVCS_GIT_EXE`, else
the first `git` on `PATH` that is not the host binary itself — so those
children reach a git. They are the one place a fork survives, exactly where git
itself forks.

A registered name occupies the *builtin* slot, so zsh's `alias → function →
builtin → external` resolution order is intact and every way of asking for the
binary on disk still gets it:

| | Result |
|---|---|
| `git status` | in-process — no fork, no `PATH` |
| `git() { … }; git status` | the function; a native command shadows like `cat` does |
| `command git status` | the `git` on `PATH` |
| `/usr/bin/git status` | that binary — a `/`-qualified word is never a registry key |
| `disable git` … `enable git` | falls through to `PATH` and back, per shell |

---

## [0x05] WHY A SEPARATE PACKAGE

The `zshrs` crate is published to crates.io. zvcs can never be: it depends on
its own vendored gitoxide fork by path, and a path dependency — even an
optional one behind a feature flag — makes a crate unpublishable. So the fat
build cannot live in `zshrs` as a feature. It lives here, with all four
runtimes as submodules.

The shell's whole REPL is in the zshrs repo's `bins/zshrs.rs`, a **binary**
target, and Cargo binaries cannot be imported by another package. `src/main.rs`
pulls it in as a module with `#[path]` instead — the same trick the original fat
binary used before the strykelang monorepo was split apart
(`strykelang/bins/zshrs.rs`, removed in strykelang `405bcfeeca` with the note
"Removed fat zshrs binary (lives in zshrs repo now)"; it was never recreated
there). `zshrs_main` is `pub` for exactly this caller.

```
src/main.rs        registers the runtimes, then runs the shell
vendor/zshrs       the shell
vendor/zvcs        git
vendor/arb         arb + the fzf engine
vendor/strykelang  stryke
```

---

## [0x06] LINK CONSTRAINTS

Both are load-bearing. Violating either breaks the build or the shell.

**One fusevm.** It exports 67 `#[no_mangle]` symbols across its `aot`/`jit`/`ffi`
modules, and Cargo treats `0.x` minors as semver-incompatible, so two copies in
the graph is a duplicate-symbol link failure. Every runtime must request the
same minor. `cargo check` does not link and will not catch a violation;
`cargo build` will. Only one `#[no_mangle] fusevm_aot_register_builtins` may
exist per binary — the zshrs dependency takes `default-features = false` to drop
its definition, so strykelang's is the one that links.

**`panic = "unwind"`.** The zshrs crate's own release profile uses
`panic = "abort"`. Profile settings come from the root package, so this one
governs the whole graph, and it deliberately differs: three large runtimes now
share the shell's address space, and an abort in any of them would take an
interactive shell down. Unwinding lets the dispatch boundary catch a panic and
return an exit status.

Linking the four surfaced three upstream conflicts, each fixed at its source:

| Runtime | Conflict | Resolution |
|---------|----------|------------|
| strykelang | fusevm 0.17 vs zshrs's 0.22 | Bumped to 0.22 with the zshrs dep (`7455abc`) |
| arb | ratatui 0.29 pinned `unicode-width =0.2.0`; zvcs's prodash needs `^0.2.2` — no version satisfies both | ratatui 0.30, which moved the dep to ratatui-core (`1bea073`) |
| zvcs | rusqlite 0.31 vs 0.32 elsewhere; `libsqlite3-sys` carries `links = "sqlite3"`, so Cargo permits exactly one | Bumped to 0.32 (`12026bf`) |

---

## [0x07] STATUS

| | State |
|---|---|
| Four runtimes link into one binary | **Working** — one fusevm, one libsqlite3-sys |
| `git` / `arb` / `stryke` as builtins | **Working.** `whence -w` reports `builtin`, `${+builtins[git]}` is 1, and with `PATH` emptied `git --version`, `stryke -e …` and `arb --filter …` all still answer while `ls` reports command not found — nothing is resolved through `PATH` or spawned |
| `@ <code>` → stryke | **Registered, not reached.** `zsh::try_stryke_dispatch` is consulted from `bins/zshrs.rs:process_line`, which serves only the line-by-line script reader; the prompt and `-c` go through the ported lexer in the library, where a leading `@` is still an ordinary character |

---

## [0xFF] LICENSE

MIT. See [LICENSE](LICENSE).
