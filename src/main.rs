extern crate lib_dot_installer;

use std::path::PathBuf;
use structopt::StructOpt;

use lib_dot_installer::{install, ConfigLink, DIResult};
use std::collections::HashMap;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    #[structopt(name = "", parse(from_os_str))]
    paths: Vec<PathBuf>,
}

fn main() -> DIResult<()> {
    Opt::from_args()
        .paths
        .into_iter()
        .for_each(|path: PathBuf| install(path.as_path()));

    Ok(())
}
