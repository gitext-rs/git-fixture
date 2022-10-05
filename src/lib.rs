mod model;

pub use model::*;

use assert_cmd::output::OutputOkExt;
use bstr::ByteSlice;
use eyre::WrapErr;

impl Dag {
    pub fn load(path: &std::path::Path) -> eyre::Result<Self> {
        let data = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("Could not read {}", path.display()))?;

        let dag: Self = match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("yaml") | Some("yml") => serde_yaml::from_str(&data)
                .wrap_err_with(|| format!("Could not parse {}", path.display()))?,
            Some("json") => serde_json::from_str(&data)
                .wrap_err_with(|| format!("Could not parse {}", path.display()))?,
            Some("toml") => toml::from_str(&data)
                .wrap_err_with(|| format!("Could not parse {}", path.display()))?,
            Some(other) => {
                return Err(eyre::eyre!("Unknown extension: {:?}", other));
            }
            None => {
                return Err(eyre::eyre!("No extension for {}", path.display()));
            }
        };

        Ok(dag)
    }

    pub fn save(&self, path: &std::path::Path) -> eyre::Result<()> {
        let raw: String = match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("yaml") | Some("yml") => serde_yaml::to_string(self)
                .wrap_err_with(|| format!("Could not parse {}", path.display()))?,
            Some("json") => serde_json::to_string(self)
                .wrap_err_with(|| format!("Could not parse {}", path.display()))?,
            Some("toml") => toml::to_string(self)
                .wrap_err_with(|| format!("Could not parse {}", path.display()))?,
            Some(other) => {
                return Err(eyre::eyre!("Unknown extension: {:?}", other));
            }
            None => {
                return Err(eyre::eyre!("No extension for {}", path.display()));
            }
        };

        std::fs::write(path, &raw)
            .wrap_err_with(|| format!("Could not write {}", path.display()))?;

        Ok(())
    }

    pub fn run(self, cwd: &std::path::Path) -> eyre::Result<()> {
        if self.init {
            std::process::Command::new("git")
                .arg("init")
                .current_dir(cwd)
                .ok()
                .wrap_err("'git init' failed")?;
        }

        let mut marks: std::collections::HashMap<String, String> = Default::default();
        self.run_events(&self.events, cwd, &mut marks)?;

        Ok(())
    }

    // Note: shelling out to git to minimize programming bugs
    fn run_events(
        &self,
        events: &[Event],
        cwd: &std::path::Path,
        marks: &mut std::collections::HashMap<String, String>,
    ) -> eyre::Result<()> {
        for event in events.iter() {
            match event {
                Event::Tree(tree) => {
                    let output = std::process::Command::new("git")
                        .arg("ls-files")
                        .current_dir(cwd)
                        .ok()?;
                    for relpath in output.stdout.lines() {
                        let relpath = std::path::Path::new(relpath.to_os_str()?);
                        std::process::Command::new("git")
                            .arg("rm")
                            .arg("-f")
                            .arg(relpath)
                            .current_dir(cwd)
                            .ok()
                            .wrap_err_with(|| format!("Failed to remove {}", relpath.display()))?;
                    }
                    for (relpath, content) in tree.tracked.iter() {
                        let path = cwd.join(relpath);
                        if let Some(parent) = path.parent() {
                            std::fs::create_dir_all(parent).wrap_err_with(|| {
                                format!("Failed to create {}", parent.display())
                            })?;
                        }
                        std::fs::write(&path, content.as_bytes())
                            .wrap_err_with(|| format!("Failed to write {}", path.display()))?;
                        if !tree.state.is_tracked() {
                            std::process::Command::new("git")
                                .arg("add")
                                .arg(relpath)
                                .current_dir(cwd)
                                .ok()?;
                        }
                    }
                    if tree.state.is_committed() {
                        // Detach
                        if let Ok(pre_commit) = current_oid(cwd) {
                            checkout(cwd, &pre_commit)?;
                        }

                        let mut p = std::process::Command::new("git");
                        p.arg("commit")
                            .arg("-m")
                            .arg(tree.message.as_deref().unwrap_or("Automated"))
                            .current_dir(cwd);
                        if let Some(author) = tree.author.as_deref() {
                            p.arg("--author").arg(author);
                        }
                        p.ok()?;
                        if let Some(sleep) = self.sleep {
                            std::thread::sleep(sleep);
                        }

                        if let Some(branch) = tree.branch.as_ref() {
                            let _ = std::process::Command::new("git")
                                .arg("branch")
                                .arg("-D")
                                .arg(branch.as_str())
                                .current_dir(cwd)
                                .ok();
                            std::process::Command::new("git")
                                .arg("checkout")
                                .arg("-b")
                                .arg(branch.as_str())
                                .current_dir(cwd)
                                .ok()?;
                        }

                        if let Some(mark) = tree.mark.as_ref() {
                            let commit = current_oid(cwd)?;
                            marks.insert(mark.as_str().to_owned(), commit);
                        }
                    }
                }
                Event::Children(events) => {
                    let start_commit = current_oid(cwd)?;
                    for run in events {
                        checkout(cwd, &start_commit)?;
                        self.run_events(run, cwd, marks)?;
                    }
                }
                Event::Head(reference) => {
                    let revspec = match &reference {
                        Reference::Mark(mark) => marks
                            .get(mark.as_str())
                            .ok_or_else(|| eyre::eyre!("Reference doesn't exist: {:?}", mark))?
                            .as_str(),
                        Reference::Branch(branch) => branch.as_str(),
                    };
                    checkout(cwd, revspec)?;
                }
            }
        }

        Ok(())
    }
}

pub fn checkout(cwd: &std::path::Path, refspec: &str) -> eyre::Result<()> {
    std::process::Command::new("git")
        .arg("checkout")
        .arg(refspec)
        .current_dir(cwd)
        .ok()?;
    Ok(())
}

pub fn current_oid(cwd: &std::path::Path) -> eyre::Result<String> {
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .current_dir(cwd)
        .ok()?;
    let commit = String::from_utf8(output.stdout)?.trim().to_owned();
    Ok(commit)
}
