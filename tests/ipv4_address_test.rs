use addrs::ipv4;

type AddressImpl = std::net::Ipv4Addr;

fn _ipv4_addr(s: &str) -> AddressImpl {
    s.parse().expect("bad ip")
}

fn address_compare(a: &str, b: &str, eq: bool) {
    fn compare<A: ipv4::Address>(a: A, b: A, eq: bool) {
        assert_eq!(eq, a == b);
        assert_ne!(eq, a != b);
    }
    compare(_ipv4_addr(a), _ipv4_addr(b), eq)
}

tests! { address_compare {
    equal("10.0.0.1", "10.0.0.1", true);
    not_equal("10.0.0.1", "10.0.0.2", false);
    zero_eq("0.0.0.0", "0.0.0.0", true);
    all_ones_eq("255.255.255.255", "255.255.255.255", true);
    extrames("0.0.0.0", "255.255.255.255", false);
} }

#[test]
fn address_size() {
    assert_eq!(32u8, <AddressImpl as ipv4::Address>::BITS);
}

#[test]
fn address_from_string() {
    let ip: AddressImpl = "10.224.24.1".parse().expect("bad ip");
    assert_eq!(AddressImpl::from(0x0ae01801), ip);
}

#[test]
fn address_from_bytes() {
    let ip: AddressImpl = [10, 224, 24, 1].into();
    assert_eq!(AddressImpl::from(0x0ae01801u32), ip);
}

#[test]
fn address_to_u32() {
    let ip: AddressImpl = [10, 224, 24, 1].into();
    assert_eq!(0x0ae01801u32, ip.into());
}

#[test]
fn address_to_string() {
    let ip: AddressImpl = [10, 224, 24, 1].into();
    assert_eq!("10.224.24.1", ip.to_string());
}

#[test]
fn address_to_octets() {
    let ip: AddressImpl = [10, 224, 24, 1].into();
    assert_eq!([10, 224, 24, 1], ip.octets());
}

#[macro_export]
macro_rules! tests {
    ($property_test_func:ident {
        $( $(#[$attr:meta])* $test_name:ident( $( $param:expr ),* ); )+
    }) => {
        $(
            paste::paste! {
                $(#[$attr])*
                #[test]
                fn [<$property_test_func ˬˬ $test_name>]() {
                    $property_test_func($( $param ),* )
                }
            }
        )+
    }
}
