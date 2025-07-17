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

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
struct PartialConfig {
    project_path: Option<String>,
    gitea_url: Option<String>,
    owner: Option<String>,
    repo: Option<String>,
    token: Option<String>,
}

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
struct WholeFile {
    all: PartialConfig,
    project_overrides: Vec<PartialConfig>,
}


/// The outer value must be a Table so we can get the sub-table from it.
fn get_table<'outer>(outer: &'outer Table, table_name: impl ToString) -> Result<&'outer Table> {
    Ok(outer
        .get(&table_name.to_string())
        .ok_or(Error::NoSuchTable)?
        .as_table()
        .ok_or(Error::BadFormat)?)
}


/// Similar to `get_property()` but maps the "Error::NoSuchProperty" result to
/// Option::None. Some properties aren't specified, and that's okay... sometimes.
fn get_maybe_property<'outer> (outer: &'outer Table, property: impl ToString) -> Result<Option<&'outer String>> {
    let maybe_prop = get_property(outer, property);
    match maybe_prop {
        Ok(value) => Ok(Some(value)),
        Err(e) => {
            if let Error::NoSuchProperty = e {
                return Ok(None);
            } else {
                return Err(e);
            }
        }
    }
}

/// The config properties are individual strings. This gets the named property,
/// or an error explaining why it couldn't be fetched.
fn get_property<'outer>(outer: &'outer Table, property: impl ToString) -> Result<&'outer String> {
    let maybe_prop = outer.get(&property.to_string()).ok_or(Error::NoSuchProperty)?;
    if let Value::String(text) = maybe_prop {
        Ok(text)
    } else {
        Err(Error::BadFormat)
    }
}

#[cfg(test)]
mod tests {
    use toml::map::Map;

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

    #[test]
    fn read_table() -> Result<()> {
        let fx_input_str = "[tab]\nwith_a_garbage = \"value\"";
        let fx_value = fx_input_str.parse::<Value>()?;
        let fx_value = fx_value.as_table().ok_or(Error::BadFormat)?;
        let mut expected: Map<String, Value> = Map::new();
        expected.insert(String::from("with_a_garbage"), Value::String(String::from("value")));

        let res = get_table(&fx_value, String::from("tab"))?;
        assert_eq!(res, &expected);
        Ok(())
    }
}
