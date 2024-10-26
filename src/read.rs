use crate::{
    geoip::{Cidr, GeoIpList},
    srs,
};
use anyhow::{bail, Context, Ok, Result};
use prost::Message as _;

use std::io::Read as _;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

trait ToCidr {
    fn to_cidr(&self) -> Result<Cidr>;
}
impl ToCidr for srs::IPv4CIDR {
    fn to_cidr(&self) -> Result<Cidr> {
        Ok(Cidr {
            ip: self.ip.into(),
            prefix: self.prefix.into(),
        })
    }
}

impl ToCidr for srs::IPv6CIDR {
    fn to_cidr(&self) -> Result<Cidr> {
        Ok(Cidr {
            ip: self.ip.into(),
            prefix: self.prefix.into(),
        })
    }
}

const MAX_RETRIES: usize = 3;

fn from_buffer(buffer: &mut [u8], country_code: &str) -> Result<(Vec<Cidr>, Vec<Cidr>)> {
    {
        let result_ptr = unsafe { srs::read_cidr_rule(buffer.as_mut_ptr(), buffer.len() as u32) };

        if !result_ptr.is_null() {
            use libc::free;
            let (mut ipv4_cidrs, mut ipv6_cidrs): (Vec<Cidr>, Vec<Cidr>) = (Vec::new(), Vec::new());
            let result = unsafe { &*result_ptr };
            let ipv4_count = result.ipv4_count;
            let ipv6_count = result.ipv6_count;
            if result.ipv4_count != 0 {
                let ipv4_list =
                    unsafe { std::slice::from_raw_parts(result.ipv4_list, ipv4_count as usize) };
                for ipv4 in ipv4_list {
                    ipv4_cidrs.push(ipv4.to_cidr()?)
                }
                unsafe { free(result.ipv4_list as *mut libc::c_void) }
            }
            if result.ipv6_count != 0 {
                let ipv6_list =
                    unsafe { std::slice::from_raw_parts(result.ipv6_list, ipv6_count as usize) };
                for ipv6 in ipv6_list {
                    ipv6_cidrs.push(ipv6.to_cidr()?)
                }
                unsafe { free(result.ipv6_list as *mut libc::c_void) }
            }
            unsafe { free(result_ptr as *mut libc::c_void) }
            return Ok((ipv4_cidrs, ipv6_cidrs));
        }
    }
    if country_code == "NULL" {
        bail!("country_code == \"NULL\"");
    }
    let geoip_list = GeoIpList::decode(&*buffer).context("Failed to decode GeoIpList")?;
    let (ipv4_cidrs, ipv6_cidrs): (Vec<Cidr>, Vec<Cidr>) = geoip_list
        .entry
        .iter()
        .filter(|geoip| geoip.country_code == country_code)
        .flat_map(|geoip| geoip.cidr.iter().cloned())
        .partition(|c| c.ip.len() == 4);
    drop(geoip_list);

    Ok((ipv4_cidrs, ipv6_cidrs))
}

pub fn fetch(country_code: &str, url: &str) -> Result<(Vec<Cidr>, Vec<Cidr>)> {
    let mut buffer = Vec::new();
    let mut retries = 0;

    loop {
        buffer.clear();

        let mut curl = curl::easy::Easy::new();
        {
            curl.url(url)?;
            curl.follow_location(true)?;
            curl.timeout(Duration::from_secs(10))?;
            let mut transfer = curl.transfer();
            transfer.write_function(|data| {
                buffer.extend_from_slice(data);
                std::result::Result::Ok(data.len())
            })?;

            if let Err(err) = transfer.perform() {
                if retries == MAX_RETRIES {
                    return Err(anyhow::Error::new(err));
                }
                retries += 1;
                sleep(Duration::from_secs(3));
                continue;
            }
        }

        let status_code = curl.response_code()?;
        if status_code != 200 {
            bail!("HTTP Status Code: {}", status_code)
        }
        if !buffer.is_empty() {
            break;
        }
        retries += 1;
    }
    from_buffer(&mut buffer, country_code)
}

pub fn from_file(country_code: &str, path: PathBuf) -> Result<(Vec<Cidr>, Vec<Cidr>)> {
    let mut buffer = Vec::new();
    std::fs::File::open(path)?.read_to_end(&mut buffer)?;
    from_buffer(&mut buffer, country_code)
}
