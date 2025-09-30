//! > DESCRIPTION

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

mod model;

pub use model::*;

#[allow(unused_imports)] // Not bothering matching the right features
use eyre::WrapErr;

impl TodoList {
    pub fn load(path: &std::path::Path) -> eyre::Result<Self> {
        match path.extension().and_then(std::ffi::OsStr::to_str) {
            #[cfg(feature = "yaml")]
            Some("yaml") | Some("yml") => {
                let data = std::fs::read_to_string(path)
                    .wrap_err_with(|| format!("Could not read {}", path.display()))?;

                Self::parse_yaml(&data)
                    .wrap_err_with(|| format!("Could not parse {}", path.display()))
            }
            #[cfg(feature = "json")]
            Some("json") => {
                let data = std::fs::read_to_string(path)
                    .wrap_err_with(|| format!("Could not read {}", path.display()))?;

                Self::parse_json(&data)
                    .wrap_err_with(|| format!("Could not parse {}", path.display()))
            }
            #[cfg(feature = "toml")]
            Some("toml") => {
                let data = std::fs::read_to_string(path)
                    .wrap_err_with(|| format!("Could not read {}", path.display()))?;

                Self::parse_toml(&data)
                    .wrap_err_with(|| format!("Could not parse {}", path.display()))
            }
            Some(other) => Err(eyre::eyre!("Unknown extension: {:?}", other)),
            None => Err(eyre::eyre!("No extension for {}", path.display())),
        }
    }

    pub fn save(&self, path: &std::path::Path) -> eyre::Result<()> {
        match path.extension().and_then(std::ffi::OsStr::to_str) {
            #[cfg(feature = "yaml")]
            Some("yaml") | Some("yml") => {
                let raw = self
                    .to_yaml()
                    .wrap_err_with(|| format!("Could not parse {}", path.display()))?;
                std::fs::write(path, raw)
                    .wrap_err_with(|| format!("Could not write {}", path.display()))
            }
            #[cfg(feature = "json")]
            Some("json") => {
                let raw = self
                    .to_json()
                    .wrap_err_with(|| format!("Could not parse {}", path.display()))?;
                std::fs::write(path, raw)
                    .wrap_err_with(|| format!("Could not write {}", path.display()))
            }
            #[cfg(feature = "toml")]
            Some("toml") => {
                let raw = self
                    .to_toml()
                    .wrap_err_with(|| format!("Could not parse {}", path.display()))?;
                std::fs::write(path, raw)
                    .wrap_err_with(|| format!("Could not write {}", path.display()))
            }
            Some(other) => Err(eyre::eyre!("Unknown extension: {:?}", other)),
            None => Err(eyre::eyre!("No extension for {}", path.display())),
        }
    }

    #[cfg(feature = "yaml")]
    pub fn parse_yaml(data: &str) -> eyre::Result<Self> {
        serde_yaml::from_str(data).map_err(|err| err.into())
    }

    #[cfg(feature = "json")]
    pub fn parse_json(data: &str) -> eyre::Result<Self> {
        serde_json::from_str(data).map_err(|err| err.into())
    }

    #[cfg(feature = "toml")]
    pub fn parse_toml(data: &str) -> eyre::Result<Self> {
        toml::from_str(data).map_err(|err| err.into())
    }

    #[cfg(feature = "yaml")]
    pub fn to_yaml(&self) -> eyre::Result<String> {
        serde_yaml::to_string(self).map_err(|err| err.into())
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> eyre::Result<String> {
        serde_json::to_string(self).map_err(|err| err.into())
    }

    #[cfg(feature = "toml")]
    pub fn to_toml(&self) -> eyre::Result<String> {
        toml::to_string(self).map_err(|err| err.into())
    }
}

impl TodoList {
    pub fn run(self, cwd: &std::path::Path) -> eyre::Result<()> {
        let repo = if self.init {
            git2::Repository::init(cwd)?
        } else {
            git2::Repository::open(cwd)?
        };

        let mut head = None;
        let mut last_oid = repo
            .head()
            .and_then(|h| h.resolve())
            .ok()
            .and_then(|r| r.target());
        let mut labels: std::collections::HashMap<Label, git2::Oid> = Default::default();
        for (i, event) in self.commands.iter().enumerate() {
            match event {
                Command::Label(label) => {
                    let current_oid = last_oid.ok_or_else(|| eyre::eyre!("no commits yet"))?;
                    log::trace!("label {}  # {}", label, current_oid);
                    labels.insert(label.clone(), current_oid);
                }
                Command::Reset(label) => {
                    let current_oid = *labels
                        .get(label.as_str())
                        .ok_or_else(|| eyre::eyre!("Label doesn't exist: {:?}", label))?;
                    log::trace!("reset {}  # {}", label, current_oid);
                    last_oid = Some(current_oid);
                }
                Command::Tree(tree) => {
                    let mut builder = repo.treebuilder(None)?;
                    for (relpath, content) in tree.files.iter() {
                        let relpath = path2bytes(relpath);
                        let blob_id = repo.blob(content.as_bytes())?;
                        let mode = 0o100755;
                        builder.insert(relpath, blob_id, mode)?;
                    }
                    let new_tree_oid = builder.write()?;
                    let new_tree = repo.find_tree(new_tree_oid)?;

                    let sig =
                        if let Some(author) = tree.author.as_deref().or(self.author.as_deref()) {
                            git2::Signature::now(author, "")?
                        } else {
                            repo.signature()?
                        };
                    let message = tree
                        .message
                        .clone()
                        .unwrap_or_else(|| format!("Commit (command {i})"));
                    let mut parents = Vec::new();
                    if let Some(last_oid) = last_oid {
                        parents.push(repo.find_commit(last_oid)?);
                    }
                    let parents = parents.iter().collect::<Vec<_>>();
                    let current_oid =
                        repo.commit(None, &sig, &sig, &message, &new_tree, &parents)?;
                    last_oid = Some(current_oid);

                    if let Some(sleep) = self.sleep {
                        std::thread::sleep(sleep);
                    }
                }
                Command::Merge(merge) => {
                    let ours_oid = last_oid.ok_or_else(|| eyre::eyre!("no commits yet"))?;
                    log::trace!(
                        "merge {}  # {}",
                        merge
                            .base
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(" "),
                        ours_oid
                    );
                    let mut parents = Vec::new();

                    let ours_commit = repo.find_commit(ours_oid)?;
                    let mut ours_tree_oid = ours_commit.tree_id();
                    parents.push(ours_commit);
                    for label in &merge.base {
                        let ours_tree = repo.find_tree(ours_tree_oid)?;

                        let their_oid = *labels
                            .get(label.as_str())
                            .ok_or_else(|| eyre::eyre!("Label doesn't exist: {:?}", label))?;
                        let their_commit = repo.find_commit(their_oid)?;
                        let their_tree = their_commit.tree()?;
                        parents.push(their_commit);

                        let base_oid = repo.merge_base(ours_oid, their_oid)?;
                        let base_commit = repo.find_commit(base_oid)?;
                        let base_tree = base_commit.tree()?;

                        let mut options = git2::MergeOptions::new();
                        options.find_renames(true);
                        options.fail_on_conflict(true);
                        let mut index =
                            repo.merge_trees(&base_tree, &ours_tree, &their_tree, Some(&options))?;
                        ours_tree_oid = index.write_tree()?;
                    }

                    let sig =
                        if let Some(author) = merge.author.as_deref().or(self.author.as_deref()) {
                            git2::Signature::now(author, "")?
                        } else {
                            repo.signature()?
                        };
                    let message = merge.message.clone().unwrap_or_else(|| {
                        format!(
                            "Merged {} (command {i})",
                            merge
                                .base
                                .iter()
                                .map(|s| s.as_str())
                                .collect::<Vec<_>>()
                                .join(" "),
                        )
                    });
                    let ours_tree = repo.find_tree(ours_tree_oid)?;
                    let parents = parents.iter().collect::<Vec<_>>();
                    let current_oid =
                        repo.commit(None, &sig, &sig, &message, &ours_tree, &parents)?;
                    last_oid = Some(current_oid);

                    if let Some(sleep) = self.sleep {
                        std::thread::sleep(sleep);
                    }
                }
                Command::Branch(branch) => {
                    let current_oid = last_oid.ok_or_else(|| eyre::eyre!("no commits yet"))?;
                    log::trace!("exec git branch --force {}  # {}", branch, current_oid);
                    let commit = repo.find_commit(current_oid)?;
                    repo.branch(branch.as_str(), &commit, true)?;
                }
                Command::Tag(tag) => {
                    let current_oid = last_oid.ok_or_else(|| eyre::eyre!("no commits yet"))?;
                    log::trace!("exec git tag --force -a {}  # {}", tag, current_oid);
                    let commit = repo.find_commit(current_oid)?;
                    let sig = if let Some(author) = self.author.as_deref() {
                        git2::Signature::now(author, "")?
                    } else {
                        repo.signature()?
                    };
                    let message = format!("Tag (command {i})");
                    repo.tag(tag.as_str(), commit.as_object(), &sig, &message, true)?;
                }
                Command::Head => {
                    let new_head = if let Some(branch) = self.last_branch(i) {
                        AnnotatedOid::Branch(branch)
                    } else {
                        let current_oid = last_oid.ok_or_else(|| eyre::eyre!("no commits yet"))?;
                        AnnotatedOid::Commit(current_oid)
                    };
                    log::trace!("exec git checkout {}", new_head);
                    head = Some(new_head);
                }
            }
        }

        let head = if let Some(head) = head {
            head
        } else if let Some(branch) = self.last_branch(self.commands.len()) {
            AnnotatedOid::Branch(branch)
        } else {
            let current_oid = last_oid.ok_or_else(|| eyre::eyre!("no commits yet"))?;
            AnnotatedOid::Commit(current_oid)
        };
        match head {
            AnnotatedOid::Commit(head) => {
                repo.set_head_detached(head)?;
            }
            AnnotatedOid::Branch(head) => {
                let branch = repo.find_branch(&head, git2::BranchType::Local)?;
                repo.set_head(branch.get().name().unwrap())?;
            }
        }
        repo.checkout_head(None)?;

        Ok(())
    }

    fn last_branch(&self, current_index: usize) -> Option<String> {
        if let Some(Command::Branch(prev)) = self.commands.get(current_index.saturating_sub(1)) {
            Some(prev.as_str().to_owned())
        } else {
            None
        }
    }
}

enum AnnotatedOid {
    Commit(git2::Oid),
    Branch(String),
}

impl std::fmt::Display for AnnotatedOid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Commit(ann) => ann.fmt(f),
            Self::Branch(ann) => ann.fmt(f),
        }
    }
}

#[cfg(unix)]
fn path2bytes(p: &std::path::Path) -> Vec<u8> {
    use std::os::unix::prelude::*;
    p.as_os_str().as_bytes().to_vec()
}

#[cfg(not(unix))]
fn path2bytes(p: &std::path::Path) -> Vec<u8> {
    _path2bytes_utf8(p)
}

fn _path2bytes_utf8(p: &std::path::Path) -> Vec<u8> {
    let mut v = p.as_os_str().to_str().unwrap().as_bytes().to_vec();
    for c in &mut v {
        if *c == b'\\' {
            *c = b'/';
        }
    }
    v
}
