use crate::{geoip::*, srs};
use anyhow::{bail, Ok, Result};
use prost::Message as _;

pub fn to_srs(cidr_pair: (Vec<Cidr>, Vec<Cidr>)) -> Result<Vec<u8>> {
    use srs::{CIDRList, IPv4CIDR, IPv6CIDR};
    let (ipv4_cidrs, ipv6_cidrs) = cidr_pair;
    let mut ipv4_vec: Vec<IPv4CIDR> = Vec::new();
    let mut ipv6_vec: Vec<IPv6CIDR> = Vec::new();
    for cidr in ipv4_cidrs {
        let ip = <[u8; 4]>::try_from(cidr.ip.as_slice())?;
        let prefix = cidr.prefix as u8;
        ipv4_vec.push(IPv4CIDR { ip, prefix });
    }
    for cidr in ipv6_cidrs {
        let ip = <[u8; 16]>::try_from(cidr.ip.as_slice())?;
        let prefix = cidr.prefix as u8;
        ipv6_vec.push(IPv6CIDR { ip, prefix });
    }
    let ipv4_count = ipv4_vec.len() as u32;
    let ipv6_count = ipv6_vec.len() as u32;
    let ipv4_list = ipv4_vec.as_mut_ptr();
    let ipv6_list = ipv6_vec.as_mut_ptr();
    let cidr_list = Box::new(CIDRList {
        ipv4_list,
        ipv4_count,
        ipv6_list,
        ipv6_count,
    });
    let cidr_list_ptr = Box::into_raw(cidr_list);
    let mut length = 0_u32;
    let length_ptr: *mut u32 = &mut length;
    let data_ptr = unsafe {
        let x = srs::write_cidr_rule(cidr_list_ptr, length_ptr);
        drop(Box::from_raw(cidr_list_ptr));
        x
    };
    if data_ptr.is_null() {
        bail!("data_ptr.is_null()")
    }
    Ok(unsafe { Vec::from_raw_parts(data_ptr, length as usize, length as usize) })
}

pub fn to_ray(cidr_pair: (Vec<Cidr>, Vec<Cidr>), country_code: &str) -> Result<Vec<u8>> {
    if country_code == "NULL" {
        bail!("country_code == \"NULL\"");
    }
    let (ipv4_cidrs, ipv6_cidrs) = cidr_pair;
    let mut buffer = Vec::new();
    let mut geoip_list = GeoIpList::default();
    let geoip_entry = GeoIp {
        country_code: country_code.to_string().to_ascii_uppercase(),
        cidr: ipv4_cidrs.into_iter().chain(ipv6_cidrs).collect(),
        reverse_match: false,
    };
    geoip_list.entry.push(geoip_entry);
    geoip_list.encode(&mut buffer)?;
    Ok(buffer)
}
