//! `qtui init`: discover the Quiqr storage (or ask), then write the config.
//!
//! Interactive: list the discovered candidates and let the user pick one or
//! enter a custom path. Headless (no TTY): auto-select the single valid
//! candidate, else error listing what was found. Writes
//! `~/.config/qiosq/config.toml`, refusing to clobber an existing one unless
//! `--force`.

use std::io::{IsTerminal, Write};

use qtui_config::discover::{self, DataDirCandidate, Source};

pub fn run(force: bool) -> Result<(), String> {
    let candidates = discover::discover();
    let target = qtui_config::default_config_path()
        .ok_or("could not determine the config location (no OS config dir)")?;

    if target.exists() && !force {
        return Err(format!(
            "config already exists at {} (pass --force to overwrite)",
            target.display()
        ));
    }

    // Choose a data directory.
    let chosen = if std::io::stdin().is_terminal() {
        choose_interactive(&candidates)?
    } else {
        choose_headless(&candidates)?
    };

    qtui_config::write_config(&target, &chosen, force).map_err(|e| e.to_string())?;
    println!("Wrote config to {} (data dir: {chosen})", target.display());
    println!("Run `qtui` to start.");
    Ok(())
}

fn describe(c: &DataDirCandidate) -> String {
    let source = match c.source {
        Source::ElectronSettings => "Quiqr desktop settings",
        Source::Fallback => "default location",
    };
    let status = if c.valid {
        format!("{} site(s)", c.site_count)
    } else {
        "no sites / not found".to_string()
    };
    format!("{}  [{source}, {status}]", c.path.display())
}

/// Non-interactive: auto-select the single valid candidate, else error listing
/// what was found (never prompt).
fn choose_headless(candidates: &[DataDirCandidate]) -> Result<String, String> {
    let valid: Vec<&DataDirCandidate> = candidates.iter().filter(|c| c.valid).collect();
    match valid.as_slice() {
        [only] => Ok(only.path.to_string_lossy().into_owned()),
        _ => Err(format!(
            "cannot auto-select a Quiqr storage folder ({} valid candidate(s)). Found:\n{}",
            valid.len(),
            candidates
                .iter()
                .map(|c| format!("  - {}", describe(c)))
                .collect::<Vec<_>>()
                .join("\n")
        )),
    }
}

/// Interactive: print the candidates and read a choice (index, `c` for a custom
/// path, or `q` to quit).
fn choose_interactive(candidates: &[DataDirCandidate]) -> Result<String, String> {
    println!("Where is your Quiqr storage?\n");
    for (i, c) in candidates.iter().enumerate() {
        println!("  {}) {}", i + 1, describe(c));
    }
    println!("  c) enter a custom path");
    println!("  q) quit");

    loop {
        print!("\nChoice: ");
        std::io::stdout().flush().ok();
        let mut line = String::new();
        if std::io::stdin()
            .read_line(&mut line)
            .map_err(|e| e.to_string())?
            == 0
        {
            return Err("no input".into());
        }
        let line = line.trim();
        match line {
            "q" | "Q" => return Err("cancelled".into()),
            "c" | "C" => {
                print!("Path: ");
                std::io::stdout().flush().ok();
                let mut p = String::new();
                std::io::stdin()
                    .read_line(&mut p)
                    .map_err(|e| e.to_string())?;
                let p = p.trim();
                if p.is_empty() {
                    println!("(empty path)");
                    continue;
                }
                return Ok(p.to_string());
            }
            _ => match line.parse::<usize>() {
                Ok(n) if (1..=candidates.len()).contains(&n) => {
                    return Ok(candidates[n - 1].path.to_string_lossy().into_owned());
                }
                _ => println!("Please enter a number 1..={}, c, or q.", candidates.len()),
            },
        }
    }
}
