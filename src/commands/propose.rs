use crate::config;
use crate::git;
use anyhow::{Context, Result};
use std::process::Command;

pub fn run() -> Result<()> {
    let repo = git::discover()?;
    let user = config::get_user(&repo)?;

    let weft_head = Command::new("jj")
        .args(&[
            "--no-pager",
            "log",
            "-r",
            "@",
            "-T",
            "commit_id",
            "--no-graph",
        ])
        .current_dir(repo.path())
        .output()?
        .stdout;
    let weft_head = String::from_utf8_lossy(&weft_head).trim().to_string();

    let commit_hash = weft_head.clone();
    let short_hash = if commit_hash.len() >= 8 {
        &commit_hash[..8]
    } else {
        &commit_hash
    };

    let candidate_id = format!("{}-{}", user, short_hash);

    let candidate_ref = format!("refs/loom/{}", candidate_id);

    let output = Command::new("git")
        .args(&[
            "push",
            "origin",
            &format!("{}:refs/{}", weft_head, candidate_ref),
        ])
        .current_dir(repo.path())
        .output()
        .context("Failed to create candidate")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to create candidate: {}", stderr));
    }

    println!("Candidate created: {}", candidate_ref);
    println!("\nNext steps:");
    println!("  weft status  # Check candidate status");
    println!("  weft weave {}  # Merge when ready", candidate_id);

    Ok(())
}
