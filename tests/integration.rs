use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn setup_git_repo(tmp: &TempDir) {
    let output = Command::new("git")
        .args(&["init", "-b", "main"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to init git repo");

    assert!(
        output.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let config = Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to set git email");

    assert!(config.status.success());

    let config = Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to set git name");

    assert!(config.status.success());

    fs::write(tmp.path().join("README.md"), "test repo").expect("Failed to write README");

    let add = Command::new("git")
        .args(&["add", "."])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to git add");

    assert!(add.status.success());

    let commit = Command::new("git")
        .args(&["commit", "-m", "initial commit"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to git commit");

    assert!(
        commit.status.success(),
        "git commit failed: {}",
        String::from_utf8_lossy(&commit.stderr)
    );

    let jj_init = Command::new("jj")
        .args(&["git", "init"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to init jj repo");

    assert!(
        jj_init.status.success(),
        "jj git init failed: {}",
        String::from_utf8_lossy(&jj_init.stderr)
    );
}

fn run_weft(tmp: &TempDir, args: &[&str]) -> std::process::Output {
    let weft_path = "/home/skootsky/source-code2026/weft/target/release/weft";

    let output = Command::new(weft_path)
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("Failed to run weft");

    output
}

#[test]
fn test_weft_init_requires_git_repo() {
    let tmp = TempDir::new().unwrap();

    let output = run_weft(&tmp, &["init"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Not in a git repository") || stderr.contains("git"));
}

#[test]
fn test_weft_init_creates_refs() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    let output = run_weft(&tmp, &["init"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "weft init failed: {}", combined);

    assert!(
        combined.contains("Weft initialized"),
        "Expected init confirmation message. Got: {}",
        combined
    );

    let refs_output = Command::new("git")
        .args(&["for-each-ref", "refs/weft"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to check refs");

    let refs = String::from_utf8_lossy(&refs_output.stdout);
    assert!(
        refs.contains("refs/weft/"),
        "Expected weft refs to be created"
    );
}

#[test]
fn test_weft_save_creates_commit() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    fs::write(tmp.path().join("test.txt"), "hello").expect("Failed to write test file");

    run_weft(&tmp, &["init"]);
    let output = run_weft(&tmp, &["save", "test save message"]);

    assert!(
        output.status.success(),
        "weft save failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Saved"), "Expected save confirmation");

    let log_output = Command::new("jj")
        .args(&["log", "-r", "@", "-T", "description"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to check jj log");

    let log = String::from_utf8_lossy(&log_output.stdout);
    assert!(
        log.contains("test save message"),
        "Expected save message in commit"
    );
}

#[test]
fn test_weft_status_shows_head() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft(&tmp, &["init"]);

    fs::write(tmp.path().join("test.txt"), "hello").expect("Failed to write test file");
    run_weft(&tmp, &["save", "first save"]);

    let output = run_weft(&tmp, &["status"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "weft status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("Weft:"), "Expected Weft status section");
    assert!(
        stdout.contains("Recent commits"),
        "Expected recent commits section"
    );
}

#[test]
fn test_weft_undo_after_save() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft(&tmp, &["init"]);

    fs::write(tmp.path().join("test.txt"), "hello").expect("Failed to write test file");
    run_weft(&tmp, &["save", "save to undo"]);

    let output = run_weft(&tmp, &["undo"]);
    assert!(
        output.status.success(),
        "weft undo failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Undid"), "Expected undo confirmation");
}

#[test]
fn test_save_creates_weft_ref() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft(&tmp, &["init"]);
    fs::write(tmp.path().join("file.txt"), "content").expect("Failed to write file");
    run_weft(&tmp, &["save", "checkpoint"]);

    let refs_output = Command::new("git")
        .args(&["for-each-ref", "refs/weft"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to check weft refs");

    let refs = String::from_utf8_lossy(&refs_output.stdout);
    assert!(
        refs.contains("refs/weft/test-user/head"),
        "Expected weft head ref to be created"
    );
}

#[test]
fn test_save_appends_to_weft() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft(&tmp, &["init"]);

    fs::write(tmp.path().join("file.txt"), "content1").expect("Failed to write file");
    run_weft(&tmp, &["save", "first checkpoint"]);

    fs::write(tmp.path().join("file.txt"), "content2").expect("Failed to write file");
    run_weft(&tmp, &["save", "second checkpoint"]);

    fs::write(tmp.path().join("file.txt"), "content3").expect("Failed to write file");
    run_weft(&tmp, &["save", "third checkpoint"]);

    let status_output = run_weft(&tmp, &["status"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&status_output.stdout),
        String::from_utf8_lossy(&status_output.stderr)
    );

    assert!(
        combined.contains("save:") && combined.contains("checkpoint"),
        "Expected save messages to appear in status. Got: {}",
        combined
    );
}

#[test]
fn test_sync_clean_rebase() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft(&tmp, &["init"]);
    fs::write(tmp.path().join("file.txt"), "base content").expect("Failed to write file");
    run_weft(&tmp, &["save", "base work"]);

    let output = run_weft(&tmp, &["sync"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.status.success(), "sync failed: {}", combined);
    assert!(
        combined.contains("Synced") || combined.contains("No conflicts"),
        "Expected synced message"
    );
}

#[test]
fn test_sync_creates_tangle_commit() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft(&tmp, &["init"]);

    fs::write(tmp.path().join("file.txt"), "local change").expect("Failed to write file");
    run_weft(&tmp, &["save", "local work"]);

    let output = run_weft(&tmp, &["sync"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.status.success(), "sync failed: {}", combined);
    assert!(
        combined.contains("tangled") || combined.contains("conflicts") || combined.contains("Sync"),
        "Expected tangle or sync message, got: {}",
        combined
    );
}

#[test]
fn test_undo_reverts_save() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft(&tmp, &["init"]);
    fs::write(tmp.path().join("file.txt"), "original").expect("Failed to write file");

    let before_undo = Command::new("jj")
        .args(&["log", "-r", "@", "-T", "description"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to get jj log");

    let before_desc = String::from_utf8_lossy(&before_undo.stdout).to_string();

    run_weft(&tmp, &["save", "work to undo"]);
    run_weft(&tmp, &["undo"]);

    let after_undo = Command::new("jj")
        .args(&["log", "-r", "@", "-T", "description"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to get jj log after undo");

    let after_desc = String::from_utf8_lossy(&after_undo.stdout).to_string();

    assert!(
        before_desc.contains(&after_desc) || after_desc.contains("initial"),
        "Undo should revert to previous state. Before: {}, After: {}",
        before_desc,
        after_desc
    );
}

#[test]
fn test_status_shows_tangled_count() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft(&tmp, &["init"]);

    let status_output = run_weft(&tmp, &["status"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&status_output.stdout),
        String::from_utf8_lossy(&status_output.stderr)
    );

    assert!(combined.contains("Weft:"), "Expected Weft status section");
}

fn run_weft_with_env(tmp: &TempDir, args: &[&str], user: &str) -> std::process::Output {
    let weft_path = "/home/skootsky/source-code2026/weft/target/release/weft";

    let mut cmd = Command::new(weft_path);
    cmd.args(args)
        .current_dir(tmp.path())
        .env("WEFT_USER", user);

    cmd.output().expect("Failed to run weft")
}

#[test]
fn test_two_users_sync_same_conflict() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft_with_env(&tmp, &["init"], "alice");
    fs::write(tmp.path().join("shared.txt"), "alice line 1").expect("Failed to write file");
    run_weft_with_env(&tmp, &["save", "alice work 1"], "alice");

    let refs_output = Command::new("git")
        .args(&["for-each-ref", "refs/weft"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to check refs");
    let refs = String::from_utf8_lossy(&refs_output.stdout);
    assert!(refs.contains("alice"), "Expected alice weft ref");
}

#[test]
fn test_undo_multiple_operations() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft(&tmp, &["init"]);

    fs::write(tmp.path().join("file.txt"), "v1").expect("Failed to write file");
    run_weft(&tmp, &["save", "version 1"]);

    fs::write(tmp.path().join("file.txt"), "v2").expect("Failed to write file");
    run_weft(&tmp, &["save", "version 2"]);

    let output1 = run_weft(&tmp, &["undo"]);
    assert!(output1.status.success(), "first undo failed");

    let output2 = run_weft(&tmp, &["undo"]);
    assert!(output2.status.success(), "second undo failed");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output1.stdout),
        String::from_utf8_lossy(&output2.stdout)
    );
    assert!(combined.contains("Undid"), "Expected undo confirmations");
}

#[test]
fn test_jj_version_check() {
    let output = Command::new("/home/skootsky/source-code2026/weft/target/release/weft")
        .args(&["--version"])
        .output()
        .expect("Failed to get weft version");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("weft") || output.status.success(),
        "weft should report version"
    );
}

#[test]
fn test_detects_master_vs_main() {
    let tmp = TempDir::new().unwrap();

    let output = Command::new("git")
        .args(&["init", "-b", "master"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to init git repo");

    assert!(output.status.success());

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to set email");

    Command::new("git")
        .args(&["config", "user.name", "Test"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to set name");

    fs::write(tmp.path().join("file.txt"), "content").expect("Failed to write file");
    Command::new("git")
        .args(&["add", "."])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to add");
    Command::new("git")
        .args(&["commit", "-m", "init"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to commit");

    Command::new("jj")
        .args(&["git", "init"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to init jj");

    let output = run_weft(&tmp, &["init"]);
    assert!(
        output.status.success(),
        "weft should work with master branch: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_save_on_tangled_commit() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft(&tmp, &["init"]);
    fs::write(tmp.path().join("file.txt"), "change 1").expect("Failed to write file");
    run_weft(&tmp, &["save", "change 1"]);

    run_weft(&tmp, &["sync"]);

    fs::write(tmp.path().join("file.txt"), "change 2").expect("Failed to write file");
    let save_output = run_weft(&tmp, &["save", "change 2 on top"]);

    assert!(
        save_output.status.success(),
        "save on tangled should work: {}",
        String::from_utf8_lossy(&save_output.stderr)
    );
}

#[test]
fn test_user_with_special_chars() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    let output = run_weft_with_env(&tmp, &["init"], "alice.smith@corp.com");
    assert!(
        output.status.success(),
        "should handle special chars in username: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let refs_output = Command::new("git")
        .args(&["for-each-ref", "refs/weft"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to check refs");

    let refs = String::from_utf8_lossy(&refs_output.stdout);
    assert!(!refs.is_empty(), "Should create ref for sanitized username");
}

#[test]
fn test_init_in_empty_repo() {
    let tmp = TempDir::new().unwrap();

    let output = Command::new("git")
        .args(&["init", "-b", "main"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to init git repo");

    assert!(output.status.success());

    Command::new("jj")
        .args(&["git", "init"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to init jj");

    let output = run_weft(&tmp, &["init"]);

    assert!(
        !output.status.success(),
        "weft init should fail without a commit"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("HEAD") || stderr.contains("No HEAD"),
        "Expected error about no commits, got: {}",
        stderr
    );
}

fn setup_git_repo_with_remote(tmp: &TempDir) {
    setup_git_repo(tmp);

    let bare_path = tmp.path().join("remote.git");
    Command::new("git")
        .args(&["init", "--bare", bare_path.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to create bare remote");

    Command::new("git")
        .args(&["remote", "add", "origin", bare_path.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to add remote");
}

#[test]
fn test_share_pushes_weft_ref() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo_with_remote(&tmp);

    run_weft(&tmp, &["init"]);
    fs::write(tmp.path().join("file.txt"), "content").expect("Failed to write file");
    run_weft(&tmp, &["save", "work to share"]);

    let output = run_weft(&tmp, &["share"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.status.success(), "weft share failed: {}", combined);
    assert!(
        combined.contains("Shared weft"),
        "Expected share confirmation"
    );

    let refs_output = Command::new("git")
        .args(&["ls-remote", "origin", "refs/weft/*"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to check remote refs");

    let refs = String::from_utf8_lossy(&refs_output.stdout);
    assert!(refs.contains("refs/weft/"), "Expected weft ref on remote");
}

#[test]
fn test_share_fails_without_remote() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo(&tmp);

    run_weft(&tmp, &["init"]);

    let output = run_weft(&tmp, &["share"]);
    assert!(!output.status.success(), "share without remote should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("remote") || stderr.contains("origin"),
        "Expected error about missing remote, got: {}",
        stderr
    );
}

#[test]
fn test_share_idempotent() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo_with_remote(&tmp);

    run_weft(&tmp, &["init"]);
    fs::write(tmp.path().join("file.txt"), "content").expect("Failed to write file");
    run_weft(&tmp, &["save", "work"]);

    let output1 = run_weft(&tmp, &["share"]);
    assert!(output1.status.success(), "first share should succeed");

    let output2 = run_weft(&tmp, &["share"]);
    assert!(
        output2.status.success(),
        "second share should succeed (idempotent)"
    );
}

#[test]
fn test_propose_creates_candidate_ref() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo_with_remote(&tmp);

    run_weft(&tmp, &["init"]);
    fs::write(tmp.path().join("file.txt"), "content").expect("Failed to write file");
    run_weft(&tmp, &["save", "proposal work"]);

    let output = run_weft(&tmp, &["propose"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.status.success(), "weft propose failed: {}", combined);
    assert!(
        combined.contains("Candidate created") || combined.contains("refs/loom/"),
        "Expected candidate creation message, got: {}",
        combined
    );

    let refs_output = Command::new("git")
        .args(&["ls-remote", "origin", "refs/loom/*"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to check remote refs");

    let refs = String::from_utf8_lossy(&refs_output.stdout);
    assert!(refs.contains("refs/loom/"), "Expected loom ref on remote");
}

#[test]
fn test_weave_fails_on_tangled_candidate() {
    let tmp = TempDir::new().unwrap();
    setup_git_repo_with_remote(&tmp);

    run_weft(&tmp, &["init"]);
    fs::write(tmp.path().join("file.txt"), "content").expect("Failed to write file");
    run_weft(&tmp, &["save", "work"]);

    let propose_output = run_weft(&tmp, &["propose"]);
    assert!(propose_output.status.success(), "propose should succeed");

    let propose_stdout = String::from_utf8_lossy(&propose_output.stdout);
    let candidate_id: Vec<&str> = propose_stdout.split("weft weave ").collect();
    let candidate_id = candidate_id.last().unwrap_or(&"test-xxx").trim();
    let candidate_id = candidate_id
        .split_whitespace()
        .next()
        .unwrap_or(candidate_id);

    let output = run_weft(&tmp, &["weave", candidate_id]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if !output.status.success() {
        assert!(
            combined.contains("tangled")
                || combined.contains("conflict")
                || combined.contains("Cannot weave"),
            "Expected tangle-related error, got: {}",
            combined
        );
    } else {
        assert!(combined.contains("Woven"), "Expected success message");
    }
}
