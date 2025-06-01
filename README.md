# gt-tools

CLI tools for interacting with the Gitea API. Use interactively to talk to your Gitea instance, or automatically via a CI/CD pipeline.

## Usage

```txt
Usage: gt-tools --url <GITEA_URL> --repo <REPO> <COMMAND>

Commands:
  list-releases   
  create-release  
  upload-release  
  help            Print this message or the help of the given subcommand(s)

Options:
  -u, --url <GITEA_URL>  [env: GTTOOL_GITEA_URL=]
  -r, --repo <REPO>      [env: GTTOOL_FQRN=]
  -h, --help             Print help
  -V, --version          Print version
```

### Authentication

Authentication is token-based via environment variable `RELEASE_KEY_GITEA`.

Ensure your token has the appropriate access for your usage. This depends on what you're doing and how your Gitea instance is configured, so you'll have to figure it out for yourself.

Most likely, you will need a token with "repository: read-and-write" permissions. See Gitea's documentation on [token scopes](https://docs.gitea.com/development/oauth2-provider#scopes) for more.

### `<GITEA_URL>`:

The Gitea server URL must be provided with `--url` or `-u` on the command line, or via the environment variable `GTTOOL_GITEA_URL`. Use the base URL for your Gitea instance.

E.g.: Using the Gitea org's demo instance, it would be: `--url "https://demo.gitea.com/"`

### `<REPO>`:

The repository name must be provided with `--repo` or `-u` on the command line, or via the environment variable `GTTOOL_GITEA_FQRN` ("fully qualified repo name"). Use the format `<owner>/<repo>`, which is the route immediately following the GITEA_URL base. This is how GitHub and Gitea identify repos in the URL, and how Golang locates it's modules, so this tool does the same.

E.g.: `--repo "go-gitea/gitea"` would name the Gitea repo belonging to the go-gitea organization.

### `<COMMAND>`:

One of these, defaults to `help`:

| Command | Description |
|-|-|
| list-releases | Prints all releases for the given repo. |
| create-release | Creates a new release. It is recommended to use the web page, but this will work in case you need it. |
| upload-release | Uploads one-or-more files to an existing release, identified by it's tag name. |
| help | prints the help text (the usage summary above). |

