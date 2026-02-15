use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Allow dead_code: used by test modules, imported via `use helpers::make_skill`
#[allow(dead_code)]
pub fn make_skill(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("SKILL.md"), content).unwrap();
    path
}
