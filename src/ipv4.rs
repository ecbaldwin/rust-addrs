use std::net::Ipv4Addr;

pub trait Address: Eq + std::str::FromStr {
    const BITS: u8 = 32;
}

impl Address for Ipv4Addr {}

#[cfg(test)]
mod tests {
    use super::*;

    fn _ipv4_addr(s: &str) -> Ipv4Addr {
        s.parse().expect("bad ip")
    }

    fn address_compare(a: &str, b: &str, eq: bool) {
        fn compare<A: Address>(a: A, b: A, eq: bool) {
            assert_eq!(eq, a == b);
            assert_ne!(eq, a != b);
        }
        compare(_ipv4_addr(a), _ipv4_addr(b), eq)
    }

    crate::tests! { address_compare {
        equal("10.0.0.1", "10.0.0.1", true);
        not_equal("10.0.0.1", "10.0.0.2", false);
        zero_eq("0.0.0.0", "0.0.0.0", true);
        all_ones_eq("255.255.255.255", "255.255.255.255", true);
        extrames("0.0.0.0", "255.255.255.255", false);
    } }

    #[test]
    fn address_size() {
        assert_eq!(32u8, <Ipv4Addr as Address>::BITS);
    }

    #[test]
    fn address_from_string() {
        let ip: Ipv4Addr = "10.224.24.1".parse().expect("bad ip");
        assert_eq!(Ipv4Addr::from(0x0ae01801), ip);
    }
}
