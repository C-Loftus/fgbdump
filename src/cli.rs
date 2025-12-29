use argh::FromArgs;


#[derive(FromArgs, Debug)]
/// Print info about a FlatGeobuf file
pub struct TopLevel {
    #[argh(subcommand)]
    pub cmd: Command,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum Command {
    Header(Header),
    Query(Query),
}

#[derive(FromArgs, Debug)]
/// Display info about the FlatGeobuf header
#[argh(subcommand, name = "header")]
pub struct Header {
    #[argh(option, description = "path or URL to the FlatGeobuf file")]
    pub file: String,

    #[argh(switch, description = "print to stdout instead of the TUI")]
    pub stdout: bool,
}

#[derive(FromArgs, Debug)]
/// Query by a bounding box
#[argh(subcommand, name = "query")]
pub struct Query {
    #[argh(option, description = "path or URL to the FlatGeobuf file")]
    pub file: String,

    #[argh(option, description = "bounding box as xmin,ymin,xmax,ymax")]
    pub bbox: String,
}