mod helpers;

#[test]
fn test_skill_properties_to_dict() {
    let props = skills_validator::models::SkillProperties {
        name: "test".to_string(),
        description: "Test description".to_string(),
        license: Some("MIT".to_string()),
        compatibility: None,
        allowed_tools: None,
        metadata: std::collections::HashMap::new(),
    };
    let dict = props.to_dict();
    assert!(dict.is_mapping());
}

#[test]
fn test_skill_properties_to_dict_with_metadata() {
    use std::collections::HashMap;
    let mut metadata = HashMap::new();
    metadata.insert("key".to_string(), "value".to_string());
    let props = skills_validator::models::SkillProperties {
        name: "test".to_string(),
        description: "Test description".to_string(),
        license: None,
        compatibility: None,
        allowed_tools: None,
        metadata,
    };
    let dict = props.to_dict();
    let map = dict.as_mapping().unwrap();
    assert!(map.contains_key(&serde_yaml::Value::String("metadata".to_string())));
}
