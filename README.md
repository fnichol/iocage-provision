# iocage-provision

Creates an iocage based FreeBSD jail.

|                  |                                                         |
| ---------------: | ------------------------------------------------------- |
|               CI | [![CI Status][badge-ci-overall]][ci]                    |
|   Latest Version | [![Latest version][badge-version]][crate]               |
|    Documentation | [![Documentation][badge-docs]][docs]                    |
|  Crate Downloads | [![Crate downloads][badge-crate-dl]][crate]             |
| GitHub Downloads | [![GitHub downloads][badge-github-dl]][github-releases] |
|          License | [![Crate license][badge-license]][github]               |

This program uses [iocage] to create a VNET networked [ZFS]-backed [FreeBSD]
jail. Suitable defaults are computed for the default gateway and base release to
reduce the number of arguments in the common case. An optional `--ssh` flag will
install and start an SSH service when the jail boots for remote management.
Finally, an optional `--user` option will create a user in the new jail by
copying values from the outside/host system.

[freebsd]: https://www.freebsd.org/
[iocage]: https://iocage.io/
[zfs]: https://zfsonfreebsd.github.io/ZoF/

**Table of Contents**

<!-- toc -->

- [Installation](#installation)
  - [Cargo Install](#cargo-install)
  - [From Source](#from-source)
- [Usage](#usage)
  - [Example 1 Provisioning a New Jail With a Name and Address](#example-1-provisioning-a-new-jail-with-a-name-and-address)
  - [Example 2 Provisioning a New Jail With a User and SSH Service](#example-2-provisioning-a-new-jail-with-a-user-and-ssh-service)
  - [Example 3 Using a Custom Default Gateway and Base Release](#example-3-using-a-custom-default-gateway-and-base-release)
- [CI Status](#ci-status)
  - [Build (main branch)](#build-main-branch)
  - [Test (main branch)](#test-main-branch)
  - [Check (main branch)](#check-main-branch)
- [Code of Conduct](#code-of-conduct)
- [Issues](#issues)
- [Contributing](#contributing)
- [Release History](#release-history)
- [Authors](#authors)
- [License](#license)

<!-- tocstop -->

## Installation

This program is intended to run on FreeBSD systems only, so while it may compile
on other systems, it likely won't do what you expect ;)

### Cargo Install

If [Rust](https://rustup.rs/) is installed, then installing with Cargo is
straight forward:

```console
$ cargo install iocage-provision
```

### From Source

To install from source, you can clone the Git repository, build with Cargo and
copy the binary into a destination directory. This will build the project from
the latest commit on the main branch, which may not correspond to the latest
stable release:

```console
$ git clone https://github.com/fnichol/iocage-provision.git
$ cd iocage-provision
$ cargo build --release
$ cp ./target/release/iocage-provision /dest/path/
```

## Usage

### Example 1 Provisioning a New Jail With a Name and Address

The following command will create a new jail called `ferris` with an IP
address/subnet mask of `192.168.0.100/24`.

```console
$ iocage-provision ferris 192.168.0.100/24
```

### Example 2 Provisioning a New Jail With a User and SSH Service

The following command will create a new jail with a running SSH service, and a
user called `jdoe` which is copied from the host system (note that the user must
exist on the host system).

```console
$ iocage-provision --user jdoe --ssh homebase 10.0.0.25/24
```

### Example 3 Using a Custom Default Gateway and Base Release

The following command will create a new jail by overriding the default gateway
and default base release values.

```console
$ iocage-provision --gateway 10.1.0.254 --release 11.1-RELEASE \
  bespoke 10.1.0.1/24
```

## CI Status

### Build (main branch)

| Operating System | Stable Rust                                                             |
| ---------------: | ----------------------------------------------------------------------- |
|          FreeBSD | [![FreeBSD Stable Build Status][badge-stable_freebsd-build]][ci-main] |

### Test (main branch)

| Operating System | Stable Rust                                                           |
| ---------------: | --------------------------------------------------------------------- |
|          FreeBSD | [![FreeBSD Stable Test Status][badge-stable_freebsd-test]][ci-main] |

### Check (main branch)

|        | Status                                            |
| ------ | ------------------------------------------------- |
| Lint   | [![Lint Status][badge-check-lint]][ci-main]     |
| Format | [![Format Status][badge-check-format]][ci-main] |

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
for inclusion in the work by you, as defined in the MPL-2.0 license, shall be
licensed as above, without any additional terms or conditions.

[badge-check-format]:
  https://img.shields.io/cirrus/github/fnichol/iocage-provision.svg?style=flat-square&task=check&script=format
[badge-check-lint]:
  https://img.shields.io/cirrus/github/fnichol/iocage-provision.svg?style=flat-square&task=check&script=lint
[badge-ci-overall]:
  https://img.shields.io/cirrus/github/fnichol/iocage-provision.svg?style=flat-square
[badge-crate-dl]:
  https://img.shields.io/crates/d/iocage-provision.svg?style=flat-square
[badge-docs]: https://docs.rs/iocage-provision/badge.svg?style=flat-square
[badge-github-dl]:
  https://img.shields.io/github/downloads/fnichol/iocage-provision/total.svg?style=flat-square
[badge-license]:
  https://img.shields.io/crates/l/iocage-provision.svg?style=flat-square
[badge-stable_freebsd-build]:
  https://img.shields.io/cirrus/github/fnichol/iocage-provision.svg?style=flat-square&task=test_stable_freebsd&script=build
[badge-stable_freebsd-test]:
  https://img.shields.io/cirrus/github/fnichol/iocage-provision.svg?style=flat-square&task=test_stable_freebsd&script=test
[badge-version]:
  https://img.shields.io/crates/v/iocage-provision.svg?style=flat-square
[changelog]:
  https://github.com/fnichol/iocage-provision/blob/main/CHANGELOG.md
[ci]: https://cirrus-ci.com/github/fnichol/iocage-provision
[ci-main]: https://cirrus-ci.com/github/fnichol/iocage-provision/main
[code-of-conduct]:
  https://github.com/fnichol/iocage-provision/blob/main/CODE_OF_CONDUCT.md
[crate]: https://crates.io/crates/iocage-provision
[docs]: https://docs.rs/iocage-provision
[fnichol]: https://github.com/fnichol
[github-releases]: https://github.com/fnichol/iocage-provision/releases
[github]: https://github.com/fnichol/iocage-provision
[issues]: https://github.com/fnichol/iocage-provision/issues
[license]: https://github.com/fnichol/iocage-provision/blob/main/LICENSE.txt
