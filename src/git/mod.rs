use anyhow::{Context, Result};
use git2::{Oid, Repository};

pub fn discover() -> Result<Repository> {
    Repository::discover(".")
        .context("Not in a git repository. Run 'git init' first or cd into a git repo.")
}

pub fn get_head(repo: &Repository) -> Result<Oid> {
    let head = repo.head()?.peel_to_commit()?.id();
    Ok(head)
}

pub fn update_weft_head(repo: &Repository, user: &str, oid: Oid, msg: &str) -> Result<()> {
    let ref_name = format!("refs/weft/{}/head", user);
    repo.reference(&ref_name, oid, true, msg)
        .context("Failed to update weft head")?;
    Ok(())
}

pub fn get_op_log(repo: &Repository) -> Result<Option<String>> {
    match repo.find_reference("refs/weft/op-log") {
        Ok(ref_) => {
            let blob = ref_.peel_to_blob()?;
            let content = blob.content();
            Ok(Some(String::from_utf8_lossy(content).to_string()))
        }
        Err(_) => Ok(None),
    }
}

pub fn update_op_log(repo: &Repository, new_entry: &str) -> Result<()> {
    let current = get_op_log(repo)?.unwrap_or_default();
    let mut new_content = current;
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(new_entry);
    new_content.push('\n');

    let blob = repo.blob(new_content.as_bytes())?;
    repo.reference("refs/weft/op-log", blob, true, "append to op-log")?;
    Ok(())
}

pub fn get_origin_main(repo: &Repository) -> Result<Oid> {
    let ref_name = "refs/remotes/origin/main";
    let ref_ = repo.find_reference(ref_name)?;
    Ok(ref_.peel_to_commit()?.id())
}

pub fn get_main(repo: &Repository) -> Result<Oid> {
    let ref_names = ["refs/heads/main", "refs/heads/master"];
    for ref_name in &ref_names {
        if let Ok(ref_) = repo.find_reference(ref_name) {
            return Ok(ref_.peel_to_commit()?.id());
        }
    }

    let head = get_head(repo)?;
    repo.reference("refs/heads/main", head, true, "create main branch")?;
    Ok(head)
}
