# gt-tool

CLI tools for interacting with the Gitea API. Use interactively to talk to your Gitea instance, or automatically via a CI/CD pipeline.

## Usage

```txt
Usage: gt-tool [OPTIONS] <COMMAND>

Commands:
  list-releases   
  create-release  
  upload-release  
  help            Print this message or the help of the given subcommand(s)

Options:
  -u, --url <GITEA_URL>    [env: GTTOOL_GITEA_URL=]
  -o, --owner <OWNER>      [env: GTTOOL_OWNER=]
  -r, --repo <REPO>        [env: GTTOOL_REPO=]
  -p, --project <PROJECT>  Path to project (relative or absolute). Used to override configuration selection.
  -h, --help               Print help
  -V, --version            Print version
```

### Required Information

To function, this program requires knowledge of these items:

- Gitea URL
- Owner of repository
- Repository name

This info will be gathered from these locations, in order of priority:

1. CLI argument
2. Environment variable
3. Configuration files

It's worth noting that the "owner" is the entity that controls the repo on the Gitea instance. This will be the first part of the route in the URL: `http://demo.gitea.com/{owner}`.

Likewise, the "repo" is what ever the Gitea instance thinks it's called -- which doesn't have to match anyone's local copy! It will be the second part of the route in the URL: `http://demo.gitea.com/{owner}/{repo}`.

### Authentication

Authentication is token-based. There is no CLI option to prevent the token from appearing in any command logs.

In order of priority, the token is loaded from:

1. The environment variable `RELEASE_KEY_GITEA`
2. Config files, key `token`

Whether or not it is required depends on how your Gitea instance and the repositories inside are configured.  For example, a default Gitea configuration will allow unauthenticated users to see public repositories but not make any changes. This means no token is required to run `gt-tool list-releases`, while `gt-tool upload-release` *will* require a token.

For details, see Gitea's documentation on [token scopes](https://docs.gitea.com/development/oauth2-provider#scopes).

### The `--project` option

Settings retrieved from config files are selected based on the project's path. By default, the current directory will be used. In case that guess is incorrect, this option can be specified with another path.

See [configuration](#configuration) for details on format and file locations.

### Commands:

| Command | Description |
|-|-|
| list-releases | Prints all releases for the given repo. |
| create-release | Creates a new release. It is recommended to use the web page, but this will work in case you need it. |
| upload-release | Uploads one-or-more files to an existing release, identified by it's tag name. |
| help | prints the help text (the usage summary above). |

## Configuration

Instead of specifying everything on the command line every single run, some TOML files can be used to persist project settings.

> Exporting some environment variables would be similar, but that would be *more* annoying when working on multiple projects. One would have to constantly re-export the settings or use two shells. But then there's the issue of losing track of which shell has which settings.

Settings are retrieved from a table named the same as the project's path on disk. For example, gt_tool itself could have an entry as follows:

```toml
["/home/robert/projects/gt-tool"]
gitea_url = "https://demo.gitea.com/"
owner = "dummy"
repo = "gt-tool"
token = "fake-token"
```

Some may apply to all projects. For this, one can use the special `[all]` table.

```toml
[all]
gitea_url = "https://demo.gitea.com/"
```

Since the more-specific settings are preferred, these can be combined to have an override effect.

```toml
[all]
gitea_url = "https://demo.gitea.com/"
owner = "robert"
# no `repo = ` section because that must be project specific.
token = "fake-token"

# Override Gitea target so I can test my uploads privately.
["/home/robert/projects/gt-tool"]
gitea_url = "http://localhost:3000"
repo = "gt-tool"
```

Similar to how unspecified project settings will fall back to those in the "`[all]`" table, whole files will fall back to other, lower priority files. First, each dir in `$XDG_CONFIG_DIRS` is scanned for a `gt-tool.toml` file. Then, `/etc/gt-tool.toml`.

> All config files **MUST** be named named `gt-tool.toml`.

### Recognized Keys

| Key | Description |
|-|-|
| gitea_url |  URL of the Gitea server. Same as `-u`, `--url`, and `$GTTOOL_GITEA_URL`. |
| owner | Owner of the repository (individual, or organization). Combined with "repo" key to produce the fully-qualified-repo-name. Front-half of `-r`, `--repo`, and `$GTTOOL_FQRN` |
| repo | Name of the repository on the Gitea server. Combined with "owner" key to produce the fully-qualified-repo-name. Back-half of `-r`, `--repo`, and `$GTTOOL_FQRN` |
| token | Gitea auth token, exactly the same as `$RELEASE_KEY_GITEA` |

Additional keys are quietly ignored. The config loading is done by querying a HashMap, so anything not touched doesn't get inspected. The only requirements are that the file is valid TOML, and that these keys are all strings.h
