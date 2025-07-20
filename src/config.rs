use std::path::PathBuf;

use toml::{Value, value::Table};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Error {
    BadFormat,
    NoSuchProperty,
    NoSuchTable,
    CouldntReadFile,
    TomlWrap(toml::de::Error),
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::TomlWrap(value)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: Print a nice output, don't just reuse the Debug impl
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

/// Creates an iterator of default (Linux) search paths. The iterator output
/// is a list of files named "gt-tool.toml" found in decreasingly specific
/// configuration folders.
///
/// - any dirs listed in env var `$XDG_CONFIG_DIRS`
/// - and the `/etc` dir
///
/// This is so that user-specific configs are used first, then machine-wide
/// ones.
pub fn default_paths() -> impl Iterator<Item = PathBuf> {
    // Read env var `XDG_CONFIG_DIRS` and split on ":" to get highest-priority list
    // TODO: Emit warning when paths aren't unicode
    std::env::var("XDG_CONFIG_DIRS")
        .unwrap_or(String::from(""))
        .split(":")
        // Set up the "/etc" list
        //      Which is pretty silly, in this case.
        //      Maybe a future version will scan nested folders and this will make
        //      more sense.
        // glue on the "/etc" path
        .chain(["/etc"])
        .map(|path_str| {
            let mut path = PathBuf::from(path_str);
            path.push("gt-tool.toml");
            path
        })
        .collect::<Vec<_>>()
        .into_iter()
}

/// Searches through the files, `search_files`, for configuration related to a
/// project, `project`.
///
/// The project string is used as a map key and should match the real path of a
/// project on disk. These are the table names for each config section.
///
/// The search files iterator must produce *files* not *folders.* For now,
/// there is no mechanism to ensure correct usage. Files that can't be opened
/// will quietly be skipped, so there will be no warning when one gives a
/// folder.
///
/// Use `fn default_paths()` to get a reasonable (Linux) default.
///
/// TODO: Check for, and warn or error when given a dir.
pub fn get_config(
    project: &str,
    search_files: impl Iterator<Item = PathBuf>,
) -> Result<PartialConfig> {
    /*
    1. Get conf search (from fn input)
    2. Iterate config dirs
    3. Try load toml::Value from file
    4. Try-get proj-specific table
    5. Try-get "[all]" table
    6. (merge) Update `Option::None`s in proj-spec with `Some(_)`s from "[all]"
    7. (merge, again) Fold the PartialConfigs into a finished one
    */

    let file_iter = search_files;

    let toml_iter = file_iter
        .map(std::fs::read_to_string) // read text from file
        .filter_map(|res| res.ok()) // remove any error messages
        // TODO: Log warnings when files couldn't be read.
        .map(|toml_text| toml_text.parse::<Value>()) // try convert to `toml::Value`
        .filter_map(|res| res.ok()); // remove any failed parses
    let config_iter = toml_iter
        .map(|val| -> Result<PartialConfig> {
            // Like `fn read_conf_str(...)`, but doesn't produce a `WholeFile`

            // 1. Get the top-level table that is the config file
            let cfg_table = val.as_table().ok_or(Error::BadFormat)?;

            // 2. Get table

            get_table(cfg_table, project)
                // 3. convert to PartialConfig
                .and_then(PartialConfig::try_from)
                // 4. assemble a 2-tuple of PartialConfigs by...
                .and_then(|proj| {
                    Ok((
                        // 4-1. Passing in the project-specific PartialConfig
                        proj.project_path(project),
                        // 4-2. Getting and converting to PartialConfig, or returning any Err() if one appears.
                        get_table(cfg_table, "all").and_then(PartialConfig::try_from)?,
                    ))
                })
                .map(|pair| pair.0.merge(pair.1))
        })
        .filter_map(|res| res.ok())
        .fold(PartialConfig::default(), |acc, inc| acc.merge(inc));
    Ok(config_iter)
}

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct PartialConfig {
    project_path: Option<String>,
    pub gitea_url: Option<String>,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub token: Option<String>,
}

impl PartialConfig {
    // One lonely builder-pattern function to set the project path.
    // This is so I can do continuation style calls instead of a bunch of
    // successive `let conf = ...` temporaries.
    fn project_path(self, path: impl ToString) -> Self {
        PartialConfig {
            project_path: Some(path.to_string()),
            ..self
        }
    }

    // Merges two `PartialConfig`'s together, producing a new one.
    // Non-None values on the right-hand side are used to replace values
    // in the left-hand side, even if they are Some(_).
    fn merge(self, other: Self) -> Self {
        Self {
            project_path: other.project_path.or(self.project_path),
            gitea_url: other.gitea_url.or(self.gitea_url),
            owner: other.owner.or(self.owner),
            repo: other.repo.or(self.repo),
            token: other.token.or(self.token),
        }
    }
}

impl TryFrom<&Table> for PartialConfig {
    type Error = crate::config::Error;

    fn try_from(value: &Table) -> Result<Self> {
        Ok(Self {
            // can't get table name because that key is gone by this point.
            project_path: None,
            gitea_url: get_maybe_property(value, "gitea_url")?.cloned(),
            owner: get_maybe_property(value, "owner")?.cloned(),
            repo: get_maybe_property(value, "repo")?.cloned(),
            token: get_maybe_property(value, "token")?.cloned(),
        })
    }
}

/// The outer value must be a Table so we can get the sub-table from it.
fn get_table(outer: &Table, table_name: impl ToString) -> Result<&Table> {
    outer
        .get(&table_name.to_string())
        .ok_or(Error::NoSuchTable)?
        .as_table()
        .ok_or(Error::BadFormat)
}

/// Similar to `get_property()` but maps the "Error::NoSuchProperty" result to
/// Option::None. Some properties aren't specified, and that's okay... sometimes.
fn get_maybe_property(outer: &Table, property: impl ToString) -> Result<Option<&String>> {
    let maybe_prop = get_property(outer, property);
    match maybe_prop {
        Ok(value) => Ok(Some(value)),
        Err(e) => {
            if let Error::NoSuchProperty = e {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}

/// The config properties are individual strings. This gets the named property,
/// or an error explaining why it couldn't be fetched.
fn get_property(outer: &Table, property: impl ToString) -> Result<&String> {
    let maybe_prop = outer
        .get(&property.to_string())
        .ok_or(Error::NoSuchProperty)?;
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
        expected.insert(
            String::from("with_a_garbage"),
            Value::String(String::from("value")),
        );

        let res = get_table(&fx_value, String::from("tab"))?;
        assert_eq!(res, &expected);
        Ok(())
    }

    #[test]
    fn check_get_config() -> Result<()> {
        let search_paths = ["./test_data/sample_config.toml"]
            .into_iter()
            .map(PathBuf::from);
        let load_result = get_config("/home/robert/projects/gt-tool", search_paths)?;
        let expected = PartialConfig {
            project_path: Some(String::from("/home/robert/projects/gt-tool")),
            owner: Some(String::from("robert")),
            repo: Some(String::from("gt-tool")),
            gitea_url: Some(String::from("http://localhost:3000")),
            token: Some(String::from("fake-token")),
        };

        assert_eq!(load_result, expected);
        Ok(())
    }
}
