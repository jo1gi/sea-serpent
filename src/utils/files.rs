use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct FileSearchSettings {
    pub recursive: bool,
    pub stdin: bool,
    pub filetype_filter: FiletypeFilter,
}

pub enum FiletypeFilter {
    All,
    FilesOnly,
    FoldersOnly
}

pub fn get_files(roots: &Vec<PathBuf>, settings: FileSearchSettings) -> Vec<PathBuf> {
    let stdin_files = get_stdin_files(&settings);
    roots.into_iter()
        .chain(stdin_files.iter())
        .flat_map(|root| {
            if settings.recursive {
                get_files_recursive(root)
            } else {
                vec![root.clone()]
            }
        })
        .filter(|file| match settings.filetype_filter {
            FiletypeFilter::All => true,
            FiletypeFilter::FilesOnly => file.is_file(),
            FiletypeFilter::FoldersOnly => file.is_dir(),
        })
        .collect()
}

fn get_stdin_files(settings: &FileSearchSettings) -> Vec<PathBuf> {
    if settings.stdin {
        std::io::stdin()
            .lines()
            .filter_map(|line| PathBuf::from_str(&line.ok()?).ok())
            .collect()
    } else {
        Vec::new()
    }
}

fn get_files_recursive(start: &Path) -> Vec<PathBuf> {
    let mut output = vec![start.to_path_buf()];
    add_files_from_dir(start, &mut output);
    return output;
}

fn add_files_from_dir(path: &Path, list: &mut Vec<PathBuf>) {
    if path.is_dir() {
        // TODO Remove unwrap
        for file in std::fs::read_dir(path).unwrap() {
            if let Ok(file) = file {
                let path = file.path();
                add_files_from_dir(&path, list);
                list.push(path);
            }
        }
    }
}
