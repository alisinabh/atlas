use atlas_rs::network_utils::SpecialIPCheck;
use std::net::IpAddr;

fn is_special(ip: &str) -> bool {
    ip.parse::<IpAddr>().unwrap().is_special_ip()
}

#[test]
fn test_special_ipv4() {
    for ip in [
        "0.0.0.0",
        "0.1.2.3",
        "10.1.2.3",
        "100.64.0.1",
        "100.127.255.255",
        "127.0.0.1",
        "169.254.10.10",
        "172.16.0.1",
        "192.168.0.1",
        "192.0.2.1",
        "198.51.100.1",
        "203.0.113.1",
        "224.0.0.1",
        "239.255.255.255",
        "240.0.0.1",
        "255.255.255.255",
    ] {
        assert!(is_special(ip), "{ip} should be special");
    }
}

#[test]
fn test_public_ipv4() {
    for ip in [
        "1.1.1.1",
        "4.2.2.4",
        "8.8.8.8",
        "100.63.255.255",
        "100.128.0.1",
        "172.32.0.1",
        "214.78.120.1",
        "223.255.255.255",
    ] {
        assert!(!is_special(ip), "{ip} should not be special");
    }
}

#[test]
fn test_special_ipv6() {
    for ip in [
        "::",
        "::1",
        "fe80::1",
        "fc00::1",
        "fd12:3456::1",
        "ff02::1",
        "2001:db8::1",
        "::ffff:10.0.0.1",
        "::ffff:127.0.0.1",
        "::ffff:192.168.1.1",
    ] {
        assert!(is_special(ip), "{ip} should be special");
    }
}

#[test]
fn test_public_ipv6() {
    for ip in [
        "2001:218::1",
        "2606:4700:4700::1111",
        "2a00:1450::1",
        "::ffff:1.1.1.1",
    ] {
        assert!(!is_special(ip), "{ip} should not be special");
    }
}
