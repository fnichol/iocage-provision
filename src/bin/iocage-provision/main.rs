// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::Result;
use log::debug;

mod cli;

fn main() -> Result<()> {
    cli::util::setup_panic_hooks();

    let args = cli::parse();
    cli::util::init_logger_with_verbosity(args.verbose);
    debug!("parsed cli arguments; args={:?}", args);

    iocage_provision::ensure_root()?;
    iocage_provision::provision_jail(
        &args.name,
        &args.ip,
        &args.gateway,
        &args.release,
        args.thick_jail,
        args.user.as_deref(),
        args.ssh,
    )?;

    Ok(())
}
