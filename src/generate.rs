use crate::geoip::Cidr;
use anyhow::{bail, Context as _, Ok, Result};

use std::{
    fmt::Write as _,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

trait ToString {
    fn to_string(&self) -> Result<String>;
}
impl ToString for Cidr {
    fn to_string(&self) -> Result<String> {
        let ip_str = match self.ip.len() {
            4 => {
                let ip = Ipv4Addr::from(
                    <[u8; 4]>::try_from(self.ip.as_slice())
                        .context("Failed to convert to Ipv4Addr")?,
                );
                IpAddr::V4(ip)
            }
            16 => {
                let ip = Ipv6Addr::from(
                    <[u8; 16]>::try_from(self.ip.as_slice())
                        .context("Failed to convert to Ipv6Addr")?,
                );
                IpAddr::V6(ip)
            }
            _ => bail!("Invalid IP length"),
        };

        Ok(format!("{}/{}", ip_str, self.prefix))
    }
}

pub fn nftables(
    cidr_pair: (Vec<Cidr>, Vec<Cidr>),
    table: &str,
    ipv4set: &str,
    ipv6set: &str,
) -> Result<String> {
    let (ipv4_cidrs, ipv6_cidrs) = cidr_pair;
    let mut script = String::new();

    for (set, elems) in &[(ipv4set, &ipv4_cidrs), (ipv6set, &ipv6_cidrs)] {
        writeln!(script, "flush set inet {} {}", table, set)?;
        let mut elem_iter = elems.iter();
        if let Some(first_elem) = elem_iter.next() {
            write!(
                script,
                "add element inet {} {} {{ {}",
                table,
                set,
                first_elem.to_string()?
            )?;
            for elem in elem_iter {
                write!(script, ", {}", elem.to_string()?)?;
            }
            writeln!(script, " }}")?;
        } else {
            writeln!(script, "add element inet {} {} {{ }}", table, set)?;
        }
    }
    Ok(script)
}

pub fn iproute2rule(
    cidr_pair: (Vec<Cidr>, Vec<Cidr>),
    delete: bool,
    table: &str,
) -> Result<String> {
    let (ipv4_cidrs, ipv6_cidrs) = cidr_pair;
    let mut script = String::new();
    let action = if delete { "delete" } else { "add" };
    for elem in ipv4_cidrs {
        writeln!(
            script,
            "ip rule {} to {} lookup {}",
            action,
            elem.to_string()?,
            table
        )?;
    }
    for elem in ipv6_cidrs {
        writeln!(
            script,
            "ip -6 rule {} to {} lookup {}",
            action,
            elem.to_string()?,
            table
        )?;
    }
    Ok(script)
}

pub fn iproute2route(
    cidr_pair: (Vec<Cidr>, Vec<Cidr>),
    delete: bool,
    table: &str,
    ipv4_gateway: &str,
    ipv6_gateway: &str,
    dev: &str,
) -> Result<String> {
    let (ipv4_cidrs, ipv6_cidrs) = cidr_pair;
    let mut script = String::new();
    let action = if delete { "delete" } else { "add" };
    for elem in ipv4_cidrs {
        writeln!(
            script,
            "ip route {} table {} {} via {} dev {}",
            action,
            table,
            elem.to_string()?,
            ipv4_gateway,
            dev
        )?;
    }
    for elem in ipv6_cidrs {
        writeln!(
            script,
            "ip -6 route {} table {} {} via {} dev {}",
            action,
            table,
            elem.to_string()?,
            ipv6_gateway,
            dev
        )?;
    }
    Ok(script)
}
