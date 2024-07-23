impl crate::ipv4::Address for std::net::Ipv4Addr {
    type UI = u32;

    fn octets(&self) -> [u8; 4] {
        self.octets()
    }
}

impl crate::ipv4::Prefix for ipnet::Ipv4Net {
    type Address = std::net::Ipv4Addr;

    fn address(&self) -> Self::Address {
        self.addr()
    }

    fn length(&self) -> u8 {
        self.prefix_len()
    }

    unsafe fn unsafe_new(ip: Self::Address, length: u8) -> Self {
        Self::new(ip, length).unwrap_unchecked()
    }
}
