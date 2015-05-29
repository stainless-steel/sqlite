use ResultCode;

/// An error.
#[derive(Debug)]
pub struct Error {
    pub code: ResultCode,
    pub message: Option<String>,
}
