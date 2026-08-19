use anyhow::Result;

use bonesremote::release::lifecycle::build::ownership::parse_user_uid;

#[test]
fn parse_user_uid_reads_uid_field() -> Result<()> {
    let passwd = "root:x:0:0:root:/root:/bin/bash\ndemo-build:x:1234:1234::/nonexistent:/usr/sbin/nologin\n";
    assert_eq!(parse_user_uid(passwd, "demo-build")?, 1234);
    Ok(())
}
