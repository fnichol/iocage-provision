// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{AppSettings, Clap};
use ipnet::IpNet;
use std::net::IpAddr;
use std::str;

lazy_static::lazy_static! {
    /// The computed default value for the gateway option.
    static ref DEFAULT_GATEWAY: String = default_gateway();

    /// The computed default value for the release option.
    static ref DEFAULT_RELEASE: String = default_release();
}

const AFTER_HELP: &str =
    "Note: Use `-h` for a short and concise overview and `--help` for full usage.";

/// An examples section at the end of the help message.
const AFTER_LONG_HELP: &str = concat!(
    include_str!("cli_examples.txt"),
    "\n\n",
    "Note: Use `-h` for a short and concise overview and `--help` for full usage."
);

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// Creates an iocage based FreeBSD jail.
///
/// This program uses iocage to create a VNET networked ZFS-backed FreeBSD jail. Suitable defaults
/// are computed for the default gateway and base release to reduce the number of arguments in the
/// common case. An optional --ssh flag will install and start an SSH service when the jail boots
/// for remote management. Finally, an optional --user option will create a user in the new jail by
/// copying values from the outside/host system.
///
/// Project home page: https://github.com/fnichol/iocage-provision
///
/// Use -h for short descriptions and --help for more details.
#[derive(Clap, Debug)]
#[clap(
    global_setting(AppSettings::UnifiedHelpMessage),
    max_term_width = 100,
    author = concat!("\nAuthor: ", env!("CARGO_PKG_AUTHORS"), "\n\n"),
    version = BuildInfo::version_short(),
    long_version = BuildInfo::version_long(),
    after_help = AFTER_HELP,
    after_long_help = AFTER_LONG_HELP,
)]
pub(crate) struct Args {
    /// IP address of the default gateway route for a VNET.
    ///
    /// This address is used when setting up the VNET networking of the jail. If not provided the
    /// default value will be the address corresponding to the default route on the underlying host
    /// as determined by using the `netstat` program.
    #[clap(
        short = 'g',
        long,
        default_value = &DEFAULT_GATEWAY,
        rename_all = "screaming-snake",
    )]
    pub(crate) gateway: IpAddr,

    /// IP address & subnet mask for the jail instance. [example: 10.200.0.50/24]
    ///
    /// The IP address and the subnet mask are both required for the value to be considered valid.
    #[clap(index = 2, rename_all = "screaming-snake")]
    pub(crate) ip: IpNet,

    /// Name for the jail instance [example: myjail]
    #[clap(index = 1, rename_all = "screaming-snake")]
    pub(crate) name: String,

    /// FreeBSD release to use for the jail instance.
    ///
    /// If not provided, the default value will be the same release version that is running on the
    /// underlying host system. For example if `uname -r` returns `11.2-STABLE`, then the default
    /// value would be `11.2-RELEASE`.
    #[clap(
        short = 'R',
        long,
        default_value = &DEFAULT_RELEASE,
        rename_all = "screaming-snake",
    )]
    pub(crate) release: String,

    /// Installs and sets up an SSH service.
    ///
    /// If this flag is set, then SSH software is installed, enabled on boot and is started on
    /// first boot. Useful for jails that required remote administration, remote file copying, etc.
    #[clap(short = 's', long)]
    pub(crate) ssh: bool,

    /// User to create in jail instance (based on host system's information).
    ///
    /// When this option is used, a user account will be created in the new jail with settings
    /// copied from the underlying system's `passwd` database. In other words, the username
    /// provided must exist on the host system, otherwise the command will result in an error and
    /// the jail will not be created.
    #[clap(short = 'u', long, rename_all = "screaming-snake")]
    pub(crate) user: Option<String>,

    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 3.
    #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
    pub(crate) verbose: usize,
}

/// A default gateway value.
fn default_gateway() -> String {
    iocage_provision::netstat_gateway_addr()
        .unwrap_or_else(|err| {
            clap::Error::with_description(
                format!("could not determine default gateway; err={}", err),
                clap::ErrorKind::Io,
            )
            .exit()
        })
        .to_string()
}

/// A default release value.
fn default_release() -> String {
    iocage_provision::default_release()
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
    pub(crate) fn init_logger_with_verbosity(verbosity: usize) {
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
            version: super::BuildInfo::version_short().into(),
            name: env!("CARGO_BIN_NAME").into(),
            authors: env!("CARGO_PKG_AUTHORS").into(),
            homepage: env!("CARGO_PKG_HOMEPAGE").into(),
        };

        if env::var("RUST_BACKTRACE").is_err() {
            let default_hook = panic::take_hook();

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
}
