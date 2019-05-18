# pls

pls allows you to list the current activity of your Plex Server via a Tautulli (Plexpy) connection.

## Example Output

![Example Output](https://i.imgur.com/wJu1Ayl.png)

## Configuration

pls processes the Tautulli server and key in the following order:
- command line flag
- env variable: PLEXPY_SERVER & PLEXPY_KEY
- config file

The optional config can be stored in your OS specific configuration directory:

| Platform | Value                                  | Example                                    |
| -------- | -------------------------------------- | ------------------------------------------ |
| Linux    | `$XDG_CONFIG_HOME` or `$HOME`/.config  | /home/alice/.config/pls.toml               |
| macOS    | `$HOME`/Library/Preferences            | /Users/Alice/Library/Preferences/pls.toml  |
| Windows  | `{FOLDERID_RoamingAppData}`            | C:\Users\Alice\AppData\Roaming\pls.toml    |

The config looks like:

```toml
server = 'https://plexpy.duckdns.com:8181'
key = 'my-super-secret-plexpy-key'
```
