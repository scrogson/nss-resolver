//! Simple top level domain (NSSwitch hosts) resolver for a Linux-based
//! development environment.

#[macro_use] extern crate nsswitch_service;

use std::ffi::CStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use nsswitch_service::*;

#[derive(Debug)]
struct Resolver;

impl NameService for Resolver {
    fn gethostbyname2_r(name: &CStr, af: AddressFamily) -> Result<Option<HostEntry>> {
        use std::borrow::Cow;

        // Convert the C null-terminated string `name` to a Rust string.
        let name_str = match name.to_str() {
            Err(_) => return Ok(None), // `name` isn't valid UTF-8
            Ok(s) => s,
        };

        let tld = match name_str.rfind('.') {
            None => return Ok(None),
            Some(index) => &name_str[index + 1..],
        };

        let domains = std::env::var("NSS_RESOLVER_TLDS").unwrap_or_else(|_| "test".to_string());

        for domain in domains.split(',') {
            if tld.eq_ignore_ascii_case(domain) {
                return Ok(Some(HostEntry {
                    name: Cow::Borrowed(name),
                    aliases: vec![],
                    addr_list: match af {
                        AddressFamily::Ipv4 => HostAddressList::V4(vec![
                            Ipv4Addr::new(127, 0, 0, 1)
                        ]),
                        AddressFamily::Ipv6 => HostAddressList::V6(vec![
                            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)
                        ]),
                    }
                }));
            }
        }

        Ok(None)
    }

    fn gethostbyaddr_r(_addr: &IpAddr) -> Result<Option<HostEntry>> {
        Ok(None)
    }
}

nssglue_gethostbyname_r!(_nss_resolver_gethostbyname_r, Resolver);
nssglue_gethostbyname2_r!(_nss_resolver_gethostbyname2_r, Resolver);
nssglue_gethostbyaddr_r!(_nss_resolver_gethostbyaddr_r, Resolver);
