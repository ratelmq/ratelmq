use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    assert_ne!(git_hash.len(), 0);
    println!("cargo:rustc-env=BUILD_GIT_HASH={}", git_hash);
}
