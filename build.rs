// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use inner::Info;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

fn main() {
    let mut info = Info::new();

    generate_version_short(&mut info);
    generate_version_long(&mut info);

    println!("cargo:rerun-if-env-changed=NIGHTLY_BUILD");
    if let Ok(date) = std::env::var("NIGHTLY_BUILD") {
        println!(
            "cargo:rustc-env=CARGO_PKG_VERSION={}-nightly.{}",
            std::env::var("CARGO_PKG_VERSION")
                .unwrap()
                .split('-')
                .next()
                .unwrap(),
            date,
        );
    }
}

fn generate_version_short(info: &mut Info) {
    let dst =
        File::create(Path::new(&env::var("OUT_DIR").unwrap()).join("version_short.txt")).unwrap();
    let mut dst = BufWriter::new(dst);
    io::copy(&mut io::Cursor::new(version_short(info)), &mut dst).unwrap();
}

fn generate_version_long(info: &mut Info) {
    let dst =
        File::create(Path::new(&env::var("OUT_DIR").unwrap()).join("version_long.txt")).unwrap();
    let mut dst = BufWriter::new(dst);
    io::copy(&mut io::Cursor::new(version_long(info)), &mut dst).unwrap();
}

fn version_short(info: &mut Info) -> String {
    let mut version = env!("CARGO_PKG_VERSION").to_string();
    if let Some(hash) = info.commit_hash_short() {
        let mut extra = String::new();
        extra.push_str(" (");
        extra.push_str(hash);
        extra.push(' ');
        if let Some(date) = info.commit_date() {
            extra.push_str(date);
            extra.push(')');
            version.push_str(&extra)
        }
    }
    version
}

fn version_long(info: &mut Info) -> String {
    let mut version = version_short(info);
    version.push_str("\nbinary: ");
    version.push_str(env!("CARGO_PKG_NAME"));
    version.push_str("\nrelease: ");
    version.push_str(env!("CARGO_PKG_VERSION"));
    if let Some(hash) = info.commit_hash_long() {
        version.push_str("\ncommit-hash: ");
        version.push_str(hash);
    }
    if let Some(date) = info.commit_date() {
        version.push_str("\ncommit-date: ");
        version.push_str(date);
    }
    version
}

mod inner {
    use std::collections::HashMap;
    use std::env;
    use std::process::Command;

    pub struct Info(HashMap<&'static str, Option<String>>);

    impl Info {
        pub fn new() -> Self {
            Info(HashMap::new())
        }

        pub fn commit_hash_short(&mut self) -> Option<&str> {
            self.0
                .entry("commit_hash_short")
                .or_insert_with(commit_hash_short)
                .as_deref()
        }

        pub fn commit_hash_long(&mut self) -> Option<&str> {
            self.0
                .entry("commit_hash_long")
                .or_insert_with(commit_hash_long)
                .as_deref()
        }

        pub fn commit_date(&mut self) -> Option<&str> {
            self.0
                .entry("commit_date")
                .or_insert_with(commit_date)
                .as_deref()
        }
    }

    pub fn commit_hash_short() -> Option<String> {
        let hash = command_stdout(Command::new(git()).args(&["show", "-s", "--format=%h"]));

        match is_dirty() {
            Some(id) if id => hash.map(|hash| format!("{}-dirty", hash)),
            _ => hash,
        }
    }

    pub fn commit_hash_long() -> Option<String> {
        let hash = command_stdout(Command::new(git()).args(&["show", "-s", "--format=%H"]));

        match is_dirty() {
            Some(id) if id => hash.map(|hash| format!("{}-dirty", hash)),
            _ => hash,
        }
    }

    pub fn commit_date() -> Option<String> {
        command_stdout(Command::new(git()).args(&["show", "-s", "--format=%ad", "--date=short"]))
    }

    pub fn is_dirty() -> Option<bool> {
        Command::new(git())
            .args(&["diff-index", "--quiet", "HEAD"])
            .status()
            .ok()
            .map(|status| !status.success())
    }

    fn git() -> String {
        env::var("GIT_CMD").unwrap_or_else(|_| "git".to_string())
    }

    fn command_stdout(cmd: &mut Command) -> Option<String> {
        cmd.output()
            .ok()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }
}
