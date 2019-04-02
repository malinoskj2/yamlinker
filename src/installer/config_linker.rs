use super::error::InstallerErr;
use super::fs_util;
use crate::FailErr;
use failure::Fail;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::hash::Hash;
use std::io::Error as IOError;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn install(repo_path: impl AsRef<Path>, cfg_name: &str) -> Result<(), FailErr> {
    let cfg_map_paths = fs_util::find_file_in_dir(repo_path.as_ref(), vec![cfg_name.to_string()])?;

    let res: Vec<()> = cfg_map_paths
        .into_iter()
        .flat_map(|path: PathBuf| parse_config_links(path))
        .map(|(key, cfg_link)| (key, expand_path(cfg_link, &repo_path)))
        .filter_map(|(k, value)| value.execute().ok())
        .collect();

    Ok(())
}

fn parse_config_links(cfg_map: impl AsRef<Path>) -> HashMap<String, ConfigLink> {
    fs::File::open(&cfg_map)
        .map(|file| {
            let res_map: HashMap<String, ConfigLink> =
                serde_yaml::from_reader(file).expect("failed to read cfg");
            res_map
        })
        .map(|map| {
            map.into_iter()
                .map(|(key, value)| (key, append_repo_dir(&cfg_map, value)))
                .collect()
        })
        .expect("failed to read cfg")
}

fn append_repo_dir(repo_dir: impl AsRef<Path>, cfg_link: ConfigLink) -> ConfigLink {
    ConfigLink {
        source: cfg_link.source,
        destination: cfg_link.destination,
        method: cfg_link.method,
    }
}

fn expand_path(cfg_link: ConfigLink, repo_path: impl AsRef<Path>) -> ConfigLink {
    let mut base_repo: PathBuf = repo_path.as_ref().to_owned();
    let mut base: PathBuf = cfg_link.source.clone();
    let sub_s: String = base.to_str().unwrap().to_owned();
    println!("sub-s:{:?}", &sub_s[1..]);
    base_repo.push(&sub_s[1..]);

    ConfigLink {
        source: base_repo,
        destination: PathBuf::from(
            shellexpand::tilde(cfg_link.destination.as_path().to_str().unwrap()).to_string(),
        ),
        method: cfg_link.method,
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum CLMethod {
    link,
    copy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigLink {
    source: PathBuf,
    destination: PathBuf,
    method: CLMethod,
}

impl ConfigLink {
    pub fn execute(self) -> Result<(), FailErr> {
        println!("link: {:?} -> {:?}", self.source, self.destination);

        symlink::symlink_file(self.source, self.destination)
            .map_err(|_| InstallerErr::SymLinkFail)
            .map(Ok)?
    }
}
