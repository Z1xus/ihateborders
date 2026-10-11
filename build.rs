use std::process::Command;

fn main() {
    let commit = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output();
    let commit = commit
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .filter(|commit| !commit.trim().is_empty());
    println!(
        "cargo:rustc-env=COMMIT={}",
        commit.as_deref().map_or("unknown", str::trim)
    );
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
    embed_resource::compile("resources.rc", embed_resource::NONE)
        .manifest_optional()
        .unwrap();
}
