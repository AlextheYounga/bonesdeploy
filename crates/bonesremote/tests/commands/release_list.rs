use bonesremote::commands::release::list::process_start_ticks;

#[test]
fn parses_process_start_ticks_after_parenthesized_name() {
    let stat = "123 (bonesremote) R 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 99 20";
    assert_eq!(process_start_ticks(stat), Some(99));
}
