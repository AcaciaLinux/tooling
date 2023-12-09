use std::process::Command;

fn main() {
    // Use git to infer the current commit hash
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute command");

    let commit_hash = String::from_utf8_lossy(&output.stdout);
    let commit_hash = commit_hash.trim();

    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", commit_hash);
}
