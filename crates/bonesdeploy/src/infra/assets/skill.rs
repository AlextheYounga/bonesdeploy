use std::collections::BTreeSet;

use anyhow::{Result, anyhow};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "./skill/"]
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

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::{doc, doc_names, orientation};

    #[test]
    fn skill_orientation_loads_and_names_the_five_moves() -> Result<()> {
        let doc = orientation()?;
        assert!(doc.contains("# BonesDeploy: the skill"));
        assert!(doc.contains("The five moves"), "orientation doc lost its five-moves anchor");
        assert!(doc.contains("bonesdeploy skill next"), "orientation must point agents at `skill next`");
        Ok(())
    }

    #[test]
    fn skill_doc_names_cover_the_expected_topics() {
        let names = doc_names();
        assert!(names.contains(&"commands".to_string()), "missing `commands` skill doc");
        assert!(names.contains(&"workflows".to_string()), "missing `workflows` skill doc");
        assert!(names.contains(&"methodology".to_string()), "missing `methodology` skill doc");
        assert!(!names.contains(&"SKILL".to_string()), "SKILL.md must be excluded from `skill list`");
    }

    #[test]
    fn skill_doc_lookup_round_trips_for_known_names() -> Result<()> {
        let commands = doc("commands")?;
        assert!(commands.contains("# BonesDeploy commands"));
        Ok(())
    }
}
