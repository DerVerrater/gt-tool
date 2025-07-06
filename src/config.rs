
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {}

impl core::fmt::Display for Error{
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// FIXME: Print a nice output, don't just reuse the Debug impl
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
	use super::*;
}