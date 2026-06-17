//! Bridge tests driven through the `Agent` trait with the in-process
//! `FakeAgent`, plus a test that drives the scripted `fake-agent` binary. No
//! live rmux daemon is involved (see bean qiosq-rts9) — this is the
//! deterministic, offline coverage of the bridge's observable behaviour.

use std::io::Write;
use std::process::{Command, Stdio};

use qtui_agent::{contains_sentinel, format_intent, Agent, FakeAgent};

const SENTINEL: &str = "<<QTUI_TASK_DONE>>";

#[test]
fn format_intent_has_the_at_path_prefix() {
    let intent = format_intent("content/posts/hello.md");
    assert!(
        intent.starts_with("@content/posts/hello.md I want to do the following… "),
        "got: {intent:?}"
    );
}

#[test]
fn sentinel_predicate() {
    assert!(contains_sentinel("...done <<QTUI_TASK_DONE>> ok", SENTINEL));
    assert!(!contains_sentinel("still working", SENTINEL));
    // An empty sentinel must never spuriously complete.
    assert!(!contains_sentinel("anything", ""));
}

#[test]
fn send_intent_requires_start() {
    let mut agent = FakeAgent::new();
    // Without start(), send_intent must error (NotStarted) — no implicit session.
    assert!(agent.send_intent("content/x.md", "do it").is_err());
}

#[test]
fn send_intent_injects_prefix_then_completes_on_sentinel() {
    let dir = std::env::temp_dir();
    let mut agent = FakeAgent::new();
    agent.start(&dir, &String::new()).expect("start");
    assert_eq!(agent.started_in(), Some(dir.as_path()));

    agent
        .send_intent("content/posts/hello.md", "translate to English")
        .expect("send_intent");

    // The sent text carries the @path prefix and the user's continuation.
    assert_eq!(agent.sent.len(), 1);
    assert!(agent.sent[0].starts_with("@content/posts/hello.md I want to do the following… "));
    assert!(agent.sent[0].ends_with("translate to English"));

    // Not complete until the agent emits the sentinel.
    assert!(!agent.is_complete(SENTINEL));
    agent.emit("working...\n");
    assert!(!agent.is_complete(SENTINEL));
    agent.emit(&format!("{SENTINEL}\n"));
    assert!(agent.is_complete(SENTINEL));

    // snapshot() returns the accumulated visible text.
    let snap = agent.snapshot().expect("snapshot");
    assert!(snap.text().contains(SENTINEL));
}

#[test]
fn scripted_fake_agent_binary_writes_file_and_prints_sentinel() {
    // The cargo-built path to the `fake-agent` bin declared in Cargo.toml.
    let bin = env!("CARGO_BIN_EXE_fake-agent");

    let tmp = tempfile::tempdir().unwrap();
    let out_file = tmp.path().join("written.md");

    let mut child = Command::new(bin)
        .env("QTUI_FAKE_SENTINEL", SENTINEL)
        .env("QTUI_FAKE_WRITE", &out_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn fake-agent");

    // Feed it a finished intent on stdin.
    let intent = format!("{}make a post\n", format_intent("content/x.md"));
    child
        .stdin
        .take()
        .unwrap()
        .write_all(intent.as_bytes())
        .unwrap();

    let output = child.wait_with_output().expect("wait fake-agent");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    // It printed the sentinel...
    assert!(
        contains_sentinel(&stdout, SENTINEL),
        "fake-agent should print the sentinel, got: {stdout:?}"
    );
    // ...and wrote the known file (simulating the agent producing content).
    let written = std::fs::read_to_string(&out_file).expect("fake-agent should write the file");
    assert!(written.contains("written by fake-agent"));
    assert!(written.contains("make a post"));
}

#[test]
fn sending_an_intent_writes_no_site_files() {
    // The FakeAgent records sent text only; prove it touches no files by running
    // it against an empty tempdir and asserting the dir stays empty.
    let tmp = tempfile::tempdir().unwrap();
    let before: Vec<_> = std::fs::read_dir(tmp.path()).unwrap().collect();
    assert!(before.is_empty());

    let mut agent = FakeAgent::new();
    agent.start(tmp.path(), &String::new()).unwrap();
    agent.send_intent("content/x.md", "do it").unwrap();

    let after: Vec<_> = std::fs::read_dir(tmp.path()).unwrap().collect();
    assert!(
        after.is_empty(),
        "send_intent must not create any files in the site dir"
    );
}
