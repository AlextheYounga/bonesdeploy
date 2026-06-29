use anyhow::Result;

pub fn post_receive(site: &str) -> Result<()> {
    super::deploy::run_full(site, None)
}
