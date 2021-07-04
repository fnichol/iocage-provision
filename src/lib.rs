// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![doc(html_root_url = "https://docs.rs/iocage-provision/0.1.2-dev")]
//#![deny(missing_docs)]

use ipnet::IpNet;
use log::{debug, info};
use nix::sys::utsname;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{self, IpAddr};
use std::path::Path;
use std::process::{ChildStdin, Command, ExitStatus, Stdio};
use std::result;
use std::str;
use std::thread;
use tempfile::NamedTempFile;
use users::{os::unix::UserExt, Group, User};

macro_rules! section {
    ($($arg:tt)+) => (
        if log::max_level() == log::LevelFilter::Info {
            println!("--- {}", format!($($arg)+));
        } else {
            log::info!($($arg)+);
        }
    )
}

macro_rules! output {
    ($($arg:tt)+) => (
        if log::max_level() == log::LevelFilter::Info {
            println!("        {}", format!($($arg)+));
        } else {
            log::info!($($arg)+);
        }
    )
}

macro_rules! eoutput {
    ($($arg:tt)+) => (
        if log::max_level() == log::LevelFilter::Info {
            eprintln!("        {}", format!($($arg)+));
        } else {
            log::warn!($($arg)+);
        }
    )
}

/// A specialized `Result` type for this crate's operations.
pub type Result<T> = result::Result<T, Error>;

/// Error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not generate json pkglist tempfile")]
    CreatePkglistJson(#[source] io::Error),
    #[error("failed to create user group")]
    ExecCreateGroup(#[source] IocageExecError),
    #[error("failed to create user")]
    ExecCreateUser(#[source] IocageExecError),
    #[error("failed to enable an SSH service")]
    ExecSshService(#[source] IocageExecError),
    #[error("failed to prepare sudo config")]
    ExecSudoConfig(#[source] IocageExecError),
    #[error("failed to create iocage jail")]
    IocageCreate(#[source] CmdError),
    /// A system group ID was not found.
    #[error("system group id not found; gid={0}")]
    NoGid(u32),
    /// The effective user is not currently the `root` user.
    #[error("root privileges required")]
    NotRoot,
    /// A system user name was not found.
    #[error("system user not found; user={0}")]
    NoUser(String),
}

#[derive(Debug, thiserror::Error)]
pub enum CmdError {
    #[error("spawned command did not start")]
    ChildWait(#[source] io::Error),
    /// A command returned a non-zero exit code and thus is considered to have failed.
    #[error("command exited with non-zero code; code={0}")]
    Failed(i32),
    #[error("command failed to spawn; program={0}")]
    Spawn(String, #[source] io::Error),
    #[error("stream was not captured; stream={0}")]
    StreamCapture(&'static str),
    #[error("io stream thread panicked; stream={0}")]
    Thread(&'static str),
    #[error("failed to write to stdin")]
    StdinWrite(#[source] io::Error),
}

/// Error when an iocage exec command fails.
#[derive(Debug, thiserror::Error)]
#[error("iocage exec command failed")]
pub struct IocageExecError(#[from] CmdError);

/// Error when determining a default gateway IP address.
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    /// A command cannot be found or run successfully.
    #[error("failed to successfully run netstat command; err={0}")]
    Cmd(#[source] io::Error),
    /// An IP address failed to be parsed.
    #[error("failed to parse ip address")]
    IpAddr(#[source] net::AddrParseError),
    /// The output of the `netstat` command failed to be parsed.
    #[error("failed to parse netstat output; cause={0}")]
    NetstatParse(&'static str),
    /// A string failed to be parsed as UTF-8.
    #[error("utf8 error; err={0}")]
    Utf8(#[source] str::Utf8Error),
}

/// Ensures that the current effective user is root.
///
/// # Errors
///
/// Returns an `Err` if the current effective `uid` is any value other than `0`.
pub fn ensure_root() -> Result<()> {
    if users::get_effective_uid() != 0 {
        Err(Error::NotRoot)
    } else {
        Ok(())
    }
}

/// Creates, starts, and sets up a new FreeBSD jail via the `iocage` program.
///
/// # Errors
///
/// Returns an `Err` if a jail could not be completely provisioned successfully. Note that a
/// failure from this function may leave behind a jail in an inconsistent state that needs to be
/// cleaned up out of band.
pub fn provision_jail(
    name: &str,
    ip: &IpNet,
    gateway: &IpAddr,
    release: &str,
    thick_jail: bool,
    user: Option<&str>,
    ssh_service: bool,
) -> Result<()> {
    let user = find_user(user)?;
    let json = create_pkglist_json(user.as_ref()).map_err(Error::CreatePkglistJson)?;

    section!("Provisioning a jail named '{}'", name);

    info!("Creating '{}' via iocage", name);
    run_iocage_create(name, ip, gateway, release, thick_jail, json.path())?;

    if let Some(user) = user {
        let group = find_group(user.primary_group_id())?;

        info!("Preparing sudo config");
        exec_sudo_config(name)?;

        info!("Creating group '{}'", group.name().to_string_lossy());
        exec_create_group(name, &group)?;

        info!("Creating user '{}'", user.name().to_string_lossy());
        exec_create_user(name, &user, &group)?;
    }

    if ssh_service {
        info!("Enabling SSH service");
        exec_ssh_service(name)?;
    }

    section!("Instance '{}' provisioned successfully", name);

    Ok(())
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
pub fn netstat_gateway_addr() -> result::Result<IpAddr, GatewayError> {
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
    .ok_or(GatewayError::NetstatParse("default line not found"))?
    .nth(1)
    .ok_or(GatewayError::NetstatParse(
        "second column not found on default line",
    ))?
    .parse()
    .map_err(GatewayError::IpAddr)
}

/// Returns a default release value based on the current host.
pub fn default_release() -> String {
    utsname::uname()
        .release()
        .split('-')
        .map(|s| if s == "STABLE" { "RELEASE" } else { s })
        .take(2)
        .collect::<Vec<_>>()
        .join("-")
}

/// Returns a `User` for a given name, if one exists.
///
/// If `None` is provided as an argument, then `Ok(None)` will be returned.
///
/// # Errors
///
/// Returns an `Err` if an associated system user cannot be found for the given user name.
fn find_user(user_str: Option<&str>) -> Result<Option<User>> {
    match user_str {
        Some(user_str) => match users::get_user_by_name(user_str) {
            Some(user_info) => Ok(Some(user_info)),
            None => Err(Error::NoUser(user_str.to_string())),
        },
        None => Ok(None),
    }
}

/// Returns a `Group` for a given group ID (i.e. `gid`).
///
/// # Errors
///
/// Returns an `Err` if an associated system group cannot be found for the given group ID.
fn find_group(gid: u32) -> Result<Group> {
    users::get_group_by_gid(gid).ok_or(Error::NoGid(gid))
}

/// Creates a package list JSON file for the `iocage create` subcommand and returns the file path.
///
/// # Errors
///
/// Returns an `Err` if the JSON file could not be successfully created and written.
fn create_pkglist_json(user: Option<&User>) -> io::Result<NamedTempFile> {
    let json_str = match user {
        Some(user) => {
            let shell = user
                .shell()
                .file_name()
                .unwrap_or_else(|| OsStr::new(""))
                .to_string_lossy();

            match shell.as_ref() {
                "bash" => r#"{"pkgs":["sudo","bash"]}"#,
                _ => r#"{"pkgs":["sudo"]}"#,
            }
        }
        None => r#"{"pkgs":[]}"#,
    };

    let json = tempfile::Builder::new()
        .prefix("pkglist")
        .suffix(".json")
        .rand_bytes(5)
        .tempfile()?;
    fs::write(json.path(), json_str.as_bytes())?;

    Ok(json)
}

/// Prepares the sudo config in the given jail.
///
/// # Errors
///
/// Returns an `Err` if the commands were not successfully executed in the jail.
fn exec_sudo_config(jail_name: &str) -> Result<()> {
    iocage_exec(
        jail_name,
        "echo '%wheel ALL=(ALL) NOPASSWD: ALL' >/usr/local/etc/sudoers.d/wheel",
    )
    .map_err(Error::ExecSudoConfig)
}

/// Creates a system group in the given jail.
///
/// # Errors
///
/// Returns an `Err` if the commands were not successfully executed in the jail.
fn exec_create_group(jail_name: &str, group: &Group) -> Result<()> {
    iocage_exec(
        jail_name,
        format!(
            "pw groupadd -n '{grp}' -g '{gid}'",
            gid = group.gid(),
            grp = group.name().to_string_lossy(),
        ),
    )
    .map_err(Error::ExecCreateGroup)
}

/// Creates a system user in the given jail.
///
/// # Errors
///
/// Returns an `Err` if the commands were not successfully executed in the jail.
fn exec_create_user(jail_name: &str, user: &User, group: &Group) -> Result<()> {
    iocage_exec(
        jail_name,
        format!(
            "pw useradd -n '{usr}' -u '{uid}' -g '{grp}' -G wheel -m -s '{shl}'",
            grp = group.name().to_string_lossy(),
            shl = user.shell().display(),
            uid = user.uid(),
            usr = user.name().to_string_lossy(),
        ),
    )
    .map_err(Error::ExecCreateUser)
}

/// Configures and starts an SSH service in the given jail.
///
/// # Errors
///
/// Returns an `Err` if the commands were not successfully executed in the jail.
fn exec_ssh_service(jail_name: &str) -> Result<()> {
    iocage_exec(
        jail_name,
        r#"sysrc -f /etc/rc.conf sshd_enable="YES" && service sshd start"#,
    )
    .map_err(Error::ExecSshService)
}

/// Creates a new jail with the given configuration.
///
/// # Errors
///
/// Returns an `Err` if the jail was not successfully created.
fn run_iocage_create(
    jail_name: &str,
    ip: &IpNet,
    gateway: &IpAddr,
    release: &str,
    thick_jail: bool,
    pkglist: &Path,
) -> Result<()> {
    let mut cmd = Command::new("iocage");
    cmd.arg("--force")
        .arg("create")
        .arg("--name")
        .arg(jail_name)
        .arg("--release")
        .arg(release)
        .arg("--pkglist")
        .arg(pkglist);
    if thick_jail {
        cmd.arg("--thickjail");
    }
    cmd.arg("vnet=on")
        .arg(format!("ip4_addr=vnet0|{}", ip))
        .arg(format!("defaultrouter={}", gateway))
        .arg("resolver=none")
        .arg("boot=on")
        .env("PYTHONUNBUFFERED", "true");

    let status = spawn_and_indent(cmd).map_err(Error::IocageCreate)?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::IocageCreate(CmdError::Failed(
            status.code().unwrap_or(-1),
        )))
    }
}

/// Executes a command or script of commands in the given jail.
///
/// # Errors
///
/// Returns an `Err` if:
///
/// * The input and output streams were not successfully set up
/// * The `iocage` program was not found
/// * The `iocage` exits with a code that is not zero
fn iocage_exec<S: AsRef<str>>(jail_name: &str, src: S) -> result::Result<(), IocageExecError> {
    let mut cmd = Command::new("iocage");
    cmd.arg("exec")
        .arg(jail_name)
        .arg("sh")
        // `iocage` is a Python program and will therefore buffer output when executed in a
        // non-interactive mode. Setting a value for the `PYTHONUNBUFFERED` environment variable
        // ensures that the output streams don't needlessly buffer.
        //
        // See: https://docs.python.org/2/using/cmdline.html#envvar-PYTHONUNBUFFERED
        .env("PYTHONUNBUFFERED", "true");

    let status = spawn_and_indent_with_stdin(cmd, |mut stdin| {
        stdin
            .write_all(b"set -eu\n\n")
            .map_err(CmdError::StdinWrite)?;
        stdin
            .write_all(src.as_ref().as_bytes())
            .map_err(CmdError::StdinWrite)?;
        Ok(())
    })?;

    if status.success() {
        Ok(())
    } else {
        Err(CmdError::Failed(status.code().unwrap_or(-1)).into())
    }
}

/// Spawns a `Command`, indents the output stream contents, and returns its `ExitStatus`.
///
/// # Errors
///
/// Returns an `Err` if:
///
/// * The command failed to spawn
/// * One of the I/O streams failed to be properly captured
/// * One of the output-reading threads panics
/// * The command wasn't running
fn spawn_and_indent(cmd: Command) -> result::Result<ExitStatus, CmdError> {
    spawn_and_indent_with_stdin(cmd, |_| Ok(()))
}

/// Spawns a `Command` with data for the standard input stream, indents the output stream contents,
/// and returns its `ExitStatus`.
///
/// # Errors
///
/// Returns an `Err` if:
///
/// * The command failed to spawn
/// * One of the I/O streams failed to be properly captured
/// * One of the output-reading threads panics
/// * The command wasn't running
fn spawn_and_indent_with_stdin<F>(
    mut cmd: Command,
    stdin_func: F,
) -> result::Result<ExitStatus, CmdError>
where
    F: FnOnce(ChildStdin) -> result::Result<(), CmdError>,
{
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    debug!("running; cmd={:?}", &cmd);
    let mut child = cmd
        .spawn()
        .map_err(|err| CmdError::Spawn(cmd_get_program(&cmd), err))?;

    {
        let stdin = child.stdin.take().ok_or(CmdError::StreamCapture("stdin"))?;
        stdin_func(stdin)?;
    }

    let stdout = BufReader::new(
        child
            .stdout
            .take()
            .ok_or(CmdError::StreamCapture("stdout"))?,
    );
    let stdout_handle = thread::spawn(move || {
        for line in stdout.lines() {
            // This error happens in a thread, so we will panic here on error
            output!("{}", line.expect("failed to read line from stdout"));
        }
    });

    let stderr = BufReader::new(
        child
            .stderr
            .take()
            .ok_or(CmdError::StreamCapture("stderr"))?,
    );
    let stderr_handle = thread::spawn(move || {
        for line in stderr.lines() {
            // This error happens in a thread, so we will panic here on error
            eoutput!("{}", line.expect("failed to read line from stderr"));
        }
    });

    let status = child.wait();

    stdout_handle
        .join()
        .map_err(|_| CmdError::Thread("stdout"))?;
    stderr_handle
        .join()
        .map_err(|_| CmdError::Thread("stderr"))?;

    status.map_err(CmdError::ChildWait)
}

fn cmd_get_program(cmd: &Command) -> String {
    shell_words::split(&format!("{:?}", cmd))
        .ok()
        .map(|args| args.into_iter().next())
        .flatten()
        .unwrap_or_else(|| "<unknown>".to_string())
}
