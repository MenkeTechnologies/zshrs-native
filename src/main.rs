//! `zshrs-native` — the zshrs shell with three sibling runtimes linked in.
//!
//! This is the shell binary you run. It is byte-for-byte the same shell as the
//! thin `zshrs`, plus `git` (zvcs), `arb`, and `stryke` compiled into the same
//! address space. They are dispatched as shell builtins, so none of them costs
//! a fork/exec — the same treatment `cat`, `head` and `sort` already get.
//!
//! # Why a separate package
//!
//! The zshrs crate is published to crates.io. zvcs cannot be: it depends on
//! its own vendored gitoxide fork by path (`src/ported/gix`). A path
//! dependency, even an optional one behind a feature, makes a package
//! unpublishable — so the fat build cannot live in the zshrs crate as a
//! feature flag. It lives here, with all four runtimes as `vendor/`
//! submodules.
//!
//! # Shape
//!
//! The shell's whole REPL lives in the zshrs repo's `bins/zshrs.rs`, which is
//! a *binary* target — Cargo binaries cannot be imported by other packages.
//! It is pulled in here as a module via `#[path]` instead, which is how the
//! original fat binary did it before the strykelang monorepo was split apart
//! (`strykelang/bins/zshrs.rs`, deleted in strykelang commit 405bcfeeca with
//! the note "Removed fat zshrs binary (lives in zshrs repo now)" — it never
//! got recreated there). `zshrs_main` is `pub` for exactly this caller.

#[path = "../vendor/zshrs/bins/zshrs.rs"]
#[allow(dead_code, unused_imports, unused_variables, unreachable_code, clippy::all)]
mod shell;

fn main() {
    // The three sibling runtimes become shell builtins here. Each exposes one
    // `run_argv(&[String]) -> i32` — the whole command line, `argv[0]`
    // included, exactly as its own `main` would have received it — and each
    // wraps that in its `hosted::run`, which is what makes a runtime written
    // to own its process safe to call inside one it does not: an `exit` from
    // deep in a rendering loop unwinds back instead of taking the shell down,
    // a panic becomes an exit status, and a `-C` that moved the working
    // directory is undone on the way out.
    //
    // `zsh::register_native_command` puts the name in the shell's *builtin*
    // slot, so zsh's alias → function → builtin → external order is preserved:
    // a user `git() { … }` still shadows this, and `command git` still reaches
    // whatever `git` is on `PATH`. What is gone is the fork, the execve, the
    // PATH walk and the dynamic loader — `git status` is now a function call.
    //
    // Every name each runtime ships a binary under is registered, not just the
    // headline one. strykelang installs `stryke`, `st` and `s` — three
    // identical entry points whose behaviour differs by `argv[0]`, which
    // `stryke::cli` reads for itself — so registering only `stryke` would have
    // left `s` and `st` forking to `/opt/homebrew/bin/s` from inside a shell
    // that has strykelang linked in.
    zsh::register_native_command("git", |argv| restore_terminal(zvcs::run_argv(argv)));
    zsh::register_native_command("arb", |argv| restore_terminal(arb::cli::run_argv(argv)));
    for name in ["stryke", "st", "s"] {
        zsh::register_native_command(name, |argv| {
            restore_terminal(stryke::cli::run_argv(argv))
        });
    }

    // `@ <code>` runs stryke instead of shell code. The hook is a `OnceLock`
    // in the zshrs lib (`zsh::set_stryke_handler`); the thin binary never
    // registers one, so `@` there is an ordinary character. It is consulted
    // via `zsh::try_stryke_dispatch` from `intercept` advice bodies
    // (`execute_advice`) and from `process_line`, which nothing reaches; the
    // prompt and `-c` do not consult it.
    zsh::set_stryke_handler(|code| {
        restore_terminal(match stryke::run(code) {
            Ok(_) => 0,
            Err(e) => {
                // zsh-style terse diagnostic on stderr: `zshrs: <cmd>: <reason>`.
                eprintln!("zshrs: stryke: {e}");
                1
            }
        })
    });

    shell::zshrs_main();
}

/// Put the terminal back the way a runtime found it, as process exit used to.
///
/// crossterm keeps the termios it saved on `enable_raw_mode` in a process-wide
/// static, and while that static is set `enable_raw_mode` returns `Ok` without
/// touching the terminal (crossterm 0.29 `src/terminal/sys/unix.rs:108`). A
/// runtime that left raw mode set by any path that skips `disable_raw_mode` —
/// an `exit` unwound by `hosted::run`, a `?` between arb's `enable_raw_mode`
/// and its `disable_raw_mode`, a caught panic — used to lose that static with
/// its process. In here it outlives the command: zle restores cooked mode at
/// the next prompt, and every later reedline (stryke's REPL, arb's REPL) then
/// "enables" raw mode on a cooked terminal. The terminal's answer to the
/// cursor-position query sits in the canonical line buffer, crossterm times out
/// with "The cursor position could not be read within a normal duration", and
/// the answer is echoed into the next line read as `^[[80;1R`.
///
/// `disable_raw_mode` is a no-op when the static is clear, so calling it after
/// every native command costs nothing when the runtime cleaned up after itself.
fn restore_terminal(status: i32) -> i32 {
    let _ = crossterm::terminal::disable_raw_mode();
    status
}
