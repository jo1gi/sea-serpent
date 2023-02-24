# Sea Serpent
![GitHub release](https://img.shields.io/github/v/release/jo1gi/sea-serpent)
![GitHub top language](https://img.shields.io/github/languages/top/jo1gi/sea-serpent)
![License](https://img.shields.io/github/license/jo1gi/sea-serpent)
[![Donate using Ko-Fi](https://img.shields.io/badge/donate-kofi-00b9fe?logo=ko-fi&logoColor=00b9fe)](https://ko-fi.com/jo1gi)

Basic cli file tagging

**This project is still early in development and can change at any moment**

Sea-serpent uses a local database (a single directory and its subdirectories) to
store tags for files.

## Installation
Installation requires cargo which can be installed from https://www.rust-lang.org/tools/install.
```shell
cargo install --git "https://github.com/jo1gi/sea-serpent.git"
```

## Usage

### Initialize database
To initialize a new sea-serpent databse simply run:
```shell
sea-serpent init
```
This will create a new folder in the current directory called `.sea-serpent`
which will contain all files for sea-serpent.

### Adding tags
To add a tag to a file simple run:
```shell
sea-serpent add -t <tag> <files>
```

You can also tag a directory and all its content recusively by using the
`--recursive` or the `-r` argument:
```shell
sea-serpent add -t <tag> <directory> --recursive
```

To only add files, use `--exclude-dirs`:
```shell
sea-serpent add -t <tag> <directory> --recursive --exclude-dirs
```

To see more options use:
```shell
sea-serpent add --help
```

### Searching
To search for files with specific tags use:
```shell
sea-serpent search <tag1> <tag2>
```

To search for files with *either* `tag1` or `tag2` run:
```shell
sea-serpent search <tag1> or <tag2>
```

You can exlude tags with the `not` keyword:
```shell
sea-serpent search <tag1> and not <tag2>
```

All this can be combined to more complex operation with grouping:
```shell
sea-serpent search (<tag1> or <tag2>) and (<tag3> or not <tag2>)
```

## Contributions
Issues, bug-reports, pull requests or ideas for features and improvements are
**very welcome**.

## Donations
If you like the project, please consider donating:
- [Ko-fi](https://ko-fi.com/jo1gi)
- [Buy me a Coffee](https://www.buymeacoffee.com/joakimholm)
