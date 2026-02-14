use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillError {
    #[error("Failed to parse SKILL.md: {0}")]
    ParseError(String),

    #[error("Skill validation failed: {0}")]
    ValidationError(String),
}

impl From<std::io::Error> for SkillError {
    fn from(err: std::io::Error) -> Self {
        SkillError::ParseError(err.to_string())
    }
}

impl From<serde_yaml::Error> for SkillError {
    fn from(err: serde_yaml::Error) -> Self {
        SkillError::ParseError(err.to_string())
    }
}
