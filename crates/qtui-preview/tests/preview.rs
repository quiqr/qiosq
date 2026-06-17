//! Integration test: start the real `hugo server` on a minimal site fixture,
//! assert the URL is surfaced and reachable, then assert clean shutdown.
//!
//! Gated on `hugo` being present on PATH. It is provided by the flake dev shell
//! and the Nix check sandbox, where this test runs for real; in a bare
//! environment without hugo the test no-ops with a printed notice rather than
//! failing.

use std::net::TcpStream;
use std::path::Path;
use std::time::{Duration, Instant};

use qtui_config::Config;
use qtui_preview::PreviewServer;

fn hugo_available() -> bool {
    std::process::Command::new("hugo")
        .arg("version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Write a minimal Hugo + Quiqr-shaped site into `dir`.
fn make_site(dir: &Path) {
    std::fs::create_dir_all(dir.join("content")).unwrap();
    std::fs::create_dir_all(dir.join("quiqr")).unwrap();
    std::fs::write(
        dir.join("hugo.toml"),
        "baseURL = \"http://localhost/\"\ntitle = \"fixture\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("content").join("_index.md"), "# Home\n").unwrap();
}

/// Build a validated `Config` whose preview range is the given window.
fn config_with_range(data_dir: &Path, low: u16, high: u16) -> Config {
    let tmp = data_dir.join("quiqr-tui.toml");
    let toml = format!(
        r#"
[storage]
quiqr_data_dir = "{data}"

[preview]
port_range = [{low}, {high}]
ready_timeout_ms = 30000

[agent]
command = "claude"
completion_sentinel = "<<QTUI_TASK_DONE>>"
"#,
        data = data_dir.display(),
    );
    std::fs::write(&tmp, toml).unwrap();
    Config::load_and_validate(&tmp).expect("test config should load")
}

/// Try a TCP connect to 127.0.0.1:port, retrying up to `dur`.
fn connectable(port: u16, dur: Duration) -> bool {
    let deadline = Instant::now() + dur;
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

#[test]
fn starts_serves_and_stops_cleanly() {
    if !hugo_available() {
        eprintln!("skipping: hugo not on PATH (provided by the nix dev shell / check sandbox)");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let site = tmp.path().join("mysite");
    make_site(&site);
    // Disjoint from the other integration test's range so the two can run in
    // parallel without racing for the same port.
    let config = config_with_range(tmp.path(), 13140, 13169);

    let server = PreviewServer::start(&config, &site).expect("preview should start");
    let url = server.url().to_string();
    let port = server.port();

    // The surfaced URL contains the selected port and is not the reserved one.
    assert!(
        url.contains(&port.to_string()),
        "url {url} should name port {port}"
    );
    assert_ne!(port, 13131, "must never serve on Quiqr's reserved port");
    assert!((13140..=13169).contains(&port));

    // The server is reachable.
    assert!(
        connectable(port, Duration::from_secs(10)),
        "served site should be reachable on 127.0.0.1:{port}"
    );

    // Stop it and assert the port frees (process gone, port re-bindable).
    server.stop();
    assert!(
        rebindable(port, Duration::from_secs(10)),
        "port {port} should be free after stop"
    );
}

#[test]
fn dropping_handle_frees_the_port() {
    if !hugo_available() {
        eprintln!("skipping: hugo not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let site = tmp.path().join("s");
    make_site(&site);
    // Disjoint from `starts_serves_and_stops_cleanly`'s range (see note there).
    let config = config_with_range(tmp.path(), 13170, 13200);

    let port = {
        let server = PreviewServer::start(&config, &site).expect("preview should start");
        let p = server.port();
        assert!(connectable(p, Duration::from_secs(10)));
        p
        // server dropped here without an explicit stop()
    };

    assert!(
        rebindable(port, Duration::from_secs(10)),
        "dropping the handle must terminate hugo and free port {port}"
    );
}

#[test]
fn readiness_timeout_errors_without_orphan() {
    if !hugo_available() {
        eprintln!("skipping: hugo not on PATH");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let site = tmp.path().join("slow");
    make_site(&site);
    // Disjoint range; an impossibly short readiness timeout so Hugo cannot be
    // ready in time and `start` must return a timeout error (and kill the child).
    let mut config = config_with_range(tmp.path(), 13201, 13230);
    config.preview.ready_timeout_ms = 1;

    let result = qtui_preview::PreviewServer::start(&config, &site);
    assert!(
        result.is_err(),
        "an ~immediate timeout should fail to start"
    );

    // No orphan: every port in the range frees up shortly (the killed child
    // released whatever it grabbed).
    for port in 13201..=13230 {
        assert!(
            rebindable(port, Duration::from_secs(5)),
            "port {port} should be free after a timed-out start (no orphan)"
        );
    }
}

/// Retry-binding a port until it succeeds or `dur` elapses (the OS may take a
/// moment to release it after the child dies).
fn rebindable(port: u16, dur: Duration) -> bool {
    let deadline = Instant::now() + dur;
    while Instant::now() < deadline {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}
