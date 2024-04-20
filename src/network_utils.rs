use std::net::IpAddr;

pub trait SpecialIPCheck {
    fn is_special_ip(&self) -> bool;
}

impl SpecialIPCheck for IpAddr {
    fn is_special_ip(&self) -> bool {
        match self {
            IpAddr::V4(ip4) => {
                ip4.is_private()
                    || ip4.is_loopback()
                    || ip4.is_multicast()
                    || ip4.is_broadcast()
                    || ip4.is_unspecified()
            }
            IpAddr::V6(ip6) => ip6.is_unspecified() || ip6.is_multicast() || ip6.is_loopback(),
        }
    }
}
