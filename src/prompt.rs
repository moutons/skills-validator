use std::path::Path;

use crate::parser::{find_skill_md, read_properties};

pub fn to_prompt(skill_dirs: &[&str]) -> String {
    if skill_dirs.is_empty() {
        return "<available_skills>\n</available_skills>".to_string();
    }

    let mut lines = vec!["<available_skills>".to_string()];

    for skill_dir in skill_dirs {
        let path = Path::new(skill_dir);
        let _resolved = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        let props = match read_properties(path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Warning: Failed to read properties for {:?}: {}", path, e);
                continue;
            }
        };

        lines.push("<skill>".to_string());
        lines.push("<name>".to_string());
        lines.push(html_escape::encode_text(&props.name).to_string());
        lines.push("</name>".to_string());
        lines.push("<description>".to_string());
        lines.push(html_escape::encode_text(&props.description).to_string());
        lines.push("</description>".to_string());

        if let Some(skill_md_path) = find_skill_md(path) {
            lines.push("<location>".to_string());
            lines.push(skill_md_path.to_string_lossy().to_string());
            lines.push("</location>".to_string());
        }

        lines.push("</skill>".to_string());
    }

    lines.push("</available_skills>".to_string());

    lines.join("\n")
}
