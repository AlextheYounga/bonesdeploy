//! `.env.build` parsing and loading for the `bonesdeploy-core` library.

use std::fs;

use anyhow::Result;
use bonesdeploy_core::config::build_env;
use tempfile::TempDir;

#[test]
fn parses_unquoted_values() -> Result<()> {
    let map = build_env::parse("KEY=hello\nOTHER=world")?;
    assert_eq!(map.get("KEY").map(String::as_str), Some("hello"));
    assert_eq!(map.get("OTHER").map(String::as_str), Some("world"));
    Ok(())
}

#[test]
fn parses_quoted_values() -> Result<()> {
    let map = build_env::parse("KEY=\"hello world\"\nOTHER='single quotes'")?;
    assert_eq!(map.get("KEY").map(String::as_str), Some("hello world"));
    assert_eq!(map.get("OTHER").map(String::as_str), Some("single quotes"));
    Ok(())
}

#[test]
fn skips_comments_and_blank_lines() -> Result<()> {
    let map = build_env::parse("# comment\n\nKEY=val\n  \n# another\nOTHER=other")?;
    assert_eq!(map.len(), 2);
    assert_eq!(map.get("KEY").map(String::as_str), Some("val"));
    Ok(())
}

#[test]
fn default_content_declares_node_version() {
    assert!(build_env::default_content().contains("# BonesDeploy Infra\nNODE_VERSION=\n"));
}

#[test]
fn rejects_invalid_keys() {
    assert!(build_env::parse("1BAD=value").is_err());
    assert!(build_env::parse("BAD-KEY=value").is_err());
    assert!(build_env::parse("=value").is_err());
}

#[test]
fn rejects_bones_reserved_prefix() {
    assert!(build_env::parse("BONES_SOMETHING=value").is_err());
    assert!(build_env::parse("BONES_X=1").is_err());
}

#[test]
fn rejects_duplicate_keys() {
    assert!(build_env::parse("KEY=one\nKEY=two").is_err());
}

#[test]
fn allows_valid_underscore_names() -> Result<()> {
    let map = build_env::parse("_UNDERSCORE=1\nNEXT_PUBLIC_API_URL=https://api.example.com")?;
    assert_eq!(map.len(), 2);
    Ok(())
}

#[test]
fn load_returns_empty_map_when_file_missing() -> Result<()> {
    let dir = TempDir::new()?;
    let map = build_env::load(dir.path())?;
    assert!(map.is_empty());
    Ok(())
}

#[test]
fn load_reads_env_build_from_directory() -> Result<()> {
    let dir = TempDir::new()?;
    fs::write(dir.path().join(".env.build"), "API_URL=https://api.example.com\nSITE_NAME=Test\n")?;
    let map = build_env::load(dir.path())?;
    assert_eq!(map.get("API_URL").map(String::as_str), Some("https://api.example.com"));
    assert_eq!(map.get("SITE_NAME").map(String::as_str), Some("Test"));
    Ok(())
}

#[test]
fn derived_bones_values_cannot_be_overridden() {
    let result = build_env::parse("BONES_RUNTIME_TEMPLATE=evil");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("reserved"), "error should mention reserved: {err}");
}
