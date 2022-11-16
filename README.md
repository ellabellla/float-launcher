# Float Launcher

A tui application launcher. The launcher stores commands, and their metadata, in a json file and then queries it based on search terms. When an application is selected to launch its command is ran using bash.


## Usage
```bash
Float Launcher: A tui application launcher.

Usage: fl [OPTIONS] [COMMAND]

Commands:
  add     Add a command to the launcher
  remove  Remove a command from the launcher
  launch  Open the launcher (running with no subcommands will also open the launcher)
  help    Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  Config Directory
  -h, --help             Print help information
  -V, --version          Print version information

```

### Add
```bash
Add a command to the launcher

Usage: fl add <NAME> <DESCRIPTION> <COMMAND> [TAGS]...

Arguments:
  <NAME>         Name
  <DESCRIPTION>  Description
  <COMMAND>      Command
  [TAGS]...      Tags

Options:
  -h, --help  Print help information
```

### Remove
```bash
Remove a command from the launcher

Usage: fl remove <NAME>

Arguments:
  <NAME>  Name

Options:
  -h, --help  Print help information
```

## License
This software is provided under the MIT license. Click [here](LICENSE) to view.