//! A minimal sample repository to run bonesdeploy commands against.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::session::Session;
use crate::{keep_artifacts, scratch_dir, status_ok, unique_suffix};

pub struct SampleProject {
    dir: PathBuf,
    keep: bool,
}

impl SampleProject {
    /// Expands a fixture archive into a git repository with one commit on `main`.
    pub fn from_fixture(session: &Session, fixture: &Path) -> Result<Self> {
        let dir = scratch_dir().join(format!("project-{}", unique_suffix()));
        fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
        mdpack::unpack_from_path(fixture, Some(&dir), mdpack::UnpackOptions::default())
            .map_err(|error| anyhow::anyhow!("Failed to expand fixture {}: {error}", fixture.display()))?;

        let project = Self { dir, keep: keep_artifacts() };
        project.git(session, &["init"])?;
        project.git(session, &["add", "-A"])?;
        project.git(session, &["commit", "-m", "fixture"])?;
        Ok(project)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn assert_infrastructure(&self, template: &str) -> Result<()> {
        for path in [".env", ".env.build", "infra/secrets/.env.gpg"] {
            if !self.dir.join(path).is_file() {
                bail!("{template} fixture is missing generated {path}");
            }
        }

        for path in ["infra/bonesinfra.whl", "infra/templates/shared/nginx/index.html.j2"] {
            if !self.dir.join(path).is_file() {
                bail!("{template} fixture is missing generated {path}");
            }
        }

        for entrypoint in ["__init__.py", "runtime.py", "manifest.py"] {
            let path = self.dir.join("infra/custom").join(entrypoint);
            if !path.is_file() {
                bail!("{template} fixture is missing generated {}", path.display());
            }
        }

        if !self.dir.join("infra/deployment").is_dir() {
            bail!("{template} fixture is missing generated infra/deployment");
        }
        if self.dir.join(".bones").exists() {
            bail!("{template} fixture contains obsolete .bones/");
        }
        Ok(())
    }

    pub fn pin_node_version(&self, version: &str) -> Result<()> {
        let path = self.dir.join(".env.build");
        let source = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        let updated = source.replace("NODE_VERSION=\n", &format!("NODE_VERSION={version}\n"));
        if updated == source {
            bail!(".env.build does not contain an empty NODE_VERSION in {}", self.dir.display());
        }
        fs::write(&path, updated).with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    pub fn configure_next_static(&self) -> Result<()> {
        let path = self.dir.join("next.config.ts");
        let source = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        let updated = source.replace(r#"output: "standalone""#, r#"output: "export""#);
        if updated == source {
            bail!("Next fixture does not declare output: \"standalone\" in {}", path.display());
        }
        fs::write(&path, updated).with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    pub fn push(&self, session: &Session, remote: &str, branch: &str) -> Result<()> {
        self.git(session, &["push", remote, branch])
    }

    pub fn commit(&self, session: &Session, message: &str) -> Result<()> {
        self.git(session, &["add", "-A"])?;

        let staged = session
            .command("git")
            .current_dir(&self.dir)
            .args(["diff", "--cached", "--quiet"])
            .status()
            .context("Failed to inspect staged project changes")?;
        match staged.code() {
            Some(0) => return Ok(()),
            Some(1) => {}
            _ => bail!("git diff --cached --quiet failed ({staged})"),
        }

        self.git(session, &["commit", "-m", message])
    }

    pub fn generate_laravel_app_key(&self, session: &Session, binary: &Path) -> Result<()> {
        let editor = self.dir.join(".bones-e2e-secrets-editor.sh");
        fs::write(
            &editor,
            "#!/usr/bin/env bash\nset -euo pipefail\nkey=$(openssl rand -base64 32 | tr -d '\\n')\nsed -i \"s|^APP_KEY=.*|APP_KEY=base64:$key|\" \"$1\"\n",
        )?;
        fs::set_permissions(&editor, fs::Permissions::from_mode(0o700))?;

        let editor_path = editor.to_string_lossy().into_owned();
        let status = session
            .command(binary)
            .current_dir(&self.dir)
            .args(["secrets", "edit"])
            .env("EDITOR", editor_path)
            .status()
            .with_context(|| "Failed to run bonesdeploy secrets edit")?;
        fs::remove_file(&editor).ok();
        status_ok(status, "bonesdeploy secrets edit")
    }

    /// Runs the given bonesdeploy binary in the project directory with output
    /// streaming to the terminal.
    pub fn bonesdeploy(&self, session: &Session, binary: &Path, args: &[&str]) -> Result<()> {
        let status = session
            .command(binary)
            .current_dir(&self.dir)
            .args(args)
            .status()
            .with_context(|| format!("Failed to run bonesdeploy {}", args.join(" ")))?;
        status_ok(status, &format!("bonesdeploy {}", args.join(" ")))
    }

    /// Runs the given bonesdeploy binary and captures stdout.
    pub fn bonesdeploy_output(&self, session: &Session, binary: &Path, args: &[&str]) -> Result<String> {
        let output = session
            .command(binary)
            .current_dir(&self.dir)
            .args(args)
            .output()
            .with_context(|| format!("Failed to run bonesdeploy {}", args.join(" ")))?;
        status_ok(output.status, &format!("bonesdeploy {}", args.join(" ")))?;
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    fn git(&self, session: &Session, args: &[&str]) -> Result<()> {
        let status = session
            .command("git")
            .current_dir(&self.dir)
            .args(args)
            .status()
            .with_context(|| format!("Failed to run git {}", args.join(" ")))?;
        status_ok(status, &format!("git {}", args.join(" ")))
    }
}

impl Drop for SampleProject {
    fn drop(&mut self) {
        if self.keep {
            eprintln!("{}: keeping sample project {} for inspection", crate::KEEP_ENV, self.dir.display());
            return;
        }
        if let Err(err) = fs::remove_dir_all(&self.dir) {
            eprintln!("Failed to clean up sample project {}: {err}", self.dir.display());
        }
    }
}
