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

# Note
1. You can execute ```cargo tizen dev -e``` to compile, package and run the project in one step (-e stands for --emulator)
2. You need to have a valid ```tizen-manifest.xml```
