use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum SniprunError {
    ///this error should only be raised when something goes very wrong, and you can't figure out
    ///what
    #[error("Unknown error: {0}")]
    UnknownError(String),
    ///internal error, should only be raised by Sniprun-specifi code; *not* in languages
    ///interpreters
    #[error("Internal error: {0}")]
    InternalError(String),
    /// raised if code cannot be fetched from files for whatever reason
    #[error("Cannot fetch code from files")]
    FetchCodeError,
    ///when the user's code run into problems because of an interpreter's implementation
    #[error("Interpreter limitations error: {0}")]
    InterpreterLimitationError(String),
    /// raised when code couldn't be run because of either incorrect code or
    /// UnsufficientSupportLevel but the language interpreter cannot determine which one
    #[error("Code contains errors or unsufficent support level")]
    InterpreterError,
    /// should be raised when users code fail to run but code is asserted correct
    #[error("Support level not high enough for this language")]
    UnsufficientSupportLevel,
    /// errors raised if the user code is incorrect and fail a compile-time (and not because the language interpreter
    /// failed to the needed code/imports
    #[error("Compile-time error: {0}")]
    CompilationError(String),
    /// errors raised if the user code is incorrect and fail a run-time (and not because the language interpreter failed to fetch the needed code/imports
    #[error("RuntimeError: {0}")]
    RuntimeError(String),
    ///custom error for advanced interpreters, the error will be displayed as-is
    #[error("{0}")]
    CustomError(String),
}
