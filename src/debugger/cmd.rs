use std::str::FromStr;

use smpl_parser::*;
use crate::utils::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    Step,
    Continue,
}

impl Cmd {
    pub fn prompt(last_cmd : Option<Self>) -> Result<Self> {
        inquire::CustomType::<Self>::new("")
            .with_parser(&|s| Self::parse(s, last_cmd.clone()))
            .prompt()
            .map_err(|err| Error::External(err.to_string()))
    }

    pub fn parse(s : &str, last_cmd : Option<Self>) -> std::result::Result<Self, ()> {
        let mut scanner = Scanner::new(tokenize(s).into());
        scanner.scan(|toks| match toks {
            [Token::Ident(cmd)] => match &**cmd {
                "s" | "step" => ScannerAction::Return(Self::Step),
                "c" | "cont" | "continue" => ScannerAction::Return(Self::Continue),
                _ => ScannerAction::None,
            }

            _ => ScannerAction::None,
        }).map(|res| res.or(last_cmd).unwrap()).map_err(|_| ())
    }
}

impl std::fmt::Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl FromStr for Cmd {
    type Err = ();

    fn from_str(_: &str) -> std::result::Result<Self, Self::Err> {
        unreachable!()
    }
}
