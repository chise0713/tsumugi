use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(subcommand_required = true, arg_required_else_help = true)]
#[command(
    version,
    about = "Simple tool for interactive with *ray geoip.dat and sing-box ruleset"
)]
pub struct App {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[clap(flatten)]
    pub source_group: SourceGroup,

    /// Country code
    #[arg(short, long, global = true)]
    pub code: Option<Box<str>>,

    /// Output path
    #[arg(short, long, global = true)]
    pub output: Option<Box<str>>,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct SourceGroup {
    /// Url of the file to download
    #[arg(short, long, global = true)]
    pub file: Option<Box<str>>,

    /// Path of the file to read
    #[arg(short, long, global = true)]
    pub url: Option<Box<str>>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Generate things, e.g. nftables script")]
    Generate {
        #[command(subcommand)]
        generate_command: GenerateCommands,
    },

    #[command(about = "Convert from one format to another")]
    Convert {
        #[command(subcommand)]
        convert_command: ConvertCommands,

        /// Output path
        #[arg(short, long, required = true)]
        output: Box<str>,
    },

    #[command(about = "Generate a systemd service unit")]
    Systemd {
        #[command(subcommand)]
        generate_command: GenerateCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum GenerateCommands {
    #[command(about = "Generate a nftables script")]
    Nftables {
        /// Table name
        #[arg(short, long)]
        table: Box<str>,

        /// IPv4 set name
        #[arg(short = '4', long)]
        ipv4set: Box<str>,

        /// IPv6 set name
        #[arg(short = '6', long)]
        ipv6set: Box<str>,
    },

    #[command(about = "Generate a iproute2 script")]
    Iproute2 {
        #[command(subcommand)]
        iproute2_command: Iproute2Commands,
    },
}

#[derive(Subcommand, Debug)]
pub enum Iproute2Commands {
    #[command(about = "Generate a iproute2 routing policy rule script")]
    Rule {
        /// Turn on delete mode
        #[arg(long, default_value = "false")]
        delete: bool,

        /// Table name
        #[arg(short, long, default_value = "main")]
        table: Box<str>,
    },
    #[command(about = "Generate a iproute2 route table script")]
    Route {
        /// Turn on delete mode
        #[arg(long, default_value = "false")]
        delete: bool,

        /// Table name
        #[arg(short, long, default_value = "main")]
        table: Box<str>,

        /// IPv4 Gateway address
        #[arg(short = '4', long)]
        ipv4_gateway: Box<str>,

        /// IPv6 Gateway address
        #[arg(short = '6', long)]
        ipv6_gateway: Box<str>,

        /// Route device
        #[arg(short, long)]
        dev: Box<str>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConvertCommands {
    #[command(about = "Convert from source to sing-box rule-set")]
    Srs {},
    #[command(about = "Convert from source to *ray geoip.dat")]
    Ray {},
}
