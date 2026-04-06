use std::collections::HashMap;
use std::path::Path;

use crate::error::SkillError;
use crate::models::SkillProperties;

pub fn find_skill_md(skill_dir: &Path) -> Option<std::path::PathBuf> {
    let path = skill_dir.join("SKILL.md");
    if path.exists() {
        return Some(path);
    }
    None
}

pub fn parse_frontmatter(content: &str) -> Result<(serde_yaml::Value, String), SkillError> {
    if !content.starts_with("---") {
        return Err(SkillError::ParseError(
            "SKILL.md must start with YAML frontmatter (---)".to_string(),
        ));
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(SkillError::ParseError(
            "SKILL.md frontmatter not properly closed with ---".to_string(),
        ));
    }

    let frontmatter_str = parts[1];
    let body = parts[2].trim().to_string();

    let metadata: serde_yaml::Value = serde_yaml::from_str(frontmatter_str)?;

    if !metadata.is_mapping() {
        return Err(SkillError::ParseError(
            "SKILL.md frontmatter must be a YAML mapping".to_string(),
        ));
    }

    let mut metadata = metadata;
    if let Some(meta_map) = metadata.get_mut("metadata") {
        if let Some(obj) = meta_map.as_mapping_mut() {
            let mut new_map = serde_yaml::Mapping::new();
            for (k, v) in obj.iter() {
                let key = k.as_str().unwrap_or_default().to_string();
                let value = v.as_str().unwrap_or_default().to_string();
                new_map.insert(
                    serde_yaml::Value::String(key),
                    serde_yaml::Value::String(value),
                );
            }
            *meta_map = serde_yaml::Value::Mapping(new_map);
        }
    }

    Ok((metadata, body))
}

pub fn parse_frontmatter_and_body(
    content: &str,
) -> Result<(serde_yaml::Mapping, String), SkillError> {
    let (metadata, body) = parse_frontmatter(content)?;
    let map = metadata
        .as_mapping()
        .cloned()
        .ok_or_else(|| SkillError::ParseError("Invalid metadata format".to_string()))?;
    Ok((map, body))
}

pub fn read_properties(skill_dir: &Path) -> Result<SkillProperties, SkillError> {
    let skill_md = find_skill_md(skill_dir)
        .ok_or_else(|| SkillError::ParseError(format!("SKILL.md not found in {:?}", skill_dir)))?;

    let content = std::fs::read_to_string(&skill_md)?;
    let (metadata, _) = parse_frontmatter(&content)?;

    let map = metadata
        .as_mapping()
        .ok_or_else(|| SkillError::ParseError("Invalid metadata format".to_string()))?;

    fn get_string(map: &serde_yaml::Mapping, key: &str) -> Result<String, SkillError> {
        map.get(serde_yaml::Value::String(key.to_string()))
            .map(|v| v.as_str().unwrap_or_default().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                SkillError::ValidationError(format!(
                    "Missing required field in frontmatter: {}",
                    key
                ))
            })
    }

    fn get_optional_string(map: &serde_yaml::Mapping, key: &str) -> Option<String> {
        map.get(serde_yaml::Value::String(key.to_string()))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }

    let name = get_string(map, "name")?;
    let description = get_string(map, "description")?;

    if name.trim().is_empty() {
        return Err(SkillError::ValidationError(
            "Field 'name' must be a non-empty string".to_string(),
        ));
    }
    if description.trim().is_empty() {
        return Err(SkillError::ValidationError(
            "Field 'description' must be a non-empty string".to_string(),
        ));
    }

    let license = get_optional_string(map, "license");
    let compatibility = get_optional_string(map, "compatibility");
    let allowed_tools = get_optional_string(map, "allowed-tools");

    let metadata: HashMap<String, String> = map
        .get(serde_yaml::Value::String("metadata".to_string()))
        .and_then(|m| m.as_mapping())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default();

    Ok(SkillProperties {
        name: name.trim().to_string(),
        description: description.trim().to_string(),
        license,
        compatibility,
        allowed_tools,
        metadata,
    })
}
