use std::str::FromStr;

use smpl_core_common::Register;
use smpl_parser::*;
use crate::utils::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    Step,
    Continue,

    GetAddr(u16),
    SetAddr(u16, u8),

    GetReg(Register),
    SetReg(Register, u16),
}

impl Cmd {
    pub fn prompt(last_cmd : Option<Self>) -> Result<Self> {
        inquire::CustomType::<Self>::new("")
            .with_parser(&|s| Self::parse(s, last_cmd.clone()))
            .prompt()
            .map_err(|err| Error::External(err.to_string()))
    }

    #[allow(clippy::result_unit_err)]
    pub fn parse(s : &str, last_cmd : Option<Self>) -> std::result::Result<Self, ()> {
        if s.trim().is_empty() {
            return last_cmd.ok_or(())
        }

        let mut scanner = Scanner::new(tokenize(s).into());
        scanner.scan(|toks| match toks {
            [Token::Ident(cmd)] => match &**cmd {
                "s" | "step" => ScannerAction::Request(Self::Step),
                "c" | "cont" | "continue" => ScannerAction::Return(Self::Continue),
                "g" | "get" | "set" => ScannerAction::Require,
                _ => ScannerAction::None,
            }

            [Token::Ident(cmd), Token::Number(addr)] => match &**cmd {
                "g" | "get" => ScannerAction::Return(Self::GetAddr(*addr as u16)),
                "s" | "set" => ScannerAction::Require,
                _ => ScannerAction::None,
            }
            [Token::Ident(cmd), Token::Ident(reg)] if Register::from_str(reg).is_ok() => match &**cmd {
                "g" | "get" => ScannerAction::Return(Self::GetReg(Register::from_str(reg).unwrap())),
                "s" | "set" => ScannerAction::Require,
                _ => ScannerAction::None,
            }

            [Token::Ident(cmd), Token::Number(addr), Token::Number(value)] => match &**cmd {
                "s" | "set" => ScannerAction::Return(Self::SetAddr(*addr as u16, *value as u8)),
                _ => ScannerAction::None,
            }
            [Token::Ident(cmd), Token::Ident(reg), Token::Number(value)] if Register::from_str(reg) .is_ok() => match &**cmd {
                "s" | "set" => ScannerAction::Return(Self::SetReg(Register::from_str(reg).unwrap(), *value as u16)),
                _ => ScannerAction::None,
            }

            _ => ScannerAction::None,
        }).map(|res| res.unwrap()).map_err(|_| ())
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

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! ok_cases {
        ($ident:ident, $cases:expr, $expect:expr) => {
            #[test]
            fn $ident() {
                for case in $cases {
                    assert_eq!(Cmd::parse(case, None), Ok($expect));
                }
            }
        };
    }

    ok_cases!(step, ["s", "step"], Cmd::Step);
    ok_cases!(r#continue, ["c", "cont", "continue"], Cmd::Continue);
    ok_cases!(getaddr, ["g 0x1234", "get 0x1234"], Cmd::GetAddr(0x1234));
    ok_cases!(setaddr, ["s 0x1234 0x56", "set 0x1234 0x56"], Cmd::SetAddr(0x1234, 0x56));
    ok_cases!(getreg, ["g r0", "get r0"], Cmd::GetReg(Register::r0()));
    ok_cases!(setreg, ["s r0 0x1234", "set r0 0x1234"], Cmd::SetReg(Register::r0(), 0x1234));

    #[test]
    fn prev() {
        assert_eq!(Cmd::parse("", Some(Cmd::Step)), Ok(Cmd::Step));
        assert_eq!(Cmd::parse("", Some(Cmd::Continue)), Ok(Cmd::Continue));
    }
}
