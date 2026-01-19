use std::process::ExitCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeftError {
    #[error("{0}")]
    Recoverable(String),

    #[error("{0}")]
    Fatal(String),
}

impl WeftError {
    pub fn recoverable(msg: impl Into<String>) -> Self {
        WeftError::Recoverable(msg.into())
    }

    pub fn fatal(msg: impl Into<String>) -> Self {
        WeftError::Fatal(msg.into())
    }
}

impl From<WeftError> for ExitCode {
    fn from(e: WeftError) -> Self {
        match e {
            WeftError::Recoverable(msg) => {
                println!("{}", msg);
                ExitCode::SUCCESS
            }
            WeftError::Fatal(msg) => {
                eprintln!("Error: {}", msg);
                eprintln!("Run with WEFT_LOG=debug for details");
                ExitCode::from(1)
            }
        }
    }
}
