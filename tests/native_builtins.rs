//! The wiring `src/main.rs` adds on top of the thin `zshrs`: `git`, `arb`,
//! `stryke`, `st` and `s` registered as in-process builtins.
//!
//! Each test runs the built `zshrs` with `-f -c` and an isolated `HOME`, so no
//! user startup file, cache or `PATH` binary takes part. `PATH` is pointed at
//! an empty directory wherever the point is that the native builtin — and not
//! some `git` on the machine — answered.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const NATIVE_NAMES: [&str; 5] = ["git", "arb", "stryke", "st", "s"];

/// A per-test scratch directory under the system temp dir, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "zshrs-native-test-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("home")).unwrap();
        std::fs::create_dir_all(dir.join("emptybin")).unwrap();
        Scratch(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Run `script` under `zshrs -f -c` in `scratch`, with `PATH` holding nothing.
fn run(scratch: &Scratch, script: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zshrs"))
        .args(["-f", "-c", script])
        .current_dir(scratch.path())
        .env_clear()
        .env("HOME", scratch.path().join("home"))
        .env("PATH", scratch.path().join("emptybin"))
        .env("TERM", "dumb")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("spawn zshrs")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn describe(out: &Output) -> String {
    format!(
        "status={:?}\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Every name a runtime installs a binary under is registered, and in the
/// builtin slot — `s` and `st` included, which `main` registers alongside
/// `stryke` so they do not fork to a `PATH` copy.
#[test]
fn every_runtime_name_is_a_builtin() {
    let s = Scratch::new("whence");
    let out = run(&s, &format!("whence -w {}", NATIVE_NAMES.join(" ")));
    let expected: String = NATIVE_NAMES
        .iter()
        .map(|n| format!("{n}: builtin\n"))
        .collect();
    assert_eq!(stdout(&out), expected, "{}", describe(&out));
    assert!(out.status.success(), "{}", describe(&out));
}

/// `${(k)builtins}` reports the same set the shell will dispatch.
#[test]
fn builtins_assoc_lists_native_names() {
    let s = Scratch::new("assoc");
    let script = NATIVE_NAMES
        .iter()
        .map(|n| format!("(( ${{+builtins[{n}]}} )) && print {n}"))
        .collect::<Vec<_>>()
        .join("; ");
    let out = run(&s, &script);
    let expected: String = NATIVE_NAMES.iter().map(|n| format!("{n}\n")).collect();
    assert_eq!(stdout(&out), expected, "{}", describe(&out));
}

/// With nothing on `PATH`, `git` still runs: the call never left the process.
#[test]
fn git_runs_without_path() {
    let s = Scratch::new("gitver");
    let out = run(&s, "git --version");
    assert!(out.status.success(), "{}", describe(&out));
    assert!(stdout(&out).starts_with("git version "), "{}", describe(&out));
}

/// A user function shadows the native builtin (alias → function → builtin),
/// and `builtin git` still reaches the native one past the function.
#[test]
fn function_shadows_native_and_builtin_reaches_it() {
    let s = Scratch::new("shadow");
    let out = run(
        &s,
        "git() { print shadowed; }; git --version; builtin git --version",
    );
    let text = stdout(&out);
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("shadowed"), "{}", describe(&out));
    assert!(
        lines.next().is_some_and(|l| l.starts_with("git version ")),
        "{}",
        describe(&out)
    );
}

/// `command git` asks for the `PATH` binary, so with an empty `PATH` it is not
/// found rather than silently running the in-process one.
#[test]
fn command_prefix_bypasses_native_git() {
    let s = Scratch::new("command");
    let out = run(&s, "command git --version; print status=$?");
    assert!(!stdout(&out).contains("git version"), "{}", describe(&out));
    assert!(stdout(&out).contains("status=127"), "{}", describe(&out));
}

/// `disable git` masks the native builtin (it falls through to the empty
/// `PATH`) and `enable git` brings it back.
#[test]
fn disable_and_enable_toggle_native_git() {
    let s = Scratch::new("disable");
    let out = run(
        &s,
        "disable git; git --version; print off=$?; enable git; git --version; print on=$?",
    );
    let text = stdout(&out);
    assert!(text.contains("off=127"), "{}", describe(&out));
    assert!(text.contains("git version "), "{}", describe(&out));
    assert!(text.contains("on=0"), "{}", describe(&out));
}

/// A failing native command reports a non-zero status to `$?` and the shell
/// keeps running the rest of the script.
#[test]
fn native_failure_status_reaches_the_shell() {
    let s = Scratch::new("gitfail");
    let out = run(&s, "git no-such-subcommand-xyz 2>/dev/null; print status=$?; print after");
    let text = stdout(&out);
    let status: i32 = text
        .lines()
        .find_map(|l| l.strip_prefix("status="))
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| panic!("no status line\n{}", describe(&out)));
    assert_ne!(status, 0, "{}", describe(&out));
    assert!(text.ends_with("after\n"), "{}", describe(&out));
}

/// `git -C DIR` changes directory for the duration of the command only; the
/// shell's working directory is unchanged afterwards.
#[test]
fn git_dash_c_does_not_move_the_shell() {
    let s = Scratch::new("gitc");
    let out = run(
        &s,
        "git init -q repo && git -C repo rev-parse --is-inside-work-tree && print -r -- ${PWD:t}",
    );
    let text = stdout(&out);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.first().copied(), Some("true"), "{}", describe(&out));
    let expected = s.path().file_name().unwrap().to_string_lossy().into_owned();
    assert_eq!(lines.get(1).copied(), Some(expected.as_str()), "{}", describe(&out));
}

/// `exit N` inside stryke unwinds back to the shell as status N instead of
/// ending the shell process — for every name stryke is registered under.
#[test]
fn stryke_exit_is_a_status_not_a_process_exit() {
    let s = Scratch::new("strykeexit");
    let script = ["stryke", "st", "s"]
        .iter()
        .enumerate()
        .map(|(i, n)| format!("{n} -e 'exit {}'; print {n}=$?", i + 3))
        .collect::<Vec<_>>()
        .join("; ");
    let out = run(&s, &script);
    assert_eq!(stdout(&out), "stryke=3\nst=4\ns=5\n", "{}", describe(&out));
    assert!(out.status.success(), "{}", describe(&out));
}

/// stryke output lands on the shell's stdout and can be captured.
#[test]
fn stryke_output_is_capturable() {
    let s = Scratch::new("strykecap");
    let out = run(&s, "x=$(stryke -e 'print 6*7'); print -r -- \"[$x]\"");
    assert_eq!(stdout(&out), "[42]\n", "{}", describe(&out));
}

/// The `@` stryke hook is consulted only at the interactive prompt and in
/// `intercept` advice; under `-c` `@` is an ordinary command word.
#[test]
fn at_hook_is_not_consulted_under_dash_c() {
    let s = Scratch::new("at");
    let out = run(&s, "@ print 1 2>/dev/null; print status=$?");
    assert_eq!(stdout(&out), "status=127\n", "{}", describe(&out));
}
