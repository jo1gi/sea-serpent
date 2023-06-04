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
sea-serpent can be installed by downloading a pre-compiled binary from
[Release](https://github.com/jo1gi/sea-serpent/releases) or by compiling it
youself.

To compile sea-serpent youself you have to install [cargo](https://www.rust-lang.org/tools/install) and run:
```shell
cargo install --git "https://github.com/jo1gi/sea-serpent.git"
```

## Usage

### Initialize database
To initialize a new sea-serpent database simply run
```shell
sea-serpent init
```
This will create a new folder in the current directory called `.sea-serpent`
which will contain all files for sea-serpent.

### Adding tags
* Add a tag to a file
```shell
sea-serpent add -t <tag> -f <file>
```

* Add tag to directory and all its descendants with the `--recursive` or the `-r` argument
```shell
sea-serpent add -t <tag> -f <directory> --recursive
```

* Don't include directories with `--exclude-dirs`
```shell
sea-serpent add -t <tag> -f <directory> --recursive --exclude-dirs
```

* Tag files with key-value pairs by separating the key and the value with a
  colon
```shell
sea-serpent add -t <key>:<value> -f <files>
```

### Searching
* Search for files with specific tags
```shell
sea-serpent search <tag1> <tag2>
```

* Search for either `tag1` or `tag2` with `or` keyword
```shell
sea-serpent search <tag1> or <tag2>
```

* Exclude tags with `not` keyword
```shell
sea-serpent search <tag1> and not <tag2>
```

* Combine to more complex operation by grouping with parentheses
```shell
sea-serpent search (<tag1> or <tag2>) and (<tag3> or not <tag2>)
```

* Search for a key-value pair by adding a colon
```shell
sea-serpent search <key>:<value>
```

* Search for with a value (not key)
```shell
sea-serpent search :<value>
```
* Search for with a key (not value)
```shell
sea-serpent search <key>:
```

* Sort the search results by the values in a key-value pair
```shell
sea-serpent search <tag> --sort-by <key>
```

* Limit the amount of results
```shell
sea-serpent search <tag> --limit <number>
```

## Contributions
Issues, bug-reports, pull requests or ideas for features and improvements are
**very welcome**.

## Donations
If you like the project, please consider donating:
- [Ko-fi](https://ko-fi.com/jo1gi)
- [Buy me a Coffee](https://www.buymeacoffee.com/joakimholm)
