use addrs::ipv4::{Prefix, Set};

mod util;

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

runner::tests! { range_contains {
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

runner::tests! { range_empty {
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
    assert_eq!(7, range.num_addresses().unwrap());
}

#[test]
fn prefix_as_range() {
    let prefix = util::p("10.224.0.0/24");
    let range = prefix.as_range_i();
    assert_eq!(&prefix.network().address(), range.start());
    assert_eq!(&prefix.broadcast().address(), range.end());
}

#[test]
fn address_as_range() {
    let address = util::a("10.224.0.0");
    let range = address.as_range_i();
    assert_eq!(&address, range.start());
    assert_eq!(&address, range.end());
}

fn num_prefixes(expected: u32, from: &str, to: &str, length: u8) {
    let from = util::a(from);
    let to = util::a(to);
    let range = from..=to;
    assert_eq!(expected, range.num_prefixes(length).unwrap());
}

runner::tests! { num_prefixes {
    empty(0, "10.224.24.1", "10.224.24.0", 32);
    single(1, "10.224.24.1", "10.224.24.1", 32);
    lowest(1, "0.0.0.0", "0.0.0.0", 32);
    highest(1, "255.255.255.255", "255.255.255.255", 32);
    class_c(256, "10.224.24.0", "10.224.24.255", 32);

    class_b_to_c(256, "10.224.0.0", "10.224.255.255", 24);
    class_b_to_c_extra(256, "10.223.255.1", "10.225.0.0", 24);

    class_c_not_aligned(0, "10.223.255.1", "10.224.0.254", 24);

    just_two(2, "127.255.255.255", "128.0.0.0", 32);
} }

#[test]
fn num_prefixes_err() {
    let from = util::a("10.224.24.1");
    let to = util::a("10.224.24.0");
    let range = from..=to;
    assert!(range.num_prefixes(33).is_err());
}
