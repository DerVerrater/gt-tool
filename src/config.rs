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

fn lconf(text: &str) -> Result<WholeFile> {
    let mut whole = WholeFile::default();

    let toml_val = text.parse::<Value>()?;

    // The config file is one big table. If the string we decoded is
    // some other toml::Value variant, it's not correct.
    // Try getting it as a table, return Err(BadFormat) otherwise.
    let cfg_table = toml_val.as_table().ok_or(Error::BadFormat)?;

    // Get the global config out of the file
    if let Some(section_all) = cfg_table.get("all") {
    	if let Some(table_all) = section_all.as_table() {
    		whole.all.gitea_url = get_maybe_property(&table_all, "gitea_url")?.cloned();
    		whole.all.owner = get_maybe_property(&table_all, "owner")?.cloned();
    		whole.all.repo = get_maybe_property(&table_all, "repo")?.cloned();
    		whole.all.token = get_maybe_property(&table_all, "token")?.cloned();
    	}
    }

    // Loop over the per-project configs, if any.
    let per_project_keys = cfg_table
        .keys()
        .filter(|s| { // Discard the "[all]" table
            *s != "all"
        });

    for path in per_project_keys {
        let tab = get_table(cfg_table, path)?;
        let part_cfg = PartialConfig {
            project_path: Some(path.clone()),
            gitea_url: get_maybe_property(&tab, "gitea_url")?.cloned(),
            owner: get_maybe_property(&tab, "owner")?.cloned(),
            repo: get_maybe_property(&tab, "repo")?.cloned(),
            token: get_maybe_property(&tab, "token")?.cloned(),
        };
        whole.project_overrides.push(part_cfg);
    }
    println!(" ->> lconf - keys {:?}", cfg_table.keys().collect::<Vec<&String>>());
    Ok(whole)
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

    #[test]
    fn read_config_string_ok() -> Result<()> {
        let fx_sample_config_string = "
[all]
gitea_url = \"http://localhost:3000\"
token = \"fake-token\"

[\"/home/robert/projects/gt-tool\"]
owner = \"robert\"
repo = \"gt-tool\"

[\"/home/robert/projects/rcalc\"]
owner = \"jamis\"
repo = \"rcalc\"

[\"/home/robert/projects/rcalc-builders\"]
owner = \"jamis\"
repo = \"rcalc\"
";
        let fx_expected_struct = WholeFile {
            all: PartialConfig {
                project_path: None,
                gitea_url: Some(String::from("http://localhost:3000")),
                owner: None,
                repo: None,
                token: Some(String::from("fake-token"))
            },
            project_overrides: vec![
                PartialConfig {
                    project_path: Some(String::from("/home/robert/projects/gt-tool")),
                    gitea_url: None,
                    owner: Some(String::from("robert")),
                    repo: Some(String::from("gt-tool")),
                    token: None,
                },
                PartialConfig {
                    project_path: Some(String::from("/home/robert/projects/rcalc")),
                    gitea_url: None,
                    owner: Some(String::from("jamis")),
                    repo: Some(String::from("rcalc")),
                    token: None,
                },
                PartialConfig {
                    project_path: Some(String::from("/home/robert/projects/rcalc-builders")),
                    gitea_url: None,
                    owner: Some(String::from("jamis")),
                    repo: Some(String::from("rcalc")),
                    token: None,
                },
            
            ],
        };
        let conf = lconf(fx_sample_config_string)?;
        println!(" ->> Test conf: {:?}", conf);
        println!(" ->> Ref  conf: {:?}", fx_expected_struct);
        
        assert_eq!(conf, fx_expected_struct);
        Ok(())
    }
}
