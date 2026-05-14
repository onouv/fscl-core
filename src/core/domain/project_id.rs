use serde::{Deserialize, Serialize};
use std::fmt;

/// Global project identifier. Projects are the top-level organizational unit
/// for FSCL resource hierarchies. Each project has its own IdFormat definition
/// and all resources within a project are identified using that format.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProjectId(String);

impl ProjectId {
    pub fn new(id: String) -> Result<Self, ProjectIdError> {
        if id.is_empty() {
            return Err(ProjectIdError::Empty);
        }
        Ok(ProjectId(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ProjectIdError {
    #[error("Project ID cannot be empty")]
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_project_id() {
        let pid = ProjectId::new("proj-1".to_string()).unwrap();
        assert_eq!(pid.as_str(), "proj-1");
    }

    #[test]
    fn rejects_empty_project_id() {
        let result = ProjectId::new("".to_string());
        assert!(matches!(result, Err(ProjectIdError::Empty)));
    }
}
