use addrs::ipv4::Address;

mod util;
type Ipv4Addr = util::Address;

#[test]
fn to_array_octets() {
    let octets = Ipv4Addr::LOCALHOST.octets();
    assert_eq!(octets, [127, 0, 0, 1]);
}

#[test]
fn to_array_into() {
    // NOTE More verbose than the octets counterpart
    let octets: [u8; 4] = Ipv4Addr::LOCALHOST.into();
    assert_eq!(octets, [127, 0, 0, 1]);
}

#[test]
fn as_ref_octets() {
    let octets: &[u8] = &Ipv4Addr::LOCALHOST.octets();
    assert_eq!(octets, [127, 0, 0, 1]);
}

#[test]
fn as_ref_into() {
    // NOTE More verbose than the octets counterpart
    let octets: &[u8] = &Into::<[u8; 4]>::into(Ipv4Addr::LOCALHOST);
    assert_eq!(octets, [127, 0, 0, 1]);
}

#[test]
fn const_octets() {
    const IP_ADDRESS: Ipv4Addr = Ipv4Addr::LOCALHOST;

    assert_eq!(IP_ADDRESS.octets(), [127, 0, 0, 1]);
}

#[test]
fn const_into() {
    const IP_ADDRESS: Ipv4Addr = Ipv4Addr::LOCALHOST;

    let octets: [u8; 4] = IP_ADDRESS.into();
    assert_eq!(octets, [127, 0, 0, 1]);
}

const ARRAY_SIZE: usize = 4;

fn needs_into<A: Into<[u8; ARRAY_SIZE]>>(address: A) {
    let octets: [u8; ARRAY_SIZE] = address.into();
    assert_eq!(octets, [127, 0, 0, 1]);
}

#[test]
fn generic_into() {
    needs_into(Ipv4Addr::LOCALHOST);
}

fn needs_into_ref<A: Into<[u8; ARRAY_SIZE]>>(address: A) {
    let octets: &[u8; ARRAY_SIZE] = &address.into();
    assert_eq!(*octets, [127, 0, 0, 1]);
}

#[test]
fn generic_into_ref() {
    needs_into_ref(Ipv4Addr::LOCALHOST);
}
