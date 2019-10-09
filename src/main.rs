// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use iocage_provision::Result;
use log::{debug, error};
use std::process;

mod cli;

fn main() {
    cli::util::setup_panic_hooks();

    if let Err(err) = try_main() {
        for line in cli::util::pretty_error(&err).lines() {
            error!("{}", line);
        }
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let args = cli::from_args();
    cli::util::init_logger(args.verbose);
    debug!("parsed cli arguments; args={:?}", args);

    iocage_provision::ensure_root()?;
    iocage_provision::provision_jail(
        &args.name,
        &args.ip,
        &args.gateway,
        &args.release,
        args.user.as_ref().map(String::as_str),
        args.ssh,
    )?;

    Ok(())
}
