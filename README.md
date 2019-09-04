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

**Table of Contents**

<!-- toc -->

- [CI Status](#ci-status)
  - [Build (master branch)](#build-master-branch)
  - [Test (master branch)](#test-master-branch)
  - [Check (master branch)](#check-master-branch)
- [Code of Conduct](#code-of-conduct)
- [License](#license)

<!-- tocstop -->

## CI Status

### Build (master branch)

| Operating System | Stable Rust                                                             |
| ---------------: | ----------------------------------------------------------------------- |
|          FreeBSD | [![FreeBSD Stable Build Status][badge-stable_freebsd-build]][ci-master] |

### Test (master branch)

| Operating System | Stable Rust                                                           |
| ---------------: | --------------------------------------------------------------------- |
|          FreeBSD | [![FreeBSD Stable Test Status][badge-stable_freebsd-test]][ci-master] |

### Check (master branch)

|        | Status                                            |
| ------ | ------------------------------------------------- |
| Lint   | [![Lint Status][badge-check-lint]][ci-master]     |
| Format | [![Format Status][badge-check-format]][ci-master] |

## Code of Conduct

This project adheres to the Contributor Covenant [code of
conduct][code-of-conduct]. By participating, you are expected to uphold this
code. Please report unacceptable behavior to fnichol@nichol.ca.

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
[ci]: https://cirrus-ci.com/github/fnichol/iocage-provision
[ci-master]: https://cirrus-ci.com/github/fnichol/iocage-provision/master
[code-of-conduct]: https://www.rust-lang.org/policies/code-of-conduct
[crate]: https://crates.io/crates/iocage-provision
[docs]: https://docs.rs/iocage-provision
[github-releases]: https://github.com/fnichol/iocage-provision/releases
[github]: https://github.com/fnichol/iocage-provision
[license]: https://github.com/fnichol/iocage-provision/blob/master/LICENSE.txt
