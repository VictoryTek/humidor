//! SSRF guard for server-side fetches of user-supplied URLs.
//!
//! A URL is allowed only if it is plain http(s) on port 80/443 without embedded credentials and
//! *every* address its host resolves to is a public unicast address. The validated addresses are
//! returned so the caller can pin the connection to them (defeating DNS rebinding).

use reqwest::Url;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

const DNS_TIMEOUT: Duration = Duration::from_secs(5);

fn is_public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_multicast()
        || a == 0
        || (a == 100 && (b & 0xc0) == 64) // 100.64.0.0/10 CGNAT
        || (a == 192 && b == 0 && c == 0) // 192.0.0.0/24 IETF protocol assignments
        || (a == 198 && (b & 0xfe) == 18) // 198.18.0.0/15 benchmarking
        || a >= 240) // 240.0.0.0/4 reserved
}

fn is_public_v6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    // Only global unicast (2000::/3). This excludes ::, ::1, fc00::/7, fe80::/10, ff00::/8 and
    // 64:ff9b::/96; also drop documentation, 6to4 and Teredo ranges, which embed IPv4 addresses.
    (s[0] & 0xe000) == 0x2000
        && !(s[0] == 0x2001 && (s[1] == 0x0db8 || s[1] == 0))
        && s[0] != 0x2002
}

pub fn is_public_ip(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(v4) => is_public_v4(v4),
        IpAddr::V6(v6) => is_public_v6(v6),
    }
}

/// Validate `url` and return the socket addresses it may be fetched from.
pub async fn check_url(url: &Url) -> Result<Vec<SocketAddr>, String> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err("Only http and https URLs are allowed".into());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("URLs with embedded credentials are not allowed".into());
    }
    let port = url.port_or_known_default().ok_or("Missing port")?;
    if port != 80 && port != 443 {
        return Err("Only ports 80 and 443 are allowed".into());
    }
    let host = url.host_str().ok_or("Missing host")?;

    let addrs: Vec<SocketAddr> = match host.trim_matches(['[', ']']).parse::<IpAddr>() {
        Ok(ip) => vec![SocketAddr::new(ip, port)],
        Err(_) => tokio::time::timeout(DNS_TIMEOUT, tokio::net::lookup_host((host, port)))
            .await
            .map_err(|_| "DNS lookup timed out".to_string())?
            .map_err(|e| format!("DNS lookup failed: {e}"))?
            .collect(),
    };

    if addrs.is_empty() || addrs.iter().any(|a| !is_public_ip(a.ip())) {
        return Err("URL does not resolve to a public address".into());
    }
    Ok(addrs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn private_and_special_addresses_are_not_public() {
        for s in [
            "127.0.0.1",
            "0.0.0.0",
            "10.1.2.3",
            "172.16.0.1",
            "172.31.255.255",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "192.0.0.1",
            "198.18.0.1",
            "224.0.0.1",
            "240.0.0.1",
            "255.255.255.255",
            "::",
            "::1",
            "fe80::1",
            "fc00::1",
            "fd12:3456::1",
            "ff02::1",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.1",
            "::ffff:169.254.169.254",
            "64:ff9b::7f00:1",
            "2001:db8::1",
            "2002:7f00:1::1",
            "2001::1",
        ] {
            assert!(!is_public_ip(ip(s)), "{s} must be rejected");
        }
    }

    #[test]
    fn public_addresses_are_allowed() {
        for s in [
            "93.184.216.34",
            "8.8.8.8",
            "1.1.1.1",
            "2606:4700:4700::1111",
            "::ffff:8.8.8.8",
        ] {
            assert!(is_public_ip(ip(s)), "{s} must be allowed");
        }
    }

    #[tokio::test]
    async fn check_url_rejects_dangerous_urls() {
        for s in [
            "file:///etc/passwd",
            "ftp://example.com/",
            "gopher://example.com/",
            "http://127.0.0.1/",
            "http://[::1]/",
            "http://169.254.169.254/latest/meta-data/",
            "http://10.0.0.5/",
            "http://2130706433/", // decimal form of 127.0.0.1
            "http://0x7f.1/",     // hex/short form of 127.0.0.1
            "http://[::ffff:127.0.0.1]/",
            "http://localhost/",
            "http://user:pw@93.184.216.34/",
            "http://93.184.216.34:8080/",
            "https://93.184.216.34:22/",
        ] {
            let url = Url::parse(s).unwrap();
            assert!(check_url(&url).await.is_err(), "{s} must be rejected");
        }
    }

    #[tokio::test]
    async fn check_url_accepts_public_ip_literals() {
        for s in [
            "http://93.184.216.34/",
            "https://93.184.216.34/x?y=1",
            "https://[2606:4700:4700::1111]/",
        ] {
            let url = Url::parse(s).unwrap();
            assert!(check_url(&url).await.is_ok(), "{s} must be accepted");
        }
    }
}
