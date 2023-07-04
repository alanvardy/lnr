pub fn get_branch() -> Result<String, String> {
    let output = std::process::Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| e.to_string())
    } else {
        Err(String::from_utf8(output.stderr)).unwrap()
    }
}
