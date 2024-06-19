pub trait Address:
    Eq
    + From<u32>
    + Into<u32>
    + From<[u8; 4]>
    // I'm not sure how to handle this. Ipv4 has `octets` which I use implicitly
    // + Into<[u8; 4]>
    + std::string::ToString
    + std::str::FromStr
    + Copy + Clone + Send + Sized + Sync + Unpin
{
    const BITS: u8 = 32;
}

impl Address for std::net::Ipv4Addr {}
