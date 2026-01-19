use crate::git;
use crate::config;
use anyhow::Result;
use std::process::Command;

pub fn run() -> Result<()> {
    let repo = git::discover()?;
    let user = config::get_user(&repo)?;

    let weft_head_ref = format!("refs/weft/{}/head", user);
    let weft_head = match repo.find_reference(&weft_head_ref) {
        Ok(ref_) => ref_.peel_to_commit()?.id(),
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Weft not initialized. Run 'weft init' first."
            ));
        }
    };

    let main_oid = git::get_main(&repo)?;
    let origin_main_oid = git::get_origin_main(&repo).ok();

    let target_oid = origin_main_oid.unwrap_or(main_oid);

    let output = Command::new("jj")
        .args(&[
            "rebase",
            "-d",
            &target_oid.to_string(),
            "-r",
            &format!("{}::", weft_head.to_string()),
        ])
        .current_dir(repo.path())
        .output()?;

    if !output.status.success() {
        println!("Sync encountered issues: {}", String::from_utf8_lossy(&output.stderr));
    }

    let tangled_count = count_tangled_commits(&repo, &user)?;

    let new_head = match repo.find_reference(&weft_head_ref) {
        Ok(ref_) => ref_.peel_to_commit()?.id(),
        Err(_) => weft_head,
    };

    git::update_weft_head(&repo, &user, new_head, "weft sync")?;

    if tangled_count > 0 {
        println!("Synced. {} tangled commits.", tangled_count);
    } else {
        println!("Synced. No conflicts.");
    }

    Ok(())
}

fn count_tangled_commits(repo: &git2::Repository, user: &str) -> Result<usize> {
    let weft_head_ref = format!("refs/weft/{}/head", user);
    let weft_head = match repo.find_reference(&weft_head_ref) {
        Ok(ref_) => ref_.peel_to_commit()?.id(),
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
