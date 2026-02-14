use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProperties {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "allowed-tools")]
    pub allowed_tools: Option<String>,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub metadata: std::collections::HashMap<String, String>,
}

impl SkillProperties {
    pub fn to_dict(&self) -> serde_yaml::Value {
        let mut map = serde_yaml::Mapping::new();
        map.insert(
            serde_yaml::Value::String("name".to_string()),
            serde_yaml::Value::String(self.name.clone()),
        );
        map.insert(
            serde_yaml::Value::String("description".to_string()),
            serde_yaml::Value::String(self.description.clone()),
        );

        if let Some(ref license) = self.license {
            map.insert(
                serde_yaml::Value::String("license".to_string()),
                serde_yaml::Value::String(license.clone()),
            );
        }

        if let Some(ref compatibility) = self.compatibility {
            map.insert(
                serde_yaml::Value::String("compatibility".to_string()),
                serde_yaml::Value::String(compatibility.clone()),
            );
        }

        if let Some(ref allowed_tools) = self.allowed_tools {
            map.insert(
                serde_yaml::Value::String("allowed-tools".to_string()),
                serde_yaml::Value::String(allowed_tools.clone()),
            );
        }

        if !self.metadata.is_empty() {
            let mut meta_map = serde_yaml::Mapping::new();
            for (k, v) in &self.metadata {
                meta_map.insert(
                    serde_yaml::Value::String(k.clone()),
                    serde_yaml::Value::String(v.clone()),
                );
            }
            map.insert(
                serde_yaml::Value::String("metadata".to_string()),
                serde_yaml::Value::Mapping(meta_map),
            );
        }

        serde_yaml::Value::Mapping(map)
    }
}
