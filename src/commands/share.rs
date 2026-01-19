use crate::config;
use crate::git;
use anyhow::{Context, Result};
use std::process::Command;

pub fn run() -> Result<()> {
    let repo = git::discover()?;
    let user = config::get_user(&repo)?;

    let weft_head_ref = format!("refs/weft/{}/head", user);
    let weft_head = match repo.find_reference(&weft_head_ref) {
        Ok(ref_) => ref_.peel_to_commit()?.id().to_string(),
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Weft not initialized. Run 'weft init' first."
            ));
        }
    };

    let remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => {
            return Err(anyhow::anyhow!(
                "No remote 'origin' configured. Add a remote with: git remote add origin <url>"
            ));
        }
    };

    let remote_url = remote.url().unwrap_or("").to_string();
    if remote_url.is_empty() {
        return Err(anyhow::anyhow!(
            "Remote 'origin' has no URL. Configure with: git remote set-url origin <url>"
        ));
    }

    let remote_ref = format!("refs/weft/{}", user);

    let mut cmd = Command::new("git");
    cmd.args(&[
        "push",
        "origin",
        &format!("{}:refs/{}", weft_head, remote_ref),
    ]);

    let output = cmd
        .current_dir(repo.path())
        .output()
        .context("Failed to run git push")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("could not read") || stderr.contains("does not exist") {
            return Err(anyhow::anyhow!(
                "Remote branch does not exist. Push may fail if remote doesn't allow refs/weft/*"
            ));
        }
        return Err(anyhow::anyhow!("Failed to push weft to remote: {}", stderr));
    }

    println!("Shared weft to: {}", remote_url);
    println!("Remote ref: refs/weft/{}", user);

    Ok(())
}
