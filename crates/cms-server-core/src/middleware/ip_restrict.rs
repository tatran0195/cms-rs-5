//! IP CIDR restriction rules and validation.
//!
//! Evaluates client IP against optional CIDR allowlists and denylists.
//! DENY rules take absolute precedence over ALLOW rules.

use std::net::IpAddr;

/// Check whether an IP address matches a CIDR range or exact IP string.
///
/// Supports both IPv4 (`10.0.0.0/8`) and IPv6 (`::1/128`).
#[must_use]
pub fn ip_in_cidr(ip: IpAddr, cidr: &str) -> bool {
    let Some((network_str, prefix_str)) = cidr.split_once('/') else {
        return cidr.parse::<IpAddr>().is_ok_and(|a| a == ip);
    };

    let Ok(network) = network_str.parse::<IpAddr>() else {
        return false;
    };

    let Ok(prefix_len) = prefix_str.parse::<u32>() else {
        return false;
    };

    match (ip, network) {
        (IpAddr::V4(ip4), IpAddr::V4(net4)) => {
            if prefix_len > 32 {
                return false;
            }
            if prefix_len == 0 {
                return true;
            }
            let mask = u32::MAX << (32 - prefix_len);
            (u32::from(ip4) & mask) == (u32::from(net4) & mask)
        }
        (IpAddr::V6(ip6), IpAddr::V6(net6)) => {
            if prefix_len > 128 {
                return false;
            }
            if prefix_len == 0 {
                return true;
            }
            let mask = u128::MAX << (128 - prefix_len);
            (u128::from(ip6) & mask) == (u128::from(net6) & mask)
        }
        _ => false,
    }
}

/// Evaluates network access rules: DENY wins (blocked CIDR rejected immediately);
/// then the allowlist (an empty allowlist allows all non-blocked traffic).
#[must_use]
pub fn ip_allowed(ip: IpAddr, allowed: &[String], blocked: &[String]) -> bool {
    if blocked.iter().any(|cidr| ip_in_cidr(ip, cidr)) {
        return false;
    }
    allowed.is_empty() || allowed.iter().any(|cidr| ip_in_cidr(ip, cidr))
}
