use aliyun_oss_client::errors::OssError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError{
  #[error("oss-client error: {0}")]
  OssError(#[from] OssError),

  #[error("io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("var error: {0}")]
  VarError(#[from] std::env::VarError),

  #[error("input error: {0}")]
  Input(&'static str),
}

pub type CliResult<T> = Result<T,CliError>;