use std::collections::BTreeSet;

use anyhow::{Result, anyhow};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "./assets/skill/"]
struct SkillAssets;

/// Skill doc names, sorted, excluding the orientation doc `SKILL`.
pub fn doc_names() -> Vec<String> {
    SkillAssets::iter()
        .map(|p| p.to_string())
        .filter(|p| !p.starts_with("SKILL"))
        .map(|p| p.trim_end_matches(".md").to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Orientation doc printed by `bonesdeploy skill` (no subcommand).
pub fn orientation() -> Result<String> {
    let asset =
        SkillAssets::get("SKILL.md").ok_or_else(|| anyhow!("embedded skill orientation SKILL.md is missing"))?;
    Ok(String::from_utf8(asset.data.to_vec())?)
}

/// Named skill doc printed by `bonesdeploy skill doc <name>`.
pub fn doc(name: &str) -> Result<String> {
    let path = format!("{name}.md");
    let asset =
        SkillAssets::get(&path).ok_or_else(|| anyhow!("no skill doc named {name}. Run `bonesdeploy skill list`."))?;
    Ok(String::from_utf8(asset.data.to_vec())?)
}
