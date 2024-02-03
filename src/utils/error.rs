#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("found invalid opcode {0} (with operands {1})")]
    InvalidOpcode(u8, u8),

    #[error("{0}")]
    CoreCommon(smpl_core_common::utils::Error),

    #[error("{0}")]
    SASM(sasm_lib::utils::Error),

    #[error("{0}")]
    External(String),
}
pub type Result<T> = std::result::Result<T, Error>;

impl From<smpl_core_common::utils::Error> for Error {
		fn from(value: smpl_core_common::utils::Error) -> Self {
				Self::CoreCommon(value)
		}
}
impl From<sasm_lib::utils::Error> for Error {
		fn from(value: sasm_lib::utils::Error) -> Self {
				Self::SASM(value)
		}
}
