use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Display info about a FlatGeobuf file
pub struct Args {
    #[argh(positional)]
    /// the FlatGeobuf file to inspect
    pub file: String,

    #[argh(switch)]
    /// output flatgeobuf info to stdout
    pub stdout: bool,
}
