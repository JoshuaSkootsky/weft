use crate::git;
use crate::config;
use anyhow::Result;
use std::process::Command;
use chrono::Utc;

pub fn run(message: &str) -> Result<()> {
    let repo = git::discover()?;
    let user = config::get_user(&repo)?;

    let output = Command::new("jj")
        .args(&[
            "describe",
            "-m",
            &format!("save: {}", message),
        ])
        .current_dir(repo.path())
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to create save: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let jj_log = Command::new("jj")
        .args(&["log", "-r", "@", "-T", "commit_id"])
        .current_dir(repo.path())
        .output()?;

    if !jj_log.status.success() {
        return Err(anyhow::anyhow!("Failed to get commit id"));
    }

    let commit_id = String::from_utf8_lossy(&jj_log.stdout)
        .trim()
        .to_string();

    let head = repo.head()?.peel_to_commit()?.id();

    git::update_weft_head(&repo, &user, head, "weft save")?;

    let now = Utc::now();
    let op_entry = serde_json::json!({
        "timestamp": now.timestamp(),
        "command": "save",
        "args": {"message": message},
        "inverse": {
            "op": "delete-commit",
            "commit": commit_id
        }
    });

    git::update_op_log(&repo, &op_entry.to_string())?;

    println!("Saved: {}", message);

    Ok(())
}
