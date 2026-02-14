use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn make_skill(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("SKILL.md"), content).unwrap();
    path
}
