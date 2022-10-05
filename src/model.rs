#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct TodoList {
    #[cfg_attr(feature = "serde", serde(default = "init_default"))]
    pub init: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "humantime_serde::serialize")
    )]
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "humantime_serde::deserialize")
    )]
    pub sleep: Option<std::time::Duration>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub author: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub commands: Vec<Command>,
}

fn init_default() -> bool {
    true
}

impl Default for TodoList {
    fn default() -> Self {
        Self {
            init: init_default(),
            sleep: None,
            author: None,
            commands: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, derive_more::IsVariant)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub enum Command {
    Label(Label),
    Reset(Label),
    Tree(Tree),
    Merge(Merge),
    Branch(Branch),
    Tag(Tag),
    Head,
}

impl From<Tree> for Command {
    fn from(tree: Tree) -> Self {
        Self::Tree(tree)
    }
}

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Tree {
    pub tracked: std::collections::HashMap<std::path::PathBuf, FileContent>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub message: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub author: Option<String>,
}

#[derive(Clone, Debug, derive_more::IsVariant)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub enum FileContent {
    Binary(Vec<u8>),
    Text(String),
}

impl FileContent {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            FileContent::Binary(v) => v.as_slice(),
            FileContent::Text(v) => v.as_bytes(),
        }
    }
}

impl From<String> for FileContent {
    fn from(data: String) -> Self {
        Self::Text(data)
    }
}

impl<'d> From<&'d String> for FileContent {
    fn from(data: &'d String) -> Self {
        Self::Text(data.clone())
    }
}

impl<'d> From<&'d str> for FileContent {
    fn from(data: &'d str) -> Self {
        Self::Text(data.to_owned())
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Merge {
    pub base: Vec<Label>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub message: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub author: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Label(String);

impl Label {
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for Label {
    fn from(other: String) -> Self {
        Self(other)
    }
}

impl<'s> From<&'s str> for Label {
    fn from(other: &'s str) -> Self {
        Self(other.to_owned())
    }
}

impl std::ops::Deref for Label {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for Label {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Branch(String);

impl Branch {
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for Branch {
    fn from(other: String) -> Self {
        Self(other)
    }
}

impl<'s> From<&'s str> for Branch {
    fn from(other: &'s str) -> Self {
        Self(other.to_owned())
    }
}

impl std::ops::Deref for Branch {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for Branch {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Tag(String);

impl Tag {
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for Tag {
    fn from(other: String) -> Self {
        Self(other)
    }
}

impl<'s> From<&'s str> for Tag {
    fn from(other: &'s str) -> Self {
        Self(other.to_owned())
    }
}

impl std::ops::Deref for Tag {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for Tag {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
