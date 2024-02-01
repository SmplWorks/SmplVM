use std::{path::PathBuf, str::FromStr};

use crate::utils::{Args, Result, Error};

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub in_path : PathBuf,

    #[serde(default = "_compile_default")]
    pub compile : bool,

    #[serde(default = "_memory_len_default")]
    pub memory_len : usize,

    #[serde(default = "_display_default")]
    pub display : bool,

    #[serde(default = "_debug_default")]
    pub debug : bool,

    #[serde(default = "_breakpoints_default")]
    pub breakpoints : Vec<u16>,

    #[serde(default = "_root_dir_default")]
    pub root_dir : PathBuf,
}

impl Config {
    pub fn load(args : &Args) -> Result<Self> {
        let fpath = PathBuf::from(args.config_path.clone());
        let mut cfg = Config::from_str(&std::fs::read_to_string(&fpath)
                                .map_err(|err| Error::External(err.to_string()))?)?;

        if cfg.root_dir == PathBuf::from("") {
            cfg.root_dir = fpath.parent().unwrap().to_owned();
        }

        let mut true_in_path = cfg.root_dir.clone();
				true_in_path.push(cfg.in_path);
        cfg.in_path = true_in_path;

        cfg.breakpoints.append(&mut args.breakpoints.clone());
        cfg.breakpoints.sort();

        Ok(cfg)
    }
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        toml::from_str(s).map_err(|err| Error::External(err.to_string()))
    }
}

fn _compile_default() -> bool {
    false
}

fn _root_dir_default() -> PathBuf {
    PathBuf::from_str("").unwrap()
}

fn _display_default() -> bool {
    true
}

fn _debug_default() -> bool {
    false
}

fn _memory_len_default() -> usize {
    32 * 1024 * 1024 // 32KiB
}

fn _breakpoints_default() -> Vec<u16> {
    vec![]
}
