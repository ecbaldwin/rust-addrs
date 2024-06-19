use std::net::Ipv4Addr;

/// Defines minimum requirements of an ipv4 address for this crate
///
/// The purpose of this trait is not to replace nor even add to [`Ipv4Addr`]. It is well thought
/// out and (nearly) complete enough for my purposes[^1]. For this reason, this crate doesn't even
/// provide a type alias for it.
///
/// This trait is mostly for use within this crate to formalize its good properties on which this
/// crate may depend. We also provide a minimal set of integration tests which are meant as a
/// sanity check on the its behavior, not a comprehensive test suite.
///
/// This trait also servers to limit the touch points this crate has on that type.
///
/// [^1]: One minor exception is that I wish it implemented Into<[u8; 4]>. There is
/// [`Ipv4Addr::octets`] but I cannot make that a formal requirement of this trait.
pub trait Address:
    Eq
    + From<u32>
    + Into<u32>
    + From<[u8; 4]>
    + std::string::ToString
    + std::str::FromStr
    + Copy
    + Clone
    + Send
    + Sized
    + Sync
    + Unpin
{
    /// formalize that all v4 address are 32 bits
    const BITS: u8 = 32;
}

impl Address for Ipv4Addr {}
