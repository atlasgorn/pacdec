use std::fmt;

use kdl::{KdlNode, KdlValue};

/// Type used to represent a package, its repo and tags if any. Note that tags are not considered when comparing packages, they are mostly metadata.
#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub repository: Option<String>,
    pub tags: Vec<String>,
}

impl Package {
    pub fn from_str(s: &str) -> Self {
        let parts: Vec<&str> = s.split('/').collect();
        let name = parts.last().unwrap_or(&s).to_string();
        let repository = if parts.len() > 1 {
            Some(parts[..parts.len() - 1].join("/"))
        } else {
            None
        };

        Package {
            name,
            repository,
            tags: Vec::new(),
        }
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(repo) = &self.repository {
            write!(f, "{}/{}", repo, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl From<String> for Package {
    fn from(s: String) -> Self {
        Self::from_str(&s)
    }
}

impl From<&str> for Package {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<Package> for KdlNode {
    fn from(pkg: Package) -> Self {
        let mut node = KdlNode::new(pkg.to_string());

        for tag in pkg.tags {
            node.push(KdlValue::String(tag));
        }

        node
    }
}

impl TryFrom<KdlNode> for Package {
    type Error = &'static str;

    fn try_from(node: KdlNode) -> Result<Self, Self::Error> {
        let mut pkg = Package::from_str(node.name().value());

        pkg.tags = node
            .entries()
            .iter()
            .filter_map(|entry| {
                if let KdlValue::String(tag) = entry.value() {
                    Some(tag.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(pkg)
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        if self.repository.is_none() || other.repository.is_none() {
            self.name == other.name
        } else {
            self.name == other.name && self.repository == other.repository
        }
    }
}

impl Eq for Package {}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.name.cmp(&other.name) {
            std::cmp::Ordering::Equal => self.repository.cmp(&other.repository),
            ordering => ordering,
        }
    }
}

impl std::hash::Hash for Package {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.repository.hash(state);
    }
}

#[derive(Hash, Eq, Ord, PartialOrd, PartialEq, Debug, Clone)]
pub struct Category {
    pub name: String,
    pub path: Vec<String>,
}

impl Category {
    pub fn from_str(s: &str) -> Self {
        let parts: Vec<&str> = s.split('/').collect();
        let name = parts.last().unwrap_or(&s).to_string();
        let path = parts[..parts.len().saturating_sub(1)]
            .iter()
            .map(|s| s.to_string())
            .collect();
        Category { name, path }
    }

    pub fn full_path(&self) -> String {
        if self.path.is_empty() {
            self.name.clone()
        } else {
            format!("{}/{}", self.path.join("/"), self.name)
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_path())
    }
}

impl From<String> for Category {
    fn from(s: String) -> Self {
        Self::from_str(&s)
    }
}

impl From<&str> for Category {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}
