//! Hugo preview lifecycle.
//!
//! On site open, [`PreviewServer::start`] picks a free port in the configured
//! range (never Quiqr's default `13131`), starts `hugo server` for the site,
//! waits for Hugo's "Web Server is available at …" readiness line, and surfaces
//! the URL. The server is stopped on [`PreviewServer::stop`] and, as a safety
//! net, on `Drop`, so no `hugo server` process is ever orphaned. One server at a
//! time (the handle is owned).
//!
//! This crate owns an external process but never writes site files.

use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::ops::RangeInclusive;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use qtui_config::{Config, QUIQR_RESERVED_PORT};
use thiserror::Error;

/// The marker Hugo prints when the dev server is serving.
const READY_MARKER: &str = "Web Server is available at";

/// Errors from the preview layer.
#[derive(Debug, Error)]
pub enum PreviewError {
    #[error("no free port available in range {low}..={high} (excluding {reserved})", reserved = QUIQR_RESERVED_PORT)]
    NoFreePort { low: u16, high: u16 },

    #[error("failed to start hugo ('{bin}'): {source}")]
    Spawn {
        bin: String,
        #[source]
        source: std::io::Error,
    },

    #[error("hugo did not become ready within {0:?}")]
    ReadyTimeout(Duration),

    #[error("hugo exited before becoming ready")]
    ExitedEarly,
}

/// Pick a free port inside `range`, skipping [`QUIQR_RESERVED_PORT`]. A port is
/// "free" if it can be bound on `127.0.0.1` at probe time. Returns the first such
/// port, or [`PreviewError::NoFreePort`] when none is available.
pub fn pick_port(range: RangeInclusive<u16>) -> Result<u16, PreviewError> {
    let (low, high) = (*range.start(), *range.end());
    for port in range {
        if port == QUIQR_RESERVED_PORT {
            continue;
        }
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            // The probe listener drops here, freeing the port for Hugo.
            return Ok(port);
        }
    }
    Err(PreviewError::NoFreePort { low, high })
}

/// A running `hugo server` for one site. Stops the child on [`Self::stop`] and
/// on drop.
#[derive(Debug)]
pub struct PreviewServer {
    child: Child,
    url: String,
    port: u16,
}

impl PreviewServer {
    /// Start `hugo server` for the site at `site_dir`, using the hugo binary,
    /// port range, and readiness timeout from `config`. Blocks until Hugo
    /// reports ready or the timeout elapses (in which case the child is killed).
    pub fn start(config: &Config, site_dir: &Path) -> Result<Self, PreviewError> {
        let port = pick_port(port_range(config))?;
        let bin = config.preview.hugo_bin.clone();
        let base_url = format!("http://localhost:{port}/");

        let mut child = Command::new(&bin)
            .arg("server")
            .arg("--bind")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(port.to_string())
            .arg("--baseURL")
            .arg(&base_url)
            .arg("--renderToMemory")
            .current_dir(site_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|source| PreviewError::Spawn { bin, source })?;

        // Stream stdout on a worker thread; report the readiness URL once seen.
        let stdout = child.stdout.take().expect("stdout was piped");
        let (tx, rx) = mpsc::channel::<String>();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(idx) = line.find(READY_MARKER) {
                    let after = line[idx + READY_MARKER.len()..].trim();
                    // The URL is the first whitespace-delimited token after the
                    // marker, e.g. "http://localhost:1313/".
                    let url = after.split_whitespace().next().unwrap_or("").to_string();
                    let _ = tx.send(url);
                    break;
                }
            }
            // Channel closes when the thread ends; a closed channel without a
            // value signals "stdout ended before readiness".
        });

        let timeout = Duration::from_millis(config.preview.ready_timeout_ms);
        match rx.recv_timeout(timeout) {
            Ok(reported) => {
                let url = if reported.is_empty() {
                    base_url
                } else {
                    reported
                };
                Ok(Self { child, url, port })
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let _ = child.kill();
                let _ = child.wait();
                Err(PreviewError::ReadyTimeout(timeout))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = child.kill();
                let _ = child.wait();
                Err(PreviewError::ExitedEarly)
            }
        }
    }

    /// The URL the site is served at (what the UI surfaces to the user).
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The port the server is bound to.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Stop the server, terminating and reaping the child process.
    pub fn stop(mut self) {
        self.terminate();
    }

    fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for PreviewServer {
    fn drop(&mut self) {
        // Safety net: never leave an orphaned `hugo server` running.
        self.terminate();
    }
}

fn port_range(config: &Config) -> RangeInclusive<u16> {
    let [low, high] = config.preview.port_range;
    low..=high
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn pick_port_returns_free_in_range() {
        let port = pick_port(13140..=13200).expect("a port should be free");
        assert!((13140..=13200).contains(&port));
        // It was free at selection time, so we can bind it now.
        TcpListener::bind(("127.0.0.1", port)).expect("selected port should be bindable");
    }

    #[test]
    fn pick_port_skips_an_occupied_low_port() {
        // Occupy the bottom of a tiny range; expect the next one.
        let low = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let low_port = low.local_addr().unwrap().port();
        // Range = [low_port, low_port+1]; low_port is held, so we must get +1.
        let picked = pick_port(low_port..=low_port + 1).expect("the higher port is free");
        assert_eq!(picked, low_port + 1);
        drop(low);
    }

    #[test]
    fn pick_port_skips_reserved_13131() {
        // A degenerate range of exactly {13131} must fail (only the reserved port).
        let err = pick_port(QUIQR_RESERVED_PORT..=QUIQR_RESERVED_PORT)
            .expect_err("the reserved port must be skipped");
        assert!(matches!(err, PreviewError::NoFreePort { .. }));
    }

    #[test]
    fn pick_port_errors_when_range_exhausted() {
        // Hold a port, then offer a range of just that one port.
        let held = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let p = held.local_addr().unwrap().port();
        let err = pick_port(p..=p).expect_err("single occupied port => no free port");
        match err {
            PreviewError::NoFreePort { low, high } => {
                assert_eq!((low, high), (p, p));
            }
            other => panic!("expected NoFreePort, got {other:?}"),
        }
        drop(held);
    }
}
