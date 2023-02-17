use std::path::{Path, PathBuf};

pub fn get_files_recursive(start: &Path) -> Vec<PathBuf> {
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
