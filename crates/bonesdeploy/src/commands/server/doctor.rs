use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::paths;
use console::style;

use crate::config;
use crate::infra::ssh;
use crate::ui::output;

pub async fn run(verbose: bool) -> Result<()> {
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    println!("{} Checking server baseline...", style("bonesdeploy server doctor").bold());
    let session = ssh::connect_privileged(&cfg).await?;
    let result = ssh::run_cmd(&session, "bonesremote doctor").await;
    let _ = session.close().await;
    let report = result?;
    if verbose {
        print!("{report}");
        if !report.is_empty() && !report.ends_with('\n') {
            println!();
        }
    } else {
        println!("{} Server baseline checks passed.", output::success_marker());
    }
    Ok(())
}
