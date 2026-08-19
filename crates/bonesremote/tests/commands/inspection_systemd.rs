use bonesremote::inspection::systemd::parse_required_services;

#[test]
fn parses_unique_service_dependencies_only() {
    assert_eq!(
        parse_required_services("nexttest-nginx.service nexttest-next.service nexttest.target nexttest-next.service"),
        ["nexttest-next.service", "nexttest-nginx.service"]
    );
}
