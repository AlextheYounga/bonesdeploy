use anyhow::Result;
use bonesdeploy::commands::site::strip_ansi;
use bonesdeploy::ui::manifest::render_report;
use bonesinfra::manifest::ManifestReport;

#[test]
fn manifest_report_renders_a_status_only_path_tree_and_managed_services() -> Result<()> {
    let report: ManifestReport = serde_json::from_str(
        r#"{
            "strategy":{"framework":"none","mode":"custom","backend":"native","ssl":false},
            "entries":[
                {"name":"current","owner":"deploy","path":"/srv/sites/example/current","kind":"link","state":"missing"},
                {"name":"site","owner":"setup","path":"/srv/sites/example","kind":"directory","state":"present"},
                {"name":"nginx","owner":"runtime","path":"/etc/nginx/example.conf","kind":"file","state":"wrong-kind"}
            ],
            "managed_services":[{"name":"app","owner":"runtime","unit":"example.service","running":true,"enabled":true}],
            "compose":null
        }"#,
    )?;
    let output = strip_ansi(&render_report(report)?);
    assert!(output.contains("├── etc/\n│   └── nginx/\n│       └── ✗ example.conf"), "{output}");
    assert!(output.contains("└── srv/\n    └── sites/\n        └── ✓ example/\n            └── ! current"), "{output}");
    assert!(output.contains("Managed services\n✓ example.service"), "{output}");
    assert!(!output.contains("directory"));
    assert!(!output.contains("setup"));
    Ok(())
}
