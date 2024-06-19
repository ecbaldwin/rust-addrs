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

pub fn a(s: &str) -> Address {
    s.parse().expect("bad ip")
}
