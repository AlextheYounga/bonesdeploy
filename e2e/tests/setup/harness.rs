use anyhow::{Context, Result, bail};

use e2e::container::Container;
use e2e::project::SampleProject;
use e2e::session::Session;
use e2e::{build, image, incus};

use std::ops::Deref;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

const E2E_NODE_VERSION: &str = "24.19.0";
const NODE_TEMPLATES: &[&str] = &["django", "laravel", "next", "nuxt", "rails", "sveltekit", "vue"];

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

    pub fn provision(&self, site: &str, template: &str, framework_vars: &[&str]) -> Result<SampleProject> {
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
        for framework_var in framework_vars {
            init_args.extend(["--framework-var", *framework_var]);
        }
        if template == "next" && framework_vars.contains(&"is_static=true") {
            project.configure_next_static()?;
        }
        project.bonesdeploy(&self.session, &self.artifacts.bonesdeploy, &init_args)?;
        if template == "laravel" {
            project.generate_laravel_app_key(&self.session, &self.artifacts.bonesdeploy)?;
        }
        project.configure_remote_environment(&self.session, &self.artifacts.bonesdeploy, site, &self.host, template)?;
        project.assert_infrastructure(template)?;
        if NODE_TEMPLATES.contains(&template) {
            project.pin_node_version(E2E_NODE_VERSION)?;
        }
        project.commit(&self.session, "bonesdeploy init")?;
        project.bonesdeploy(&self.session, &self.artifacts.bonesdeploy, &["setup", "--yes"])?;
        project.bonesdeploy(&self.session, &self.artifacts.bonesdeploy, &["secrets", "push"])?;
        self.assert_site(site)?;
        let manifest = project.bonesdeploy_output(&self.session, &self.artifacts.bonesdeploy, &["manifest"])?;
        eprintln!("\n--- manifest for {site} ---\n{manifest}--- end manifest for {site} ---");
        Ok(project)
    }

    pub fn deploy(&self, project: &SampleProject) -> Result<()> {
        project.push(&self.session, "production", "main")?;
        project.bonesdeploy(&self.session, &self.artifacts.bonesdeploy, &["deploy"])
    }

    pub fn assert_site(&self, site: &str) -> Result<()> {
        self.exec("id git")?;
        self.exec("bonesremote version")?;
        self.exec("systemctl is-active --quiet nginx")?;
        self.exec(&format!(
            "systemctl is-active --quiet {site}.target && systemctl is-active --quiet {site}-nginx.service && test -d /srv/sites/{site}"
        ))?;
        self.exec(&format!(
            "test \"$(stat -c '%U:%G:%a' /usr/local/bin/bonesremote)\" = 'root:root:755' && test \"$(stat -c '%U:%G:%a' /root/.config/bonesremote/sites/{site})\" = 'root:root:700'"
        ))?;
        self.exec(&format!(
            "test -f /srv/sites/{site}/shared/.env && test -d /srv/sites/{site}/releases/19700101_000000"
        ))?;
        Ok(())
    }

    pub fn assert_service(&self, service: &str) -> Result<()> {
        self.exec(&format!("systemctl is-active --quiet {service}"))?;
        Ok(())
    }

    pub fn assert_service_condition_skipped(&self, service: &str) -> Result<()> {
        self.exec(&format!(
            "test \"$(systemctl show --property=LoadState --value -- {service})\" = loaded && \
             test \"$(systemctl show --property=ConditionResult --value -- {service})\" = no && \
             ! systemctl is-active --quiet {service}"
        ))?;
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
