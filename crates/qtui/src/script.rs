//! Headless scripted mode: drive the full flow without a TTY, for the e2e test.
//!
//! Steps are a comma-separated list passed to `--script`. Supported steps:
//!   open-site   — open the chosen site (start preview, load model + content)
//!   open-file   — open the first content file found, read-only
//!   ask-ai      — run the configured agent with the file intent
//!   await       — assert the agent completed (sentinel seen) [implied by ask-ai]
//!
//! The chosen site is `--site <name>`, or the first enumerated site. Each step
//! prints a line so the VM test log shows progress; any failure is an error.

use std::path::Path;

use qtui_storage::{ContentNode, Site};

use crate::host::Host;

pub fn run(config_path: &Path, site_name: Option<&str>, steps: &[String]) -> Result<(), String> {
    let mut host = Host::load(config_path)?;

    // Resolve the site up front so the steps can borrow it.
    let site = pick_site(&host, site_name)?;
    println!("script: site = {}", site.name);

    for step in steps {
        match step.as_str() {
            "open-site" => {
                host.open_site(&site)?;
                println!(
                    "script: open-site ok (preview {})",
                    host.preview_url().unwrap_or("<none>")
                );
            }
            "open-file" => {
                let rel =
                    first_content_file(&host, &site).ok_or("open-file: no content file found")?;
                host.open_file(&site, &rel)?;
                println!("script: open-file ok ({rel})");
            }
            "ask-ai" => {
                let rel = host
                    .state
                    .open_file()
                    .map(str::to_string)
                    .ok_or("ask-ai: no file open (run open-file first)")?;
                host.run_agent_intent(&site, &rel)?;
                println!("script: ask-ai ok (agent completed for {rel})");
            }
            "await" => {
                // Completion is enforced inside run_agent_intent; this is a no-op
                // marker kept for readable scripts.
                println!("script: await ok");
            }
            other => return Err(format!("unknown script step {other:?}")),
        }
    }

    host.shutdown();
    println!("script: done");
    Ok(())
}

fn pick_site(host: &Host, name: Option<&str>) -> Result<Site, String> {
    match name {
        Some(n) => host
            .site_by_name(n)
            .cloned()
            .ok_or_else(|| format!("site {n:?} not found")),
        None => host
            .sites()
            .first()
            .cloned()
            .ok_or_else(|| "no sites found in the data directory".to_string()),
    }
}

/// Find the first file in the site's content tree (depth-first), returning its
/// path relative to `content/`.
fn first_content_file(host: &Host, site: &Site) -> Option<String> {
    let tree = qtui_storage::content_tree(site, &host.config.storage.hidden_dirs);
    fn find(nodes: &[ContentNode]) -> Option<String> {
        for n in nodes {
            match n {
                ContentNode::File { rel_path, .. } => {
                    return Some(rel_path.to_string_lossy().into_owned())
                }
                ContentNode::Dir { children, .. } => {
                    if let Some(f) = find(children) {
                        return Some(f);
                    }
                }
            }
        }
        None
    }
    find(&tree)
}
