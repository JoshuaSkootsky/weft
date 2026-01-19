use crate::git;
use crate::config;
use anyhow::Result;

pub fn run() -> Result<()> {
    let repo = git::discover()?;
    let user = config::get_user(&repo)?;

    let head = repo.head()?.peel_to_commit()?.id();

    let weft_head_ref = format!("refs/weft/{}/head", user);
    match repo.find_reference(&weft_head_ref) {
        Ok(_) => {
            println!("Weft already initialized for user '{}'", user);
            return Ok(());
        }
        Err(_) => {}
    }

    repo.reference(&weft_head_ref, head, true, "weft init")?;

    match repo.find_reference("refs/weft/op-log") {
        Ok(_) => {}
        Err(_) => {
            let empty_content = b"";
            let blob = repo.blob(empty_content)?;
            repo.reference("refs/weft/op-log", blob, true, "init op-log")?;
        }
    }

    println!("Weft initialized for user '{}'", user);
    println!("Your weft head is at: refs/weft/{}/head", user);
    println!("\nNext steps:");
    println!("  weft save \"checkpoint message\"");
    println!("  weft sync");

    Ok(())
}
