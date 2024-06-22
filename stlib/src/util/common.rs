use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use sha1::{Digest, Sha1};
use crate::model::Sha1Hash;

pub fn sha1_hash(bytes: impl AsRef<[u8]>) -> Sha1Hash {
    let mut hash = [0u8; 20];
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    let hasher_output = hasher.finalize();
    let result: &[u8] = hasher_output.as_slice();
    hash.copy_from_slice(result);
    hash
}

pub fn resolve_ipv4_addr(addr: &str) -> Result<SocketAddr, io::Error> {
    if ! addr.starts_with("udp:") && ! addr.starts_with("http:") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Address should start with 'udp:' or 'http:'"));
    }
    let addr = &addr[addr.find(':').unwrap() + 1..];
    match addr.to_socket_addrs()?
        .filter(|addr| addr.is_ipv4())
        .next() {
        Some(socket_addr) => Ok(socket_addr),
        None => Err(io::Error::new(io::ErrorKind::AddrNotAvailable, "No ipv4 socket address found"))
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use super::*;

    #[test]
    fn test_hashing() {
        let bytes = b"hello world";
        let correct_hash = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";
        assert_eq!(hex::encode(sha1_hash(bytes)), correct_hash);
    }

    #[test]
    fn test_resolve_ipv4_addr_given_domain_name() {
        let addr = "http:google.com:80";
        let socket_addr = resolve_ipv4_addr(addr).unwrap();
        assert!(socket_addr.is_ipv4());
        assert_eq!(socket_addr.port(), 80);
    }

    #[test]
    fn test_resolve_ipv4_addr_given_ip_addr() {
        let addr = "udp:172.64.155.141:80";
        let socket_addr = resolve_ipv4_addr(addr).unwrap();
        assert!(socket_addr.is_ipv4());
        assert_eq!(socket_addr.ip(), IpAddr::from([172, 64, 155, 141]));
        assert_eq!(socket_addr.port(), 80);
    }

    #[test]
    fn test_resolve_ipv4_addr_given_malformed_addr() {
        let addr = "random text";
        let err = resolve_ipv4_addr(addr).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(err.to_string(), "Address should start with 'udp:' or 'http:'");
    }
}