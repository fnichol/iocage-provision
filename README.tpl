<h1 align="center">
  <br/>
  {{crate}}
  <br/>
</h1>

<h4 align="center">
  Creates an iocage based FreeBSD jail.
</h4>

|                  |                                                                                          |
| ---------------: | ---------------------------------------------------------------------------------------- |
|               CI | [![CI Status][badge-ci-overall]][ci]<br /> [![Bors enabled][badge-bors]][bors-dashboard] |
|   Latest Version | [![Latest version][badge-version]][crate]                                                |
|    Documentation | [![Documentation][badge-docs]][docs]                                                     |
|  Crate Downloads | [![Crate downloads][badge-crate-dl]][crate]                                              |
| GitHub Downloads | [![Github downloads][badge-github-dl]][github-releases]                                  |
|          License | [![Crate license][badge-license]][github]                                                |

This program uses [iocage] to create a VNET networked [ZFS]-backed [FreeBSD]
jail. Suitable defaults are computed for the default gateway and base release to
reduce the number of arguments in the common case. An optional `--ssh` flag will
install and start an SSH service when the jail boots for remote management.
Finally, an optional `--user` option will create a user in the new jail by
copying values from the outside/host system.

[freebsd]: https://www.freebsd.org/
[iocage]: https://iocage.io/
[zfs]: https://zfsonfreebsd.github.io/ZoF/

<details>
<summary><strong>Table of Contents</strong></summary>

<!-- toc -->

</details>

## CLI

### Usage

#### Example 1 Provisioning a New Jail With a Name and Address

The following command will create a new jail called `ferris` with an IP
address/subnet mask of `192.168.0.100/24`.

```console
$ iocage-provision ferris 192.168.0.100/24
```

#### Example 2 Provisioning a New Jail With a User and SSH Service

The following command will create a new jail with a running SSH service, and a
user called `jdoe` which is copied from the host system (note that the user must
exist on the host system).

```console
$ iocage-provision --user jdoe --ssh homebase 10.0.0.25/24
```

#### Example 3 Using a Custom Default Gateway and Base Release

The following command will create a new jail by overriding the default gateway
and default base release values.

```console
$ iocage-provision --gateway 10.1.0.254 --release 11.1-RELEASE \
  bespoke 10.1.0.1/24
```

### Installation

#### install.sh (Pre-Built Binaries)

An installer is provided at
<https://fnichol.github.io/iocage-provision/install.sh> which installs a
suitable pre-built binary for FreeBSD. It can be downloaded and run locally or
piped into a shell interpreter in the "curl-bash" style as shown below. Note
that if you're opposed to this idea, feel free to check some of the alternatives
below.

To install the latest release for your system into `$HOME/bin`:

```console
> curl -sSf https://fnichol.github.io/iocage-provision/install.sh | sh
```

When the installer is run as `root` the installation directory defaults to
`/usr/local/bin`:

```console
> curl -sSf https://fnichol.github.io/iocage-provision/install.sh | sudo sh
```

A [nightly] release built from `HEAD` of the main branch is available which can
also be installed:

```console
> curl -sSf https://fnichol.github.io/iocage-provision/install.sh \
    | sh -s -- --release=nightly
```

For a full set of options, check out the help usage with:

```console
> curl -sSf https://fnichol.github.io/iocage-provision/install.sh \
    | sh -s -- --help
```

#### GitHub Releasees (Pre-Built Binaries)

Each release comes with binary artifacts published in [GitHub
Releases][github-releases]. The `install.sh` program downloads its artifacts
from this location so this serves as a manual alternative. Each artifact ships
with MD5 and SHA256 checksums to help verify the artifact on a target system.

#### Cargo Install

If [Rust](https://rustup.rs/) is installed on your system, then installing with
Cargo is straight forward with:

```console
> cargo install iocage-provision
```

#### From Source

To install from source, you can clone the Git repository, build with Cargo and
copy the binary into a destination directory. This will build the project from
the latest commit on the main branch, which may not correspond to the latest
stable release:

```console
> git clone https://github.com/fnichol/iocage-provision.git
> cd iocage-provision
> cargo build --release
> cp ./target/release/iocage-provision /dest/path/
```

---

## Library

{{readme}}

## CI Status

### Build (main branch)

| Operating System | Target                   | Stable Rust                                                                  |
| ---------------: | ------------------------ | ---------------------------------------------------------------------------- |
|          FreeBSD | `x86_64-unknown-freebsd` | [![FreeBSD Build Status][badge-ci-build-x86_64-unknown-freebsd]][ci-staging] |

### Test (main branch)

| Operating System | Stable Rust                                                               | Nightly Rust                                                                |
| ---------------: | ------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
|          FreeBSD | [![FreeBSD Stable Test Status][badge-ci-test-stable-freebsd]][ci-staging] | [![FreeBSD Nightly Test Status][badge-ci-test-nightly-freebsd]][ci-staging] |

**Note**: The
[Minimum Supported Rust Version (MSRV)](https://github.com/rust-lang/rfcs/pull/2495)
is also tested and can be viewed in the [CI dashboard][ci-staging].

### Check (main branch)

|        | Status                                                |
| ------ | ----------------------------------------------------- |
| Lint   | [![Lint Status][badge-ci-check-lint]][ci-staging]     |
| Format | [![Format Status][badge-ci-check-format]][ci-staging] |

## Code of Conduct

This project adheres to the Contributor Covenant [code of
conduct][code-of-conduct]. By participating, you are expected to uphold this
code. Please report unacceptable behavior to fnichol@nichol.ca.

## Issues

If you have any problems with or questions about this project, please contact us
through a [GitHub issue][issues].

## Contributing

You are invited to contribute to new features, fixes, or updates, large or
small; we are always thrilled to receive pull requests, and do our best to
process them as fast as we can.

Before you start to code, we recommend discussing your plans through a [GitHub
issue][issues], especially for more ambitious contributions. This gives other
contributors a chance to point you in the right direction, give you feedback on
your design, and help you find out if someone else is working on the same thing.

## Release History

See the [changelog] for a full release history.

## Authors

Created and maintained by [Fletcher Nichol][fnichol] (<fnichol@nichol.ca>).

## License

Licensed under the Mozilla Public License Version 2.0 ([LICENSE.txt][license]).

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the MIT license, shall be
licensed as above, without any additional terms or conditions.

[badge-bors]: https://bors.tech/images/badge_small.svg
[badge-ci-build-x86_64-unknown-freebsd]:
  https://img.shields.io/cirrus/github/fnichol/iocage-provision/staging?style=flat-square&task=build-bin-iocage-provision-x86_64-unknown-freebsd.tar.gz
[badge-ci-check-format]:
  https://img.shields.io/cirrus/github/fnichol/iocage-provision/staging?style=flat-square&task=check&script=format
[badge-ci-check-lint]:
  https://img.shields.io/cirrus/github/fnichol/iocage-provision/staging?style=flat-square&task=check&script=lint
[badge-ci-overall]:
  https://img.shields.io/cirrus/github/fnichol/iocage-provision/main?style=flat-square
[badge-ci-test-nightly-freebsd]:
  https://img.shields.io/cirrus/github/fnichol/iocage-provision/staging?style=flat-square&task=test-nightly-x86_64-unknown-freebsd
[badge-ci-test-stable-freebsd]:
  https://img.shields.io/cirrus/github/fnichol/iocage-provision/staging?style=flat-square&task=test-stable-x86_64-unknown-freebsd
[badge-crate-dl]:
  https://img.shields.io/crates/d/iocage-provision.svg?style=flat-square
[badge-docs]: https://docs.rs/iocage-provision/badge.svg?style=flat-square
[badge-github-dl]:
  https://img.shields.io/github/downloads/fnichol/iocage-provision/total.svg
[badge-license]:
  https://img.shields.io/crates/l/iocage-provision.svg?style=flat-square
[badge-version]:
  https://img.shields.io/crates/v/iocage-provision.svg?style=flat-square
[bors-dashboard]: https://app.bors.tech/repositories/32089
[changelog]: https://github.com/fnichol/iocage-provision/blob/main/CHANGELOG.md
[ci]: https://cirrus-ci.com/github/fnichol/iocage-provision
[ci-staging]: https://cirrus-ci.com/github/fnichol/iocage-provision/staging
[code-of-conduct]:
  https://github.com/fnichol/iocage-provision/blob/main/CODE_OF_CONDUCT.md
[crate]: https://crates.io/crates/iocage-provision
[docs]: https://docs.rs/iocage-provision
[fnichol]: https://github.com/fnichol
[github]: https://github.com/fnichol/iocage-provision
[github-releases]: https://github.com/fnichol/iocage-provision/releases
[issues]: https://github.com/fnichol/iocage-provision/issues
[license]: https://github.com/fnichol/iocage-provision/blob/main/LICENSE.txt
[nightly]: https://github.com/fnichol/iocage-provision/releases/tag/nightly
