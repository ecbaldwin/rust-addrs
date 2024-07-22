use addrs::errors::Result;
use addrs::ipv4::{self, Prefix, Set};

mod util;

#[test]
fn test_u32() {
    let prefix = util::p("10.224.24.1/24");
    assert_eq!(0x0ae01801u32, prefix.address().into());
    assert_eq!(0xffffff00u32, prefix.mask().into());
}

#[test]
fn octets() {
    let address = util::a("1.2.3.4");
    assert_eq!([1, 2, 3, 4], Into::<[u8; 4]>::into(address));
}

fn prefix_addr_len(prefix: util::Prefix, address: util::Address, mask: util::Address, len: u8) {
    assert_eq!(address, prefix.address());
    assert_eq!(mask, prefix.mask());
    assert_eq!(len, prefix.length());
}

runner::tests! { prefix_addr_len {
    basic(util::p("10.0.0.1/24"), util::a("10.0.0.1"), util::a("255.255.255.0"), 24);
} }

fn prefix_compare(a: &str, b: &str, eq: bool) {
    fn compare<A: ipv4::Prefix>(a: A, b: A, eq: bool) {
        assert_eq!(eq, a == b);
        assert_ne!(eq, a != b);
    }
    compare(util::p(a), util::p(b), eq)
}

runner::tests! { prefix_compare {
    equal("10.0.0.1/24", "10.0.0.1/24", true);
    length_not_equal("10.0.0.1/24", "10.0.0.1/25", false);
    host_bits_not_equal("10.0.0.1/24", "10.0.0.2/24", false);
    prefix_bits_not_equal("10.0.0.1/24", "11.0.0.1/24", false);
    extremes("0.0.0.0/0", "255.255.255.255/32", false);
} }

fn prefix_from_string(expected: std::result::Result<util::Prefix, ()>, cidr: &str) {
    assert_eq!(expected, cidr.parse().or(Err(())))
}

runner::tests! { prefix_from_string {
    success(Ok(util::p("10.224.24.1/27")), "10.224.24.1/27");
    ipv6(Err(()), "2001::1/64");
    bogus(Err(()), "bogus");
} }

fn prefix_to_string(expected: &str, prefix: util::Prefix) {
    assert_eq!(expected, prefix.to_string());
}

runner::tests! { prefix_to_string {
    success("10.224.24.1/27", util::p("10.224.24.1/27"));
    zero("0.0.0.0/0", util::p("0.0.0.0/0"));
    one_seventeen("10.224.24.117/25", util::p("10.224.24.117/25"));
    one_to_four("1.2.3.4/32", util::p("1.2.3.4/32"));
} }

fn prefix_net_host_broadcast(
    prefix: util::Prefix,
    network: util::Prefix,
    host: util::Prefix,
    broadcast: util::Prefix,
) {
    assert_eq!(network, Prefix::network(&prefix));
    assert_eq!(host, Prefix::host(&prefix));
    assert_eq!(broadcast, Prefix::broadcast(&prefix));
}

runner::tests! { prefix_net_host_broadcast {
    zero( util::p("10.224.24.1/0"), util::p("0.0.0.0/0"), util::p("10.224.24.1/0"), util::p("255.255.255.255/0"));
    eight( util::p("10.224.24.1/8"), util::p("10.0.0.0/8"), util::p("0.224.24.1/8"), util::p("10.255.255.255/8"));
    twentytwo( util::p("10.224.24.1/22"), util::p("10.224.24.0/22"), util::p("0.0.0.1/22"), util::p("10.224.27.255/22"));
    thirtytwo( util::p("10.224.24.1/32"), util::p("10.224.24.1/32"), util::p("0.0.0.0/32"), util::p("10.224.24.1/32"));
} }

fn num_addresses(expected: Result<u32>, prefix: util::Prefix) {
    util::assert_result(expected, prefix.num_addresses());
}

runner::tests! { num_addresses {
    all(Err(addrs::errors::Error::TooMany), util::p("0.0.0.0/0"));
    private(Ok(0x00100000), util::p("172.16.0.0/12"));
    host(Ok(1), util::p("172.16.244.117/32"));
} }

fn num_prefixes(expected: Result<u32>, prefix: util::Prefix, length: u8) {
    util::assert_result(expected, prefix.num_prefixes(length));
}

runner::tests! { num_prefixes {
    same_size(Ok(1), util::p("203.0.113.0/24"), 24);
    too_big(Ok(0), util::p("203.0.113.0/24"), 23);
    size_28(Ok(0x40), util::p("203.0.113.0/24"), 30);
    size_26(Ok(4), util::p("203.0.113.0/24"), 26);
    bad_length(Err(addrs::errors::Error::InvalidLength), util::p("0.0.0.0/0"), 33);
    too_many(Err(addrs::errors::Error::TooMany), util::p("0.0.0.0/0"), 32);
} }

fn from_address_length(expected: Result<util::Prefix>, address: util::Address, length: u8) {
    util::assert_result(expected, util::Prefix::from_address_length(address, length));
}

runner::tests! { from_address_length {
    basic(Ok(util::p("192.168.1.1/24")), util::a("192.168.1.1"), 24);
    invalid_length(Err(addrs::errors::Error::InvalidLength), util::a("192.168.1.1"), 33);
} }

fn from_address_mask(expected: Result<util::Prefix>, address: util::Address, mask: util::Address) {
    util::assert_result(expected, util::Prefix::from_address_mask(address, mask));
}

runner::tests! { from_address_mask {
    basic(Ok(util::p("192.168.1.1/24")), util::a("192.168.1.1"), util::a("255.255.255.0"));
    invalid_length(Err(addrs::errors::Error::InvalidMask), util::a("192.168.1.1"), util::a("192.168.1.0"));
} }

fn halves(expected: Option<(util::Prefix, util::Prefix)>, prefix: util::Prefix) {
    assert_eq!(expected, prefix.halves())
}

runner::tests! { halves {
    all_ipv4(Some((util::p("0.0.0.0/1"), util::p("128.0.0.0/1"))), util::p("0.0.0.0/0"));
    size_16(Some((util::p("10.224.0.0/17"), util::p("10.224.128.0/17"))), util::p("10.224.0.0/16"));
    size_24(Some((util::p("10.224.24.0/25"), util::p("10.224.24.128/25"))), util::p("10.224.24.1/24"));
    size_31(Some((util::p("10.224.24.116/32"), util::p("10.224.24.117/32"))), util::p("10.224.24.117/31"));
    size_32(None, util::p("10.224.24.117/32"));
} }

fn contains<A: addrs::ipv4::Prefix, B: addrs::ipv4::Prefix<Address = A::Address>>(
    container: A,
    containee: B,
) {
    assert!(container.contains(&containee));
    assert_eq!(
        // I cannot use == between different concrete types of Prefix
        container.length() == containee.length(),
        containee.contains(&container)
    );
}

runner::tests! { contains {
    all(util::p("0.0.0.0/0"), util::p("1.2.3.4/32"));
    same_host(util::p("1.2.3.4/24"), util::p("1.2.3.4/24"));
    same_host_route(util::p("1.2.3.4/32"), util::p("1.2.3.4/32"));
    same_prefix(util::p("192.168.20.0/24"), util::p("192.168.20.0/24"));
    contained_smaller(util::p("192.168.0.0/16"), util::p("192.168.20.0/24"));
    ignore_host_part(util::p("1.2.3.4/24"), util::p("1.2.3.5/32"));

    all_32(util::p("0.0.0.0/0"), util::a("1.2.3.4"));
    same_host_route_32(util::a("1.2.3.4"), util::a("1.2.3.4"));
    ignore_host_part_32(util::p("1.2.3.4/24"), util::a("1.2.3.5"));
} }
