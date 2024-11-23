mod app;
mod convert;
mod generate;
mod read;
mod srs {
    include!(concat!(env!("OUT_DIR"), "/libsrs.rs"));
}
mod systemd;
mod geoip {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

use anyhow::Result;
use app::*;
use clap::{CommandFactory as _, FromArgMatches as _};

use std::fs::File;
use std::io::Write as _;

fn main() -> Result<()> {
    let mut cmd = App::command();
    cmd.build();
    let args = App::from_arg_matches(&cmd.clone().get_matches())?;
    let (code, url, mut output) = (
        args.code.unwrap_or_default(),
        args.source_group.url.unwrap_or_default(),
        args.output.unwrap_or_default(),
    );
    let (mut nftables, mut nf_table, mut nf_ipv4set, mut nf_ipv6set) =
        (false, Box::from(""), Box::from(""), Box::from(""));
    let (mut iproute2_rule, mut iproute2_route) = (false, false);
    let (mut delete, mut ru_table, mut r_table, mut r_ipv4_gateway, mut r_ipv6_gateway, mut r_dev) = (
        false,
        Box::from(""),
        Box::from(""),
        Box::from(""),
        Box::from(""),
        Box::from(""),
    );
    let mut systemd = false;
    let (mut to_srs, mut to_ray) = (false, false);
    match args.command {
        Some(Commands::Generate {
            generate_command: c,
        }) => match c {
            GenerateCommands::Nftables {
                table,
                ipv4set,
                ipv6set,
            } => {
                nftables = true;
                nf_table = table;
                nf_ipv4set = ipv4set;
                nf_ipv6set = ipv6set;
            }
            GenerateCommands::Iproute2 {
                iproute2_command: c,
            } => match c {
                Iproute2Commands::Rule { delete: d, table } => {
                    delete = d;
                    iproute2_rule = true;
                    ru_table = table;
                }
                Iproute2Commands::Route {
                    delete: d,
                    table,
                    ipv4_gateway,
                    ipv6_gateway,
                    dev,
                } => {
                    delete = d;
                    iproute2_route = true;
                    r_table = table;
                    r_ipv4_gateway = ipv4_gateway;
                    r_ipv6_gateway = ipv6_gateway;
                    r_dev = dev;
                }
            },
        },
        Some(Commands::Convert {
            convert_command: c,
            output: o,
        }) => {
            output = o;
            match c {
                ConvertCommands::Srs {} => to_srs = true,
                ConvertCommands::Ray {} => to_ray = true,
            }
        }
        Some(Commands::Systemd {
            generate_command: c,
            ..
        }) => {
            systemd = true;
            match c {
                GenerateCommands::Nftables {
                    table,
                    ipv4set,
                    ipv6set,
                } => {
                    nftables = true;
                    nf_table = table;
                    nf_ipv4set = ipv4set;
                    nf_ipv6set = ipv6set;
                }
                GenerateCommands::Iproute2 {
                    iproute2_command: c,
                } => match c {
                    Iproute2Commands::Rule { delete: _, table } => {
                        delete = false;
                        iproute2_rule = true;
                        ru_table = table;
                    }
                    Iproute2Commands::Route {
                        delete: _,
                        table,
                        ipv4_gateway,
                        ipv6_gateway,
                        dev,
                    } => {
                        delete = false;
                        iproute2_route = true;
                        r_table = table;
                        r_ipv4_gateway = ipv4_gateway;
                        r_ipv6_gateway = ipv6_gateway;
                        r_dev = dev;
                    }
                },
            }
        }
        None => {
            unreachable!()
        }
    }
    let print = output.is_empty();
    let s: Box<str>;
    let buffer: Box<[u8]>;

    if systemd {
        if url.is_empty() {
            cmd.subcommand_value_name("systemd")
                .error(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "When using systemd subcommand, you should use --url:\n  \x1b[32m<--url <URL>>\x1b[0m",
                )
                .exit();
        }
        s = if nftables {
            systemd::generate_nftables(&url, &code, &nf_table, &nf_ipv4set, &nf_ipv6set)?
        } else if iproute2_route {
            systemd::generate_iproute2_route(
                &url,
                &code,
                &r_table,
                &r_ipv4_gateway,
                &r_ipv6_gateway,
                &r_dev,
            )?
        } else if iproute2_rule {
            systemd::generate_iproute2_rule(&url, &code, &ru_table)?
        } else {
            unreachable!()
        };
        buffer = if !print {
            s.as_bytes().into()
        } else {
            vec![].into()
        }
    } else {
        let cidr_pair = if !url.is_empty() {
            read::fetch(&code, &url)?
        } else if let Some(file) = &args.source_group.file {
            read::from_file(&code, file.parse()?)?
        } else {
            unreachable!()
        };

        if nftables || iproute2_route || iproute2_rule {
            s = if nftables {
                generate::nftables(cidr_pair, &nf_table, &nf_ipv4set, &nf_ipv6set)?
            } else if iproute2_rule {
                generate::iproute2rule(cidr_pair, delete, &ru_table)?
            } else {
                generate::iproute2route(
                    cidr_pair,
                    delete,
                    &r_table,
                    &r_ipv4_gateway,
                    &r_ipv6_gateway,
                    &r_dev,
                )?
            };
            buffer = if !print {
                s.as_bytes().into()
            } else {
                vec![].into()
            }
        } else {
            s = Box::from("");
            buffer = if to_srs {
                convert::to_srs(cidr_pair)?
            } else if to_ray {
                convert::to_ray(cidr_pair, &code)?
            } else {
                unreachable!()
            };
        }
    }

    if print {
        print!("{}", s);
    } else {
        File::create(&*output)?.write_all(&buffer)?;
    }

    Ok(())
}
