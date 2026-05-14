use crate::core::domain::ProjectId;
use std::fmt;

use serde::Serialize;
use thiserror::Error;

static MAX_BLOCK_LENGTH: usize = 8;
static MAX_PREFIX_LENGTH: usize = 4;
static MAX_SEPARATOR_LENGTH: usize = 4;

/// The idiomatic identifier type for FSCL resources.
/// Composite value object: (project_id, local_id, format).
/// All resources are identified within the context of a project and validated by that project's IdFormat.
///
/// Usage:
/// ```
/// //let project_id = ProjectId::new("proj-1".to_string()).unwrap();
/// //let format = IdFormat::new(Some("=".to_string()), Some("-".to_string()), Some(4)).unwrap();
/// //let resource_id = ResourceId::new(project_id, "=-1234".to_string(), format).unwrap();
/// //assert_eq!(resource_id.local_id(), "=-1234");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ResourceId {
    project_id: ProjectId,
    local_id: String,
    format: IdFormat,
}

impl ResourceId {
    pub fn new(
        project_id: ProjectId,
        local_id: String,
        format: IdFormat,
    ) -> Result<Self, ResourceIdError> {
        if local_id.is_empty() {
            return Err(ResourceIdError::Empty);
        }

        format.validate(local_id.clone())?;

        Ok(Self {
            project_id,
            local_id,
            format,
        })
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn local_id(&self) -> &str {
        &self.local_id
    }

    pub fn format(&self) -> &IdFormat {
        &self.format
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.project_id, self.local_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct IdFormat {
    prefix: Option<String>,
    separator: Option<String>,
    block_length: Option<usize>,
}

impl IdFormat {
    pub fn new(
        prefix: Option<String>,
        separator: Option<String>,
        block_length: Option<usize>,
    ) -> Result<Self, ResourceIdError> {
        if let Some(prefix) = prefix.as_ref()
            && prefix.len() > MAX_PREFIX_LENGTH
        {
            return Err(ResourceIdError::PrefixLengthTooBig {
                expected: MAX_PREFIX_LENGTH,
                actual: prefix.len(),
            });
        }

        if let Some(separator) = separator.as_ref()
            && separator.len() > MAX_SEPARATOR_LENGTH
        {
            return Err(ResourceIdError::SeparatorLengthTooBig {
                expected: MAX_SEPARATOR_LENGTH,
                actual: separator.len(),
            });
        }

        if let Some(block_length) = block_length
            && block_length > MAX_BLOCK_LENGTH
        {
            return Err(ResourceIdError::BlockLengthTooBig {
                expected: MAX_BLOCK_LENGTH,
                actual: block_length,
            });
        }

        Ok(Self {
            prefix,
            separator,
            block_length,
        })
    }

    pub fn validate(&self, id: String) -> Result<(), ResourceIdError> {
        if id.is_empty() {
            return Err(ResourceIdError::Empty);
        }

        if let Some(prefix) = &self.prefix
            && !id.starts_with(prefix)
        {
            return Err(ResourceIdError::PrefixViolated);
        }

        // Block-length validation is defined on the identifier payload, i.e. without prefix.
        let payload = match self.prefix.as_deref() {
            Some(prefix) => id.strip_prefix(prefix).unwrap_or(id.as_str()),
            None => id.as_str(),
        };

        if let Some(block_length) = self.block_length {
            let blocks: Vec<&str> = match self.separator.as_deref() {
                Some(separator) => payload.split(separator).collect(),
                None => vec![payload],
            };

            for block in blocks {
                if block.len() > block_length {
                    return Err(ResourceIdError::BlockLengthViolated);
                }
            }
        }

        if let Some(separator) = self.separator.as_deref() {
            if id.starts_with(separator) {
                return Err(ResourceIdError::SeparatorPositionViolated {
                    separator: self.separator.clone().unwrap(),
                    position: 0,
                });
            }

            if id.ends_with(separator) {
                return Err(ResourceIdError::SeparatorPositionViolated {
                    separator: self.separator.clone().unwrap(),
                    position: id.len() - 1,
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ResourceIdError {
    #[error("Resource ID cannot be empty")]
    Empty,
    #[error("Resource ID has invalid prefix")]
    PrefixViolated,
    #[error("Resource ID has invalid block length")]
    BlockLengthViolated,
    #[error("Invalid use of separator '{separator}' in Resource ID at position '{position}'")]
    SeparatorPositionViolated { separator: String, position: usize },
    #[error("Invalid use of prefix '{prefix}' at position '{position}'")]
    PrefixPositionViolated { prefix: char, position: usize },
    #[error("Resource ID block length is too big. Expected: {expected}, Actual: {actual}")]
    BlockLengthTooBig { expected: usize, actual: usize },
    #[error("Resource ID prefix length is too big. Expected: {expected}, Actual: {actual}")]
    PrefixLengthTooBig { expected: usize, actual: usize },
    #[error("Resource ID separator length is too big. Expected: {expected}, Actual: {actual}")]
    SeparatorLengthTooBig { expected: usize, actual: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_project_id() -> ProjectId {
        ProjectId::new("test-proj".to_string()).unwrap()
    }

    #[test]
    fn can_be_created_with_prefix() {
        let project_id = test_project_id();
        let format = IdFormat::new(Some("=".to_string()), Some(".".to_string()), Some(4)).unwrap();
        let id1 = ResourceId::new(project_id.clone(), "=1234".to_string(), format.clone());
        assert!(id1.is_ok());
        assert_eq!(id1.unwrap().local_id(), "=1234");

        let id2 = ResourceId::new(project_id.clone(), "=1234.0010".to_string(), format.clone());
        assert!(id2.is_ok());
        assert_eq!(id2.unwrap().local_id(), "=1234.0010");

        let id3 = ResourceId::new(
            project_id.clone(),
            "=1234.0001.0001".to_string(),
            format.clone(),
        );
        assert!(id3.is_ok());
        assert_eq!(id3.unwrap().local_id(), "=1234.0001.0001");
    }

    #[test]
    fn accepts_id_without_prefix_when_within_block_length() {
        let project_id = test_project_id();
        let format = IdFormat::new(None, None, Some(4)).unwrap();
        let result = ResourceId::new(project_id, "1234".to_string(), format);
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_empty_id() {
        let project_id = test_project_id();
        let format = IdFormat::new(Some("=".to_string()), Some(".".to_string()), Some(4)).unwrap();
        let result = ResourceId::new(project_id, "".to_string(), format);
        assert!(matches!(result, Err(ResourceIdError::Empty)));
    }

    #[test]
    fn rejects_prefix_longer_than_max() {
        let result = IdFormat::new(Some("12345".to_string()), Some(".".to_string()), Some(4));

        assert!(matches!(
            result,
            Err(ResourceIdError::PrefixLengthTooBig {
                expected: 4,
                actual: 5,
            })
        ));
    }

    #[test]
    fn rejects_separator_longer_than_max() {
        let result = IdFormat::new(Some("=".to_string()), Some(".....".to_string()), Some(4));

        assert!(matches!(
            result,
            Err(ResourceIdError::SeparatorLengthTooBig {
                expected: 4,
                actual: 5,
            })
        ));
    }

    #[test]
    fn rejects_block_length_longer_than_max() {
        let result = IdFormat::new(Some("=".to_string()), Some(".".to_string()), Some(9));

        assert!(matches!(
            result,
            Err(ResourceIdError::BlockLengthTooBig {
                expected: 8,
                actual: 9,
            })
        ));
    }

    #[test]
    fn rejects_id_violating_prefix() {
        let project_id = test_project_id();
        let format = IdFormat::new(Some("=".to_string()), Some(".".to_string()), Some(4)).unwrap();
        let result = ResourceId::new(project_id, "1234".to_string(), format);
        assert!(matches!(result, Err(ResourceIdError::PrefixViolated)));
    }

    #[test]
    fn rejects_id_violating_block_length() {
        let project_id = test_project_id();
        let format = IdFormat::new(Some("=".to_string()), Some(".".to_string()), Some(4)).unwrap();
        let result = ResourceId::new(project_id.clone(), "=12345".to_string(), format.clone());
        assert!(matches!(result, Err(ResourceIdError::BlockLengthViolated)));

        let result2 = ResourceId::new(project_id, "=1234.00001".to_string(), format.clone());
        assert!(matches!(result2, Err(ResourceIdError::BlockLengthViolated)));
    }

    #[test]
    fn display_formats_as_project_colon_local_id() {
        let project_id = test_project_id();
        let format = IdFormat::new(Some("=".to_string()), Some(".".to_string()), Some(4)).unwrap();
        let resource_id = ResourceId::new(project_id, "=1234".to_string(), format).unwrap();
        assert_eq!(resource_id.to_string(), "test-proj:=1234");
    }
}
