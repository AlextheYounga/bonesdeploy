use std::{env, fs, os::unix::fs::PermissionsExt, process};

use anyhow::Result;

use bonesremote::release::lifecycle::build::build_user::BuildScriptEnv;
use bonesremote::release::lifecycle::build::container::*;

#[test]
fn build_env_values_use_a_private_env_file_instead_of_command_arguments() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-build-env-file-{}", process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;
    let variables = vec![("PUBLIC_API_URL".to_string(), "https://example.test/private-value".to_string())];
    let env = BuildScriptEnv {
        project_name: "demo",
        build_user: "demo-build",
        build_group: "demo-build",
        web_root: ".output/public",
        deployment_dir: &root,
        build_cache_dir: &root,
        build_env_vars: &variables,
        script_timeout_seconds: None,
    };

    let env_file = write_build_env_file(&root, &env)?;
    let command = build_container_command(&root, &env, "bonesdeploy-build-demo", &env_file);
    let arguments: Vec<_> = command.get_args().map(|argument| argument.to_string_lossy().into_owned()).collect();

    let env_file_argument = env_file.to_string_lossy();
    assert!(arguments.windows(2).any(|pair| pair[0] == "--env-file" && pair[1] == env_file_argument.as_ref()));
    assert!(!arguments.iter().any(|argument| argument.contains("private-value")));
    assert_eq!(fs::metadata(&env_file)?.permissions().mode() & 0o777, 0o600);
    assert_eq!(fs::read_to_string(&env_file)?, "PUBLIC_API_URL=https://example.test/private-value\n");

    fs::remove_file(env_file).ok();
    fs::remove_dir_all(root).ok();
    Ok(())
}
