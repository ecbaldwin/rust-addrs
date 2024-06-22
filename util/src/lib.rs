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

pub type Address = std::net::Ipv4Addr;
pub type Prefix = ipnet::Ipv4Net;

pub fn a(s: &str) -> Address {
    s.parse().expect("bad ip")
}

pub fn p(s: &str) -> Prefix {
    s.parse().expect("bad prefix")
}

pub fn assert_result<T: Eq + std::fmt::Debug, E: ToString>(
    expected: Result<T, E>,
    actual: Result<T, E>,
) {
    match expected {
        Ok(expected) => {
            assert!(actual
                .inspect(|actual| {
                    assert_eq!(expected, *actual);
                })
                .is_ok());
        }
        Err(expected) => {
            assert!(actual
                .inspect_err(|actual| {
                    assert_eq!(expected.to_string(), actual.to_string());
                })
                .is_err());
        }
    }
}
