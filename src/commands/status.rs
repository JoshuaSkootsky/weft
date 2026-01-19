use crate::git;
use crate::config;
use anyhow::Result;
use std::process::Command;

pub fn run() -> Result<()> {
    let repo = git::discover()?;
    let user = config::get_user(&repo)?;

    let weft_head_ref = format!("refs/weft/{}/head", user);
    let weft_head = match repo.find_reference(&weft_head_ref) {
        Ok(ref_) => ref_.peel_to_commit()?.id().to_string(),
        Err(_) => {
            println!("Weft not initialized. Run 'weft init' first.");
            return Ok(());
        }
    };

    let origin_main_oid = git::get_origin_main(&repo).ok();

    let jj_head_output = Command::new("jj")
        .args(&["log", "-r", "@", "-T", "commit_id"])
        .current_dir(repo.path())
        .output()?;

    if jj_head_output.status.success() {
        let jj_head = String::from_utf8_lossy(&jj_head_output.stdout).trim().to_string();
        if let Some(origin_main) = origin_main_oid {
            let ahead_count = count_commits_between(&repo, origin_main.to_string(), jj_head)?;
            println!("Weft: {} commits ahead of warp", ahead_count);
        } else {
            let jj_log = Command::new("jj")
                .args(&["log", "-r", "::", "-T", "description.first_line()"])
                .current_dir(repo.path())
                .output()?;

            if jj_log.status.success() {
                let log_output = String::from_utf8_lossy(&jj_log.stdout).to_string();
                let commits: Vec<&str> = log_output
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .collect();
                if !commits.is_empty() {
                    println!("Weft: {} commits ahead of warp", commits.len() - 1);
                }
            }
        }
    } else {
        println!("Weft: commits ahead of warp (no origin/main)");
    }

    let tangled_count = count_tangled_commits(&repo, &user)?;

    if tangled_count > 0 {
        let output = Command::new("jj")
            .args(&[
                "log",
                "-r",
                &format!("{}::", weft_head),
                "--no-graph",
                "-T",
                r#"if(conflict, description.first_line() ++ " [" ++ conflict.description() ++ "]", "")"#,
            ])
            .current_dir(repo.path())
            .output()?;

        if output.status.success() {
            let conflicts = String::from_utf8_lossy(&output.stdout);
            let conflict_lines: Vec<&str> = conflicts.lines().filter(|l| !l.trim().is_empty()).collect();

            if !conflict_lines.is_empty() {
                println!("\nTangled commits:");
                for line in &conflict_lines {
                    println!("  - {}", line);
                }
            }
        }
    }

    println!("\nRecent commits:");
    let recent_output = Command::new("jj")
        .args(&[
            "log",
            "-r",
            &format!("{}::", weft_head),
            "-n", "5",
            "--no-graph",
            "-T",
            r#"description.first_line() ++ " (" ++ commit_id ++ ")" ++ "\n""#,
        ])
        .current_dir(repo.path())
        .output()?;

    if recent_output.status.success() {
        let recent = String::from_utf8_lossy(&recent_output.stdout);
        for line in recent.lines().take(5) {
            println!("  {}", line.trim());
        }
    }

    Ok(())
}

fn count_commits_between(repo: &git2::Repository, _from: String, _to: String) -> Result<usize> {
    let output = Command::new("jj")
        .args(&["log", "-r", "::", "-T", "description.first_line()"])
        .current_dir(repo.path())
        .output()?;

    if output.status.success() {
        let log_output = String::from_utf8_lossy(&output.stdout).to_string();
        let commits: Vec<&str> = log_output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        return Ok(commits.len().saturating_sub(1));
    }

    Ok(0)
}

fn count_tangled_commits(repo: &git2::Repository, user: &str) -> Result<usize> {
    let weft_head_ref = format!("refs/weft/{}/head", user);
    let weft_head = match repo.find_reference(&weft_head_ref) {
        Ok(ref_) => ref_.peel_to_commit()?.id().to_string(),
        Err(_) => return Ok(0),
    };

    let output = Command::new("jj")
        .args(&[
            "log",
            "-r",
            &format!("{}::", weft_head),
            "-T",
            r#"if(conflict, "tangled\n", "clean\n")"#,
        ])
        .current_dir(repo.path())
        .output()?;

    if !output.status.success() {
        return Ok(0);
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let count = content.lines().filter(|l| l.trim() == "tangled").count();

    Ok(count)
}
