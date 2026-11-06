use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub trait SpecialIPCheck {
    fn is_special_ip(&self) -> bool;
}

impl SpecialIPCheck for IpAddr {
    fn is_special_ip(&self) -> bool {
        match self {
            IpAddr::V4(ip4) => ip4.is_special_ip(),
            IpAddr::V6(ip6) => ip6.is_special_ip(),
        }
    }
}

impl SpecialIPCheck for Ipv4Addr {
    fn is_special_ip(&self) -> bool {
        let [first, second, ..] = self.octets();

        self.is_private()
            || self.is_loopback()
            || self.is_link_local()
            || self.is_multicast()
            || self.is_broadcast()
            || self.is_unspecified()
            || self.is_documentation()
            // "This network" 0.0.0.0/8
            || first == 0
            // Shared address space 100.64.0.0/10
            || (first == 100 && (second & 0b1100_0000) == 0b0100_0000)
            // Reserved 240.0.0.0/4
            || first >= 240
    }
}

impl SpecialIPCheck for Ipv6Addr {
    fn is_special_ip(&self) -> bool {
        if let Some(ip4) = self.to_ipv4_mapped() {
            return ip4.is_special_ip();
        }

        let [first, second, ..] = self.segments();

        self.is_unspecified()
            || self.is_loopback()
            || self.is_multicast()
            || self.is_unique_local()
            || self.is_unicast_link_local()
            // Documentation 2001:db8::/32
            || (first == 0x2001 && second == 0x0db8)
    }
}
