// Copyright 2025 Colton Loftus
// SPDX-License-Identifier: Apache-2.0

use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Print info about a FlatGeobuf file. Author: Colton Loftus
pub struct Args {
    #[argh(positional)]
    /// the FlatGeobuf file to inspect
    pub file: String,

    #[argh(switch)]
    /// output flatgeobuf info to stdout instead of the TUI
    pub stdout: bool,
}
