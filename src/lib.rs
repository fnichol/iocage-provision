use failure::{Fail, ResultExt};
use ipnet::IpNet;
use log::{debug, info, warn};
use std::error;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::IpAddr;
use std::path::Path;
use std::process::{ChildStdin, Command, ExitStatus, Stdio};
use std::result;
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

pub type Result<T> = result::Result<T, failure::Error>;

#[derive(Debug)]
pub enum Error {
    CmdFail(i32),
    CmdIO,
    CmdThread,
    NoGid(u32),
    NoUser(String),
    NotRoot,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match self {
            CmdFail(ref code) => write!(f, "command exited with code: {}", code),
            CmdIO => f.write_str("command i/o error"),
            CmdThread => f.write_str("command i/o thread error"),
            NoGid(ref group) => write!(f, "system group id not found: {}", group),
            NoUser(ref user) => write!(f, "system user not found: {}", user),
            NotRoot => f.write_str("root privileges required"),
        }
    }
}

impl error::Error for Error {}

pub fn ensure_root() -> Result<()> {
    if users::get_effective_uid() != 0 {
        Err(Error::NotRoot.into())
    } else {
        Ok(())
    }
}

pub fn provision_jail(
    name: &str,
    ip: &IpNet,
    gateway: &IpAddr,
    release: &str,
    user: Option<&str>,
    ssh_service: bool,
) -> Result<()> {
    let user = find_user(user)?;
    let json =
        create_pkglist_json(user.as_ref()).context("could not generate json pkglist tempfile")?;

    section!("Provisioning a jail named '{}'", name);

    info!("Creating '{}' via iocage", name);
    run_iocage_create(name, ip, gateway, release, json.path()).context("failed to create jail")?;

    if let Some(user) = user {
        let group = find_group(user.primary_group_id())?;

        info!("Preparing sudo config");
        exec_sudo_config(name).context("failed to prepare sudo config")?;

        info!("Creating group '{}'", group.name().to_string_lossy());
        exec_create_group(name, &group).context("failed to create group")?;

        info!("Creating user '{}'", user.name().to_string_lossy());
        exec_create_user(name, &user, &group).context("failed to create user")?;
    }

    if ssh_service {
        info!("Enabling SSH service");
        exec_ssh_service(name).context("failed to enable SSH service")?;
    }

    section!("Instance '{}' provisioned successfully", name);

    Ok(())
}

fn find_user(user_str: Option<&str>) -> Result<Option<User>> {
    match user_str {
        Some(user_str) => match users::get_user_by_name(user_str) {
            Some(user_info) => Ok(Some(user_info)),
            None => Err(Error::NoUser(user_str.to_string()).into()),
        },
        None => Ok(None),
    }
}

fn find_group(gid: u32) -> Result<Group> {
    users::get_group_by_gid(gid).ok_or_else(|| Error::NoGid(gid).into())
}

fn create_pkglist_json(user: Option<&User>) -> Result<NamedTempFile> {
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

fn exec_sudo_config(name: &str) -> Result<()> {
    iocage_exec(
        name,
        "echo '%wheel ALL=(ALL) NOPASSWD: ALL' >/usr/local/etc/sudoers.d/wheel",
    )
}

fn exec_create_group(name: &str, group: &Group) -> Result<()> {
    iocage_exec(
        name,
        format!(
            "pw groupadd -n '{grp}' -g '{gid}'",
            gid = group.gid(),
            grp = group.name().to_string_lossy(),
        ),
    )
}

fn exec_create_user(name: &str, user: &User, group: &Group) -> Result<()> {
    iocage_exec(
        name,
        format!(
            "pw useradd -n '{usr}' -u '{uid}' -g '{grp}' -G wheel -m -s '{shl}'",
            grp = group.name().to_string_lossy(),
            shl = user.shell().display(),
            uid = user.uid(),
            usr = user.name().to_string_lossy(),
        ),
    )
}

fn exec_ssh_service(name: &str) -> Result<()> {
    iocage_exec(
        name,
        r#"sysrc -f /etc/rc.conf sshd_enable="YES" && service sshd start"#,
    )
}

fn run_iocage_create(
    name: &str,
    ip: &IpNet,
    gateway: &IpAddr,
    release: &str,
    pkglist: &Path,
) -> Result<()> {
    let mut cmd = Command::new("iocage");
    cmd.arg("create")
        .arg("--name")
        .arg(name)
        .arg("--release")
        .arg(release)
        .arg("--pkglist")
        .arg(pkglist)
        .arg("--force")
        .arg("vnet=on")
        .arg(format!("ip4_addr=vnet0|{}", ip))
        .arg(format!("defaultrouter={}", gateway))
        .arg("resolver=none")
        .arg("boot=on")
        .env("PYTHONUNBUFFERED", "true");

    let status = spawn_and_indent(cmd).context("failed to run: iocage create")?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::CmdFail(status.code().unwrap_or(-1))
            .context("iocage create command failed")
            .into())
    }
}

fn iocage_exec<S: AsRef<str>>(name: &str, src: S) -> Result<()> {
    let mut cmd = Command::new("iocage");
    cmd.arg("exec")
        .arg(name)
        .arg("sh")
        .env("PYTHONUNBUFFERED", "true");

    let status = spawn_and_indent_with_stdin(cmd, |mut stdin| {
        stdin
            .write_all(b"set -eu\n\n")
            .context("failed to write to stdin")?;
        stdin
            .write_all(src.as_ref().as_bytes())
            .context("failed to write to stdin")?;
        Ok(())
    })
    .context("failed to run: iocage exec")?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::CmdFail(status.code().unwrap_or(-1))
            .context("iocage exec command failed")
            .into())
    }
}

fn spawn_and_indent(cmd: Command) -> Result<ExitStatus> {
    spawn_and_indent_with_stdin(cmd, |_| Ok(()))
}

fn spawn_and_indent_with_stdin<F>(mut cmd: Command, stdin_func: F) -> Result<ExitStatus>
where
    F: FnOnce(ChildStdin) -> Result<()>,
{
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    debug!("running; cmd={:?}", &cmd);
    let mut child = cmd.spawn().context("command failed to spawn")?;
    {
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::CmdIO.context("stdin was not captured"))?;
        stdin_func(stdin)?;
    }

    let stdout = BufReader::new(
        child
            .stdout
            .take()
            .ok_or_else(|| Error::CmdIO.context("stdout was not captured"))?,
    );
    let stdout_handle = thread::spawn(move || {
        for line in stdout.lines() {
            // This error happens in a thread, so we will panic here on error
            let line = line.expect("failed to read line from stdout");

            if log::max_level() == log::LevelFilter::Info {
                println!("        {}", line);
            } else {
                info!("{}", line);
            }
        }
    });

    let stderr = BufReader::new(
        child
            .stderr
            .take()
            .ok_or_else(|| Error::CmdIO.context("stderr was not captured"))?,
    );
    let stderr_handle = thread::spawn(move || {
        for line in stderr.lines() {
            // This error happens in a thread, so we will panic here on error
            let line = line.expect("failed to read line from stderr");

            if log::max_level() == log::LevelFilter::Info {
                eprintln!("        {}", line);
            } else {
                warn!("{}", line);
            }
        }
    });

    let status = child.wait();

    stdout_handle
        .join()
        .map_err(|_| Error::CmdThread.context("stdout thread panicked"))?;
    stderr_handle
        .join()
        .map_err(|_| Error::CmdThread.context("stderr thread panicked"))?;

    Ok(status.context("command did not run")?)
}
