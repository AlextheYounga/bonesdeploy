use bonesremote::commands::service::services_for_release;

#[test]
fn release_restart_excludes_only_the_project_cloudflared_unit() {
    let services = vec![
        String::from("atlas-nginx.service"),
        String::from("atlas-cloudflared.service"),
        String::from("atlas-php.service"),
    ];

    assert_eq!(
        services_for_release("atlas.target", &services),
        vec![String::from("atlas-nginx.service"), String::from("atlas-php.service")]
    );
}

#[test]
fn release_restart_preserves_cloudflared_units_for_other_projects() {
    let services = vec![String::from("atlas-cloudflared.service"), String::from("other-cloudflared.service")];

    assert_eq!(services_for_release("atlas.target", &services), vec![String::from("other-cloudflared.service")]);
}
