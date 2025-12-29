use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Print info about a FlatGeobuf file
pub struct Args {
    #[argh(positional)]
    /// the FlatGeobuf file to inspect
    pub first: String,

    #[argh(switch)]
    /// print to stdout
    pub stdout: bool,
}
