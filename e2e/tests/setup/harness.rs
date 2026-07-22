use anyhow::{Context, Result, bail};

use e2e::container::Container;
use e2e::project::SampleProject;
use e2e::session::Session;
use e2e::{build, image, incus};

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
        container.authorize_root_key(&session.public_key()?)?;
        container.wait_active("ssh")?;
        // Pre-seed the locally built bonesremote so bootstrap uses this working tree.
        container.push_file(&artifacts.bonesremote, "/usr/local/bin/bonesremote", "0755")?;
        let host = container.ipv4()?;

        Ok(Self { artifacts, container, host, session })
    }

    pub fn provision(&self, site: &str, template: &str, runtime_vars: &[&str]) -> Result<()> {
        let project = SampleProject::create(&self.session)?;
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
        self.assert_site(site)
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

    pub fn write_placeholder(&self, site: &str, web_root: &str, marker: &str) -> Result<()> {
        self.exec(&format!(
            "printf '%s\\n' '{marker}' > /srv/sites/{site}/current/{web_root}/index.html"
        ))?;
        Ok(())
    }

    pub fn write_laravel_probe(&self, site: &str, marker: &str) -> Result<()> {
        self.exec(&format!(
            "printf '%s\\n' '<?php error_log(\"{marker}\"); header(\"Content-Type: text/plain\"); echo \"{marker}\";' > /srv/sites/{site}/current/public/index.php"
        ))?;
        Ok(())
    }

    pub fn assert_route(&self, site: &str, marker: &str) -> Result<()> {
        let preview_host = format!("{}-{}.nip.io", site, self.host.replace('.', "-"));
        let response = self.exec(&format!(
            "curl --silent --show-error --fail --max-time 10 --resolve {preview_host}:80:127.0.0.1 http://{preview_host}/"
        ))?;
        if response.contains(marker) {
            Ok(())
        } else {
            bail!("Route for {site} did not contain {marker:?}: {response}")
        }
    }

    pub fn assert_owner(&self, path: &str, owner: &str) -> Result<()> {
        self.exec(&format!("test \"$(stat -c '%U:%G' {path})\" = '{owner}'"))
            .with_context(|| format!("Expected {path} to be owned by {owner}"))?;
        Ok(())
    }

    fn exec(&self, script: &str) -> Result<String> {
        self.container.exec(script)
    }
}
