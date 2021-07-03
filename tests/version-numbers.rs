// Copyright 2020 Fletcher Nichol and/or applicable contributors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license (see <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be copied, modified, or
// distributed except according to those terms.

#[test]
fn test_readme_deps() {
    // If the current version is a `*-dev` string, then ignore the check in README. We'd like to
    // keep the *last* released version string in the README as instructions in the main source
    // code branch.
    if !env!("CARGO_PKG_VERSION").ends_with("-dev") {
        version_sync::assert_markdown_deps_updated!("README.md");
    }
}

#[test]
fn test_html_root_url() {
    version_sync::assert_html_root_url_updated!("src/lib.rs");
}
