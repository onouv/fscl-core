use crate::core::domain::{IdFormat, ProjectId};

#[derive(Debug, Clone)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub resource_id_format: IdFormat,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ProjectError {
    #[error("Project name cannot be empty")]
    EmptyName,
}

impl Project {
    pub fn new(
        id: ProjectId,
        name: String,
        description: Option<String>,
        resource_id_format: IdFormat,
    ) -> Result<Self, ProjectError> {
        if name.is_empty() {
            return Err(ProjectError::EmptyName);
        }
        Ok(Self {
            id,
            name,
            description,
            resource_id_format,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::{IdFormat, ProjectId};

    fn project_id() -> ProjectId {
        ProjectId::new("proj-1".to_string()).unwrap()
    }

    fn format() -> IdFormat {
        IdFormat::new(Some("=".to_string()), Some("-".to_string()), Some(4)).unwrap()
    }

    #[test]
    fn can_create_project() {
        let project =
            Project::new(project_id(), "Main Project".to_string(), None, format()).unwrap();
        assert_eq!(project.name, "Main Project");
        assert!(project.description.is_none());
    }

    #[test]
    fn rejects_empty_name() {
        let result = Project::new(project_id(), String::new(), None, format());
        assert!(matches!(result, Err(ProjectError::EmptyName)));
    }
}
