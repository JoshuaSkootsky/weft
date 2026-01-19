use crate::git;
use crate::config;
use anyhow::Result;
use std::process::Command;
use serde_json;

pub fn run() -> Result<()> {
    let repo = git::discover()?;
    let _user = config::get_user(&repo)?;

    let log_content = match git::get_op_log(&repo) {
        Ok(Some(content)) => content,
        Ok(None) => {
            return Err(anyhow::anyhow!("No operations to undo"));
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to read op-log: {}", e));
        }
    };

    let lines: Vec<&str> = log_content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return Err(anyhow::anyhow!("No operations to undo"));
    }

    let last_op: serde_json::Value = serde_json::from_str(lines.last().unwrap())
        .map_err(|_| anyhow::anyhow!("Failed to parse op-log"))?;

    let command = last_op["command"].as_str().unwrap_or("unknown");
    let inverse = &last_op["inverse"];

    let op_type = inverse["op"].as_str().unwrap_or("");

    match op_type {
        "delete-commit" => {
            let output = Command::new("jj")
                .args(&["op", "undo", "--no-pager"])
                .current_dir(repo.path())
                .output()?;

            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "Failed to undo: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            println!("Undid: weft {}", command);
        }
        "reset-ref" => {
            let ref_name = inverse["ref"].as_str().unwrap_or("");
            let old_oid = inverse["old"].as_str().unwrap_or("");

            if !ref_name.is_empty() && !old_oid.is_empty() {
                let old_oid_parsed = old_oid.parse::<git2::Oid>()?;
                repo.reference(ref_name, old_oid_parsed, true, &format!("undo {}", command))?;
            }

            println!("Undid: weft {}", command);
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Cannot undo operation: unknown inverse op type '{}'",
                op_type
            ));
        }
    }

    Ok(())
}
