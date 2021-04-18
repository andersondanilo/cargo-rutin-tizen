# cargo-rutin-tizen
Cargo tool to compile and run tizen wearable applications (wrapper arround tizen sdk tools)

# Installation
- 1. Install 'cargo-rutin-tizen'
- 2. Create '.cargo/config' (or modify the existing), adding the path to tizen studio:
```toml
[tizen]
studio_path = "/home/MYUSER/Tizen/tizen-studio"
```
- 3. Now you can run the command ```cargo tizen --help``` to see all available commands, and ```cargo tizen config``` to see all configurable options

## Usage
```console
$ cargo tizen --help
cargo-rutin-tizen 0.1.0
Anderson Danilo <contact@andersondanilo.com>
Cargo tool to compile and run tizen applications (wrapper arround tizen sdk tools)

USAGE:
    cargo-tizen <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    build      Wrapper arround cargo build
    clean      Wrapper arround cargo clean
    config     Show config used for building the app
    dev        Build, package, install and run
    help       Prints this message or the help of the given subcommand(s)
    install    Wrapper arround tizen install
    package    Wrapper arround tizen package
    run        Wrapper arround tizen run
```

- You can execute ```cargo tizen dev -e``` to compile, package and run the project in one step (-e stands for --emulator)
- You need to have a valid ```tizen-manifest.xml```
- The ```config``` command show all computed configurations, and show you the Env var and Xml attr (cargo) you can use to personalize
