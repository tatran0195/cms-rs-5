//! SSRF protection helper for webhook integrations.

use std::net::IpAddr;

/// Checks if an IP address is private, loopback, or restricted.
#[must_use]
pub fn is_private_or_restricted_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_loopback()
                || ipv4.is_private()
                || ipv4.is_link_local()
                || ipv4.is_broadcast()
                || ipv4.is_documentation()
                || ipv4.is_multicast()
                || ipv4.octets()[0] == 0 // 0.0.0.0/8 (Current network)
                || (ipv4.octets()[0] == 100 && (ipv4.octets()[1] & 0xc0) == 64) // CGNAT 100.64.0.0/10
                || (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 0) // 192.0.0.0/24 (IETF Protocol)
                || (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 2) // 192.0.2.0/24 (TEST-NET-1)
                || (ipv4.octets()[0] == 198 && (ipv4.octets()[1] == 18 || ipv4.octets()[1] == 19)) // 198.18.0.0/15 (Benchmarking)
                || (ipv4.octets()[0] == 198 && ipv4.octets()[1] == 51 && ipv4.octets()[2] == 100) // 198.51.100.0/24 (TEST-NET-2)
                || (ipv4.octets()[0] == 203 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 113) // 203.0.113.0/24 (TEST-NET-3)
                || ipv4.octets()[0] >= 240 // 240.0.0.0/4 (Reserved / Future use)
        }
        IpAddr::V6(ipv6) => {
            ipv6.is_loopback()
                || ipv6.is_multicast()
                || ipv6.is_unspecified()
                || ((ipv6.segments()[0] & 0xfe00) == 0xfc00) // Unique local (fc00::/7)
                || ((ipv6.segments()[0] & 0xffc0) == 0xfe80) // Link-local (fe80::/10)
                || (ipv6.segments()[0] == 0x2001 && ipv6.segments()[1] == 0xdb8) // 2001:db8::/32 (Documentation)
                || (ipv6.segments()[0] == 0x2001 && ipv6.segments()[1] == 0x2 && ipv6.segments()[2] == 0) // 2001:2::/48 (Benchmarking)
                || (ipv6.segments()[0] == 0x0100) // 100::/64 (Discard prefix)
                || ipv6.to_ipv4_mapped().map(|v4| is_private_or_restricted_ip(IpAddr::V4(v4))).unwrap_or(false)
                || ipv6.to_ipv4().map(|v4| is_private_or_restricted_ip(IpAddr::V4(v4))).unwrap_or(false)
        }
    }
}
