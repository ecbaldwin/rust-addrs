use addrs::ipv4::Prefix;

fn range_contains(from: &str, to: &str, contained: &str, not_contained: Vec<&str>) {
    let from = util::a(from);
    let to = util::a(to);
    let address = util::a(contained);
    let range = from..=to;
    assert!(range.contains(&address));
    for a in not_contained {
        let a = util::a(a);
        assert!(!range.contains(&a));
    }
}

util::tests! { range_contains {
    all_ipv4("0.0.0.0", "255.255.255.255", "1.2.3.4", vec![]);
    ten("10.0.0.0", "10.255.255.255", "10.64.128.196", vec!["9.255.255.255", "11.0.0.0"]);
    single("10.0.0.0", "10.0.0.0", "10.0.0.0", vec!["0.0.0.0", "9.255.255.255", "11.0.0.0", "255.255.255.255"]);
} }

fn range_empty(from: &str, empty_to: &str, not_empty_to: &str) {
    let from = util::a(from);
    let empty_to = util::a(empty_to);
    assert!((from..=empty_to).is_empty());

    let not_empty_to = util::a(not_empty_to);
    assert!(!(from..=not_empty_to).is_empty());
}

util::tests! { range_empty {
    extremes("255.255.255.255", "0.0.0.0", "255.255.255.255");
    one_off("10.0.0.1", "10.0.0.0", "10.0.0.1");
} }

#[test]
fn debug() {
    let range = util::a("192.168.0.1")..=util::a("192.168.0.7");
    let range_str = format!("{range:?}");
    assert_eq!("192.168.0.1..=192.168.0.7", range_str);
}

#[test]
fn iterator() {
    let range = util::a("192.168.0.1")..=util::a("192.168.0.7");
    assert_eq!(7, range.count());
}

#[test]
fn prefix_as_range() {
    let prefix = util::p("10.224.0.0/24");
    let range = prefix.as_range_i();
    assert_eq!(&prefix.network(), range.start());
    assert_eq!(&prefix.broadcast(), range.end());
}

#[test]
fn address_as_range() {
    let address = util::a("10.224.0.0");
    let range = address.as_range_i();
    assert_eq!(&address, range.start());
    assert_eq!(&address, range.end());
}
