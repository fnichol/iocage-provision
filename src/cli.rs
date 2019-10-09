// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ipnet::IpNet;
use nix::sys::utsname;
use std::error;
use std::fmt;
use std::io;
use std::net::{self, IpAddr};
use std::process::Command;
use std::str;
use structopt::{clap, StructOpt};

lazy_static::lazy_static! {
    /// The computed default value for the gateway option.
    static ref DEFAULT_GATEWAY: String = default_gateway();

    /// The computed default value for the release option.
    static ref DEFAULT_RELEASE: String = default_release();
}

/// The "author" string for help messages.
const AUTHOR: &str = concat!(env!("CARGO_PKG_AUTHORS"), "\n\n");

/// The "about" string for help messages
const ABOUT: &str = concat!(
    "\
Creates an iocage based FreeBSD jail.

Project home page: ",
    env!("CARGO_PKG_HOMEPAGE"),
    "

Use -h for short descriptions and --help for more details.",
);

/// The "long_about" string for help messages.
const LONG_ABOUT: &str = concat!(
    "\
Creates an iocage based FreeBSD jail.

This program uses iocage to create a VNET networked ZFS-backed FreeBSD jail. Suitable defaults are
computed for the default gateway and base release to reduce the number of arguments in the common
case. An optional --ssh flag will install and start an SSH service when the jail boots for remote
management. Finally, an optional --user option will create a user in the new jail by copying values
from the outside/host system.

Project home page: ",
    env!("CARGO_PKG_HOMEPAGE"),
    "

Use -h for short descriptions and --help for more details.",
);

/// An examples section at the end of the help message.
const AFTER_HELP: &str = "\
EXAMPLES:
    Example 1 Provisioning a New Jail With a Name and Address

      The following command will create a new jail called ferris with an IP
      address/subnet mask of 192.168.0.100/24.

        # iocage-provision ferris 192.168.0.100/24

    Example 2 Provisioning a New Jail With a User and SSH Service

      The following command will create a new jail with a running SSH service,
      and a user called jdoe which is copied from the host system (note that
      the user must exist on the host system).

        # iocage-provision --user jdoe --ssh homebase 10.0.0.25/24

    Example 3 Using a Custom Default Gateway and Base Release

      The following command will create a new jail by overriding the default
      gateway and default base release values.

        # iocage-provision --gateway 10.1.0.254 --release 11.1-RELEASE \\
          bespoke 10.1.0.1/24

";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn from_args() -> Args {
    Args::from_clap(
        &Args::clap()
            // TODO: StructOpt generates a Clap app with an unconditional
            // `.version(env!("CARGO_PKG_VERSION"))` at the end of the builder chain which
            // overrides any values inserted in the proc macro. Until this behavior can be
            // fixed, this is a temporary workaround which wraps the underlying `App` type and
            // chains on a call to `version`.
            .version(BuildInfo::version_short())
            .get_matches(),
    )
}

/// The parsed CLI arguments.
#[derive(Debug, StructOpt)]
#[structopt(
    global_settings(&[clap::AppSettings::UnifiedHelpMessage]),
    max_term_width = 100,
    author = AUTHOR,
    about = ABOUT,
    long_about = LONG_ABOUT,
    version = BuildInfo::version_short(),
    long_version = BuildInfo::version_long(),
    after_help = AFTER_HELP,
)]
pub(crate) struct Args {
    /// IP address of the default gateway route for a VNET.
    ///
    /// This address is used when setting up the VNET networking of the jail. If not provided the
    /// default value will be the address corresponding to the default route on the underlying host
    /// as determined by using the `netstat` program.
    #[structopt(
        short = "g",
        long,
        default_value = &DEFAULT_GATEWAY,
        rename_all = "screaming-snake",
    )]
    pub(crate) gateway: IpAddr,

    /// IP address & subnet mask for the jail instance. [example: 10.200.0.50/24]
    ///
    /// The IP address and the subnet mask are both required for the value to be considered valid.
    #[structopt(index = 2, rename_all = "screaming-snake")]
    pub(crate) ip: IpNet,

    /// Name for the jail instance [example: myjail]
    #[structopt(index = 1, rename_all = "screaming-snake")]
    pub(crate) name: String,

    /// FreeBSD release to use for the jail instance.
    ///
    /// If not provided, the default value will be the same release version that is running on the
    /// underlying host system. For example if `uname -r` returns `11.2-STABLE`, then the default
    /// value would be `11.2-RELEASE`.
    #[structopt(
        short = "R",
        long,
        default_value = &DEFAULT_RELEASE,
        rename_all = "screaming-snake",
    )]
    pub(crate) release: String,

    /// Installs and sets up an SSH service.
    ///
    /// If this flag is set, then SSH software is installed, enabled on boot and is started on
    /// first boot. Useful for jails that required remote administration, remote file copying, etc.
    #[structopt(short = "s", long)]
    pub(crate) ssh: bool,

    /// User to create in jail instance (based on host system's information).
    ///
    /// When this option is used, a user account will be created in the new jail with settings
    /// copied from the underlying system's `passwd` database. In other words, the username
    /// provided must exist on the host system, otherwise the command will result in an error and
    /// the jail will not be created.
    #[structopt(short = "u", long, rename_all = "screaming-snake")]
    pub(crate) user: Option<String>,

    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 3.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub(crate) verbose: usize,
}

/// A default gateway value.
fn default_gateway() -> String {
    netstat_gateway_addr()
        .unwrap_or_else(|err| {
            clap::Error::with_description(
                &format!("could not determine default gateway, {}", err.to_string()),
                clap::ErrorKind::Io,
            )
            .exit()
        })
        .to_string()
}

/// A default release value.
fn default_release() -> String {
    utsname::uname().release().replace("-STABLE", "-RELEASE")
}

/// Determines and returns a default gateway IP address by querying the `netstat` command.
///
/// # Errors
///
/// Returns an `Err` if:
///
/// * The `netstat` command cannot be found
/// * The output of the command cannot be parsed as UTF-8
/// * No line of output starting with `"default"` can be found
/// * The default line cannot be successfully split
/// * The IP address string cannot be parsed as an IP address
fn netstat_gateway_addr() -> Result<IpAddr, GatewayError> {
    str::from_utf8(
        Command::new("netstat")
            .args(&["-r", "-n", "-f", "inet"])
            .output()
            .map_err(GatewayError::Cmd)?
            .stdout
            .as_ref(),
    )
    .map_err(GatewayError::Utf8)?
    .lines()
    .find(|line| line.starts_with("default"))
    .map(|line| line.split_ascii_whitespace())
    .ok_or(GatewayError::NetstatParse("netstat default line not found"))?
    .nth(1)
    .ok_or(GatewayError::NetstatParse(
        "netstat second column not found on default line",
    ))?
    .parse()
    .map_err(GatewayError::IpAddr)
}

/// Error when determining a default gateway IP address.
#[derive(Debug)]
enum GatewayError {
    /// A command cannot be found or run successfully.
    Cmd(io::Error),
    /// An IP address failed to be parsed.
    IpAddr(net::AddrParseError),
    /// The output of the `netstat` command failed to be parsed.
    NetstatParse(&'static str),
    /// A string failed to be parsed as UTF-8.
    Utf8(str::Utf8Error),
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use GatewayError::*;

        match self {
            Cmd(ref err) => write!(f, "failed to run netstat command ({})", err),
            IpAddr(ref err) => err.fmt(f),
            NetstatParse(ref msg) => f.write_str(msg),
            Utf8(ref err) => err.fmt(f),
        }
    }
}

impl error::Error for GatewayError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use GatewayError::*;

        match self {
            Cmd(ref err) => err.source(),
            IpAddr(ref err) => err.source(),
            NetstatParse(_) => None,
            Utf8(ref err) => err.source(),
        }
    }
}

/// Build time metadata.
struct BuildInfo;

impl BuildInfo {
    /// Returns a short version string.
    fn version_short() -> &'static str {
        include_str!(concat!(env!("OUT_DIR"), "/version_short.txt"))
    }

    /// Returns a long version string.
    fn version_long() -> &'static str {
        include_str!(concat!(env!("OUT_DIR"), "/version_long.txt"))
    }
}

pub(crate) mod util {
    use chrono::{SecondsFormat, Utc};
    use std::env;
    use std::panic;

    /// The logger.
    const LOGGER: &Logger = &Logger;

    /// A custom and minimal `Log` implementation.
    ///
    /// This logger writes either to the standard output stream or standard error stream, depending
    /// on the log level.
    ///
    /// Thanks to the logger implementations from ripgrep and the simplelog crate which served as
    /// an inspiration.
    struct Logger;

    impl log::Log for Logger {
        fn enabled(&self, _: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            if log::max_level() == log::LevelFilter::Info {
                match record.level() {
                    log::Level::Info => println!("  - {}", record.args()),
                    log::Level::Warn => eprintln!("!!! {}", record.args()),
                    log::Level::Error => eprintln!("xxx {}", record.args()),
                    _ => unreachable!("illegal log level"),
                }
            } else {
                let file = record.file().unwrap_or("<unknown>");
                let location = match record.line() {
                    Some(line) => format!("{}:{}", file, line),
                    None => format!("{}:<unknown>", file),
                };

                match record.level() {
                    log::Level::Info => {
                        println!(
                            "{} {:<5} [{}] {}",
                            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
                            record.level(),
                            location,
                            record.args()
                        );
                    }
                    _ => {
                        eprintln!(
                            "{} {:<5} [{}] {}",
                            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
                            record.level(),
                            location,
                            record.args()
                        );
                    }
                }
            }
        }

        fn flush(&self) {
            // `eprintln!` and `println!` flush on every call
        }
    }

    /// Sets up and initializes the logger.
    pub(crate) fn init_logger(verbosity: usize) {
        log::set_logger(LOGGER).expect("error setting logger");

        match verbosity {
            0 => log::set_max_level(log::LevelFilter::Info),
            1 => log::set_max_level(log::LevelFilter::Debug),
            v if v >= 2 => log::set_max_level(log::LevelFilter::Trace),
            _ => {}
        }
        log::debug!("verbosity={}", verbosity);
    }

    /// Wires up a human-first experience if the program panics unexpectedly and also supports the
    /// normal `RUST_BACKTRACE` environment variable.
    ///
    /// A big thanks to https://github.com/rustwasm/wasm-pack for demonstrating such a delightful
    /// pattern. All credit here goes to the wasm-pack crew, thanks!
    pub(crate) fn setup_panic_hooks() {
        let meta = human_panic::Metadata {
            version: env!("CARGO_PKG_VERSION").into(),
            name: env!("CARGO_PKG_NAME").into(),
            authors: env!("CARGO_PKG_AUTHORS").into(),
            homepage: env!("CARGO_PKG_HOMEPAGE").into(),
            repository: option_env!("CARGO_PKG_REPOSITORY").unwrap_or("").into(),
        };

        let default_hook = panic::take_hook();

        if env::var("RUST_BACKTRACE").is_err() {
            panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
                // First call the default hook that prints to standard error
                default_hook(info);

                // Then call human panic
                let file_path = human_panic::handle_dump(&meta, info);
                human_panic::print_msg(file_path, &meta)
                    .expect("human-panic: printing error message to console failed");
            }));
        }
    }

    /// Return a prettily formatted error, including its entire causal chain.
    ///
    /// Thanks again to the imdb-rename crate and wasm-pack which form the basis of this
    /// implementation.
    pub(crate) fn pretty_error(err: &failure::Error) -> String {
        let mut pretty = "Error: ".to_string();
        pretty.push_str(&err.to_string());
        pretty.push_str("\n");
        for cause in err.iter_causes() {
            pretty.push_str("Caused by: ");
            pretty.push_str(&cause.to_string());
            pretty.push_str("\n");
        }
        pretty
    }
}
