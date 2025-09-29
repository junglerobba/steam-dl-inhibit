use serde::Deserialize;
use std::{
    env::{self, home_dir},
    fs::{self},
    path::PathBuf,
};

const DEFAULT_LIBRARY_PATHS: &[&str] = &[
    ".local/share/Steam",
    ".var/app/com.valvesoftware.Steam/data/Steam",
];

const VDF_PATH: &str = "steamapps/libraryfolders.vdf";

#[derive(Deserialize, Debug)]
struct Libraryfolders {
    libraryfolders: Vec<Libraryfolder>,
}

#[derive(Debug, Deserialize)]
struct Libraryfolder {
    path: PathBuf,
}

pub fn get_library_paths() -> Option<Vec<PathBuf>> {
    let home = home_dir()?;
    let librarypath = env::var("STEAM_DIR")
        .map(|str| vec![str])
        .unwrap_or_else(|_| {
            DEFAULT_LIBRARY_PATHS
                .iter()
                .map(|str| str.to_string())
                .collect::<Vec<_>>()
        });
    let libraryfolders = librarypath
        .iter()
        .map(|path| home.join(path))
        .map(|path| path.join(VDF_PATH))
        .filter_map(|path| fs::read_to_string(path).ok())
        .map(|str| vdf_reader::from_str::<Libraryfolders>(&str).unwrap())
        .flat_map(|f| f.libraryfolders.into_iter())
        .map(|f| f.path)
        .collect::<Vec<_>>();

    Some(libraryfolders)
}
