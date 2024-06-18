use std::net::Ipv4Addr;

pub trait Address: Eq {}

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
}
