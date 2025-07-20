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

impl core::fmt::Display for Error{
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// FIXME: Print a nice output, don't just reuse the Debug impl
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

/// Retrieves the configuration values for the project located at a given path.
/// 
/// Configs are read from files named "gt-tool.toml" in 
/// - /etc
/// - each dir in $XDG_CONFIG_DIRS.
/// FIXME: Allow user-specified search paths.
pub fn get_config(project: &str) -> Result<PartialConfig> {
    /*
     1. Find all search dirs
        - anything in `$XDG_CONFIG_DIRS` and the `/etc` dir
            - will require splitting on ":"
        - Prefer user-specific configs, so take XDG_CONFIG_DIRS first.
        - I can't know which are "more specific" in the XDG_CONFIG_DIRS var, so
          I'll have to take it as-is.
     2. Iterate config dirs
     3. Try load toml::Value from file
     4. Try-get proj-specific table
     5. Try-get "[all]" table
     6. (merge) Update `Option::None`s in proj-spec with `Some(_)`s from "[all]"
     7. (merge, again) Fold the PartialConfigs into a finished one
     */


    // Read env var `XDG_CONFIG_DIRS` and split on ":" to get highest-priority list
    // TODO: Emit warning when paths aren't unicode
    let xdg_var = std::env::var("XDG_CONFIG_DIRS")
        .unwrap_or(String::from(""));
    let xdg_iter = xdg_var.split(":");

    // Set up the "/etc" list
    //      Which is pretty silly, in this case.
    //      Maybe a future version will scan nested folders and this will make
    //      more sense.
    let etc_iter = ["/etc"];

    // glue on the "/etc" path
    let path_iter = xdg_iter.chain(etc_iter);
    let file_iter = path_iter
        .map(|path_str| {
            let mut path = PathBuf::from(path_str);
            path.push("gt-tool.toml");
            path
        });
    let toml_iter = file_iter
        .map(std::fs::read_to_string)   // read text from file
        .filter_map(|res| res.ok()) // remove any error messages
        // TODO: Log warnings when files couldn't be read.
        .map(|toml_text| toml_text.parse::<Value>())    // try convert to `toml::Value`
        .filter_map(|res| res.ok()); // remove any failed parses
    let config_iter = toml_iter
        .map(|val| -> Result<PartialConfig> {
            // Like `fn read_conf_str(...)`, but doesn't produce a `WholeFile`

            // 1. Get the top-level table that is the config file
            let cfg_table = val.as_table().ok_or(Error::BadFormat)?;

            // 2. Get table
            let maybe_proj = get_table(cfg_table, project)
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
                // 5. Merge the PartialConfigs together, project-specific has higher priority
                .and_then(|pair| {
                    Ok(pair.0.merge(pair.1))
                });
            maybe_proj
        })
        .filter_map(|res| res.ok())
        .fold(PartialConfig::default(), |acc, inc|{
            acc.merge(inc)
        });
    return Ok(config_iter);
}

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct PartialConfig {
    project_path: Option<String>,
    gitea_url: Option<String>,
    owner: Option<String>,
    repo: Option<String>,
    token: Option<String>,
}

impl PartialConfig {
    // One lonely builder-pattern function to set the project path.
    // This is so I can do continuation style calls instead of a bunch of
    // successive `let conf = ...` temporaries.
    fn project_path(self, path: impl ToString) -> Self{
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
            gitea_url: get_maybe_property(&value, "gitea_url")?.cloned(),
            owner: get_maybe_property(&value, "owner")?.cloned(),
            repo: get_maybe_property(&value, "repo")?.cloned(),
            token: get_maybe_property(&value, "token")?.cloned(),
        })
    }

}

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
struct WholeFile {
    all: PartialConfig,
    project_overrides: Vec<PartialConfig>,
}

fn load_from_file(path: &str) -> Result<WholeFile> {
    let res = std::fs::read_to_string(path);
    match res {
        Ok(s) => read_conf_str(s.as_str()),
        Err(e) => {
            eprintln!("->> file io err: {:?}", e);
            Err(Error::CouldntReadFile)
        }
    }
}

fn read_conf_str(text: &str) -> Result<WholeFile> {
    let mut whole = WholeFile::default();

    let toml_val = text.parse::<Value>()?;

    // The config file is one big table. If the string we decoded is
    // some other toml::Value variant, it's not correct.
    // Try getting it as a table, return Err(BadFormat) otherwise.
    let cfg_table = toml_val.as_table().ok_or(Error::BadFormat)?;

    // Get the global config out of the file
    let table_all = get_table(cfg_table, "all")?;
    whole.all = PartialConfig::try_from(table_all)?;

    // Loop over the per-project configs, if any.
    let per_project_keys = cfg_table
        .keys()
        .filter(|s| { // Discard the "[all]" table
            *s != "all"
        });

    for path in per_project_keys {
        let tab = get_table(cfg_table, path)?;
        let part_cfg = PartialConfig::try_from(tab)?
            .project_path(path.clone());
        whole.project_overrides.push(part_cfg);
    }
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

    // Util for generating a reference struct
    fn gen_expected_struct() -> WholeFile {
        WholeFile {
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
        }
    }

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
        let fx_sample_config_string = include_str!("../test_data/sample_config.toml");
        let fx_expected_struct = gen_expected_struct();
        let conf = read_conf_str(fx_sample_config_string)?;
        
        assert_eq!(conf, fx_expected_struct);
        Ok(())
    }

    /* TODO: Improve semantics around reading an empty string
    An empty config string will result in Error::NoSuchTable when "[all]"
    is retrieved. But this will *also* happen when other configs are present,
    but "[all]" isn't. Do I treat these as valid configurations, using some
    hard-coded default as the fallback? Or do I reject configs that don't have
    an all-table?
     */
    #[test]
    fn read_config_string_empty() {
        let fx_sample_cfg = "";
        let conf = read_conf_str(fx_sample_cfg);
        assert!(conf.is_err());
    }

    #[test]
    // File exists and has valid configuration.
    fn load_from_file_ok() -> Result<()> {
        let conf = load_from_file("test_data/sample_config.toml")?;
        assert_eq!(conf, gen_expected_struct());
        Ok(())
    }

    #[test]
    // File does not exist.
    fn load_from_file_missing() -> Result<()> {
        let res = load_from_file("test_data/doesnt_exist.toml");
        let err = res.unwrap_err();
        assert_eq!(err, Error::CouldntReadFile);
        Ok(())
    }

    #[test]
    // File exists but has garbage inside.
    // TODO: This bumps against the same semantic issue as the todo note on
    // the 'read_config_string_empty' test
    fn load_from_file_bad() {
        let res = load_from_file("test_data/missing_all_table.toml");
        assert!(res.is_err());
    }

    #[test]
    // FIXME: Allow user-specified search paths, then update test to use them.
    // This test can only work if there's a config file the program can find.
    // Right now, that means a file called "gt-tool.toml" in
    // 1. `/etc`
    // 2. anything inside `$XDG_CONFIG_DIRS`
    fn check_get_config() -> Result<()>{
        let load_conf = get_config("/home/robert/projects/gt-tool")?;
        let expected = PartialConfig {
            project_path: Some(String::from("/home/robert/projects/gt-tool")),
            owner: Some(String::from("robert")),
            repo: Some(String::from("gt-tool")),
            gitea_url: Some(String::from("http://localhost:3000")),
            token: Some(String::from("fake-token")),
        };

        assert_eq!(load_conf, expected);
        Ok(())
    }
}
