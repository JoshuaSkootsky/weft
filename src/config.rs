use anyhow::Result;
use git2::Repository;
use std::env;

pub fn get_user(repo: &Repository) -> Result<String> {
    if let Ok(user) = env::var("WEFT_USER") {
        if !user.is_empty() {
            return Ok(sanitize_username(&user));
        }
    }

    if let Ok(config) = repo.config() {
        if let Ok(name) = config.get_string("user.weft-username") {
            return Ok(sanitize_username(&name));
        }
        if let Ok(name) = config.get_string("user.name") {
            return Ok(sanitize_username(&name));
        }
    }

    let username = whoami::username();
    if !username.is_empty() {
        return Ok(sanitize_username(&username));
    }

    Err(anyhow::anyhow!(
        "Cannot determine user identity. Set WEFT_USER env var or configure git user.name"
    ))
}

fn sanitize_username(name: &str) -> String {
    name.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
        .to_lowercase()
}
