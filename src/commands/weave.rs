use crate::git;
use anyhow::{Context, Result};
use std::process::Command;

pub fn run(candidate_id: &str) -> Result<()> {
    let repo = git::discover()?;

    let candidate_ref = format!("refs/loom/{}", candidate_id);

    let candidate_commit = match repo.find_reference(&candidate_ref) {
        Ok(ref_) => ref_.peel_to_commit()?.id().to_string(),
        Err(_) => {
            let fetch_output = Command::new("git")
                .args(&[
                    "fetch",
                    "origin",
                    &format!("refs/loom/{}:refs/loom/{}", candidate_id, candidate_id),
                ])
                .current_dir(repo.path())
                .output()
                .context("Failed to fetch candidate")?;

            if !fetch_output.status.success() {
                let stderr = String::from_utf8_lossy(&fetch_output.stderr);
                return Err(anyhow::anyhow!(
                    "Candidate '{}' not found locally or on remote.\n{}",
                    candidate_id,
                    stderr
                ));
            }

            match repo.find_reference(&candidate_ref) {
                Ok(ref_) => ref_.peel_to_commit()?.id().to_string(),
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Candidate '{}' not found. Run 'weft propose' first.",
                        candidate_id
                    ));
                }
            }
        }
    };

    let output = Command::new("jj")
        .args(&[
            "log",
            "-r",
            &candidate_id,
            "-T",
            "if(conflict, \"tangled\", \"clean\")",
        ])
        .current_dir(repo.path())
        .output()
        .context("Failed to check candidate state")?;

    if output.status.success() {
        let state = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if state == "tangled" {
            return Err(anyhow::anyhow!(
                "Cannot weave: candidate '{}' has unresolved conflicts. Run 'weft untangle' first.",
                candidate_id
            ));
        }
    }

    let main_ref = "refs/heads/main";
    let _main_commit = match repo.find_reference(main_ref) {
        Ok(ref_) => ref_.peel_to_commit()?.id().to_string(),
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Branch 'main' not found. Create it first or run 'weft sync'."
            ));
        }
    };

    let output = Command::new("git")
        .args(&[
            "push",
            "origin",
            &format!("{}:refs/heads/main", candidate_commit),
        ])
        .current_dir(repo.path())
        .output()
        .context("Failed to update main")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("non-fast-forward") || stderr.contains("updates were rejected") {
            return Err(anyhow::anyhow!(
                "Weave failed: main branch has been updated since candidate was created.\n\
                 Run 'weft sync' and try again, or re-propose your changes."
            ));
        }
        return Err(anyhow::anyhow!("Failed to update main: {}", stderr));
    }

    let output = Command::new("git")
        .args(&["reset", "--hard", &format!("origin/main")])
        .current_dir(repo.path())
        .output()
        .context("Failed to update local main")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Warning: Failed to update local main: {}", stderr);
    }

    let output = Command::new("git")
        .args(&["push", "origin", &format!(":{}", candidate_ref)])
        .current_dir(repo.path())
        .output()
        .context("Failed to clean up candidate")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Warning: Failed to clean up candidate ref: {}", stderr);
    }

    println!("Woven '{}' into main.", candidate_id);
    println!("\nNext steps:");
    println!("  weft sync  # Update your weft with the new main");

    Ok(())
}
