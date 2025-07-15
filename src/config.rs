use toml::{Value, value::Table};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Error {
    BadFormat,
    NoSuchProperty,
    NoSuchTable,
    TomlWrap(toml::de::Error),
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::TomlWrap(value)
    }
}

impl core::fmt::Display for Error{
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// FIXME: Print a nice output, don't just reuse the Debug impl
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

/// The config properties are individual strings. This gets the named property,
/// or an error explaining why it couldn't be fetched.
fn get_property<'outer>(outer: &'outer Table, property: String) -> Result<&'outer String> {
    let maybe_prop = outer.get(&property).ok_or(Error::NoSuchProperty)?;
    if let Value::String(text) = maybe_prop {
        Ok(text)
    } else {
        Err(Error::BadFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_single_prop() -> Result<()> {
        let fx_input_str = "owner = \"dingus\"";
        let fx_value = fx_input_str.parse::<Value>()?;
        let fx_value = fx_value.as_table().ok_or(Error::NoSuchTable)?;
        let expected = "dingus";

        let res = get_property(&fx_value, String::from("owner"))?;
        assert_eq!(res, expected);
        Ok(())
    }

    // The property is given the value of empty-string `""`
    #[test]
    fn read_single_prop_empty_quotes() -> Result<()> {
        let fx_input_str = "owner = \"\"";
        let fx_value = fx_input_str.parse::<Value>()?;
        let fx_value = fx_value.as_table().ok_or(Error::NoSuchTable)?;
        let expected = "";

        let res = get_property(&fx_value, String::from("owner"))?;
        assert_eq!(res, expected);
        Ok(())
    }
}
