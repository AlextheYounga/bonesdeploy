use anyhow::{Context, Result, bail};

use e2e::container::Container;
use e2e::project::SampleProject;
use e2e::session::Session;
use e2e::{build, image, incus};

use std::ops::Deref;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use dtor::dtor;

static HARNESS: OnceLock<Mutex<Option<Harness>>> = OnceLock::new();

pub struct HarnessRef {
    _guard: MutexGuard<'static, Option<Harness>>,
}

impl Deref for HarnessRef {
    type Target = Harness;

    fn deref(&self) -> &Harness {
        match self._guard.as_ref() {
            Some(h) => h,
            None => std::process::abort(),
        }
    }
}

#[dtor(unsafe)]
fn teardown_harness() {
    if let Some(mutex) = HARNESS.get() {
        let mut guard = mutex.lock().unwrap_or_else(|e| e.into_inner());
        drop(guard.take());
    }
}

pub fn shared_harness() -> Result<HarnessRef> {
    let mutex = HARNESS.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(Harness::create()?);
    }
    Ok(HarnessRef { _guard: guard })
}

pub struct Harness {
    artifacts: build::Artifacts,
    container: Container,
    host: String,
    session: Session,
}

impl Harness {
    pub fn create() -> Result<Self> {
        incus::check_server()?;
        let artifacts = build::artifacts()?;
        let base = image::ensure_base()?;
        let session = Session::create()?;

        let container = Container::launch(&base)?;
        container.wait_ready()?;
        container.use_slirp4netns()?;
        container.authorize_root_key(&session.public_key()?)?;
        container.wait_active("ssh")?;
        // Pre-seed the locally built bonesremote so bootstrap uses this working tree.
        container.push_file(&artifacts.bonesremote, "/usr/local/bin/bonesremote", "0755")?;
        let host = container.ipv4()?;

        Ok(Self { artifacts, container, host, session })
    }

    pub fn provision(&self, site: &str, template: &str, runtime_vars: &[&str]) -> Result<SampleProject> {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures").join(format!("{template}.md"));
        let project = SampleProject::from_fixture(&self.session, &fixture)?;
        let mut init_args = vec![
            "init",
            "--non-interactive",
            "--project-name",
            site,
            "--branch",
            "main",
            "--host",
            &self.host,
            "--template",
            template,
        ];
        for runtime_var in runtime_vars {
            init_args.extend(["--runtime-var", *runtime_var]);
        }
        project.bonesdeploy(&self.session, &self.artifacts.bonesdeploy, &init_args)?;
        project.bonesdeploy(&self.session, &self.artifacts.bonesdeploy, &["setup", "--yes"])?;
        self.assert_site(site)?;
        Ok(project)
    }

    pub fn deploy(&self, site: &str, project: &SampleProject) -> Result<()> {
        self.seed_shared_env(site, &project.read_file(".env.production")?)?;
        project.push(&self.session, "production", "main")?;
        project.bonesdeploy(&self.session, &self.artifacts.bonesdeploy, &["deploy"])
    }

    pub fn seed_shared_env(&self, site: &str, content: &str) -> Result<()> {
        let content = shell_quote(content);
        self.exec(&format!(
            "printf '%s' {content} > /srv/sites/{site}/shared/.env && chown {site}:{site} /srv/sites/{site}/shared/.env && chmod 640 /srv/sites/{site}/shared/.env"
        ))?;
        Ok(())
    }

    pub fn assert_site(&self, site: &str) -> Result<()> {
        self.exec("id git")?;
        self.exec("bonesremote version")?;
        self.exec("systemctl is-active --quiet nginx")?;
        self.exec(&format!(
            "systemctl is-active --quiet {site}.target && systemctl is-active --quiet {site}-nginx.service && test -d /srv/sites/{site}"
        ))?;
        Ok(())
    }

    pub fn assert_service(&self, service: &str) -> Result<()> {
        self.exec(&format!("systemctl is-active --quiet {service}"))?;
        Ok(())
    }

    pub fn write_laravel_probe(&self, site: &str, marker: &str) -> Result<()> {
        self.exec(&format!(
            "printf '%s\\n' '<?php error_log(\"{marker}\"); header(\"Content-Type: text/plain\"); echo \"{marker}\";' > /srv/sites/{site}/current/public/index.php"
        ))?;
        Ok(())
    }

    pub fn assert_route(&self, site: &str, expected_content: &str) -> Result<()> {
        let response = self.route_response(site)?;
        if response.contains(expected_content) {
            Ok(())
        } else {
            bail!("Route for {site} did not contain {expected_content:?}: {response}")
        }
    }

    pub fn assert_deployed(&self, site: &str) -> Result<()> {
        self.exec(&format!(
            "test \"$(readlink -f /srv/sites/{site}/current)\" != /srv/sites/{site}/releases/19700101_000000"
        ))?;
        let response = self.route_response(site)?;
        if response.contains("It's Working!") {
            bail!("Route for {site} still served the placeholder: {response}")
        }
        Ok(())
    }

    pub fn assert_owner(&self, path: &str, owner: &str) -> Result<()> {
        self.exec(&format!("test \"$(stat -c '%U:%G' {path})\" = '{owner}'"))
            .with_context(|| format!("Expected {path} to be owned by {owner}"))?;
        Ok(())
    }

    fn exec(&self, script: &str) -> Result<String> {
        self.container.exec(script)
    }

    fn route_response(&self, site: &str) -> Result<String> {
        let preview_host = format!("{}-{}.nip.io", site, self.host.replace('.', "-"));
        self.exec(&format!(
            "curl --silent --show-error --fail --max-time 10 --resolve {preview_host}:80:127.0.0.1 http://{preview_host}/"
        ))
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
