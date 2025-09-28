use anyhow::Result;
use dbus::{
    arg::OwnedFd,
    blocking::{Connection, Proxy},
};
use inotify::{Event, EventMask, Inotify, WatchMask};
use serde::Deserialize;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    os::fd::FromRawFd,
    path::PathBuf,
    time::Duration,
};

use crate::library::get_library_paths;

mod library;

const DL_PATH: &str = "steamapps/downloading";

const MANIFEST_PATH: &str = "steamapps";

struct State<'a> {
    proxy: Proxy<'a, &'a Connection>,
    fds: HashMap<usize, File>,
    libraries: Vec<PathBuf>,
    app_names: HashMap<usize, String>,
}

#[derive(Debug, Deserialize)]
struct AppManifest {
    #[serde(rename = "AppState")]
    appstate: App,
}

#[derive(Debug, Deserialize)]
struct App {
    name: String,
}

fn main() -> Result<()> {
    let libraries = get_library_paths().unwrap_or_default();
    println!("Libraries found: {:?}", libraries);
    let dl_paths = libraries.iter().map(|path| path.join(DL_PATH));

    let mut inotify = Inotify::init()?;

    for path in dl_paths {
        inotify.watches().add(
            &path,
            WatchMask::ACCESS | WatchMask::CREATE | WatchMask::CLOSE_WRITE,
        )?;
    }
    let mut buffer = [0u8; 4096];
    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        Duration::from_secs(10),
    );

    let mut manager = State::new(proxy, libraries);
    loop {
        let events = inotify.read_events_blocking(&mut buffer)?;
        for event in events {
            manager.process(&event)?;
        }
    }
}

impl<'a> State<'a> {
    pub fn new(proxy: Proxy<'a, &'a Connection>, libraries: Vec<PathBuf>) -> Self {
        Self {
            proxy,
            fds: HashMap::new(),
            libraries,
            app_names: HashMap::new(),
        }
    }

    pub fn process(&mut self, event: &Event<&OsStr>) -> Result<()> {
        let Some(name) = event.name.and_then(|name| name.to_str()) else {
            return Ok(());
        };
        if !name.starts_with("state_") {
            return Ok(());
        }
        let split = name.split('_').collect::<Vec<_>>();
        let Some(appid) = split.get(1) else {
            return Ok(());
        };
        let appid: usize = appid.parse()?;
        match event.mask {
            EventMask::ACCESS | EventMask::CREATE => {
                println!(
                    "Download started for appid {appid}: {:?} {:?}",
                    event.name, event.mask
                );
                self.start_inhibit(appid)?;
            }
            EventMask::CLOSE_WRITE => {
                println!(
                    "Download stopped for appid {appid}: {:?} {:?}",
                    event.name, event.mask
                );
                self.stop_inhibit(appid);
            }
            _ => {}
        };
        Ok(())
    }

    fn start_inhibit(&mut self, appid: usize) -> Result<()> {
        let Some(app_name) = self.get_app_name(appid) else {
            return Ok(());
        };
        let (fd,): (OwnedFd,) = self.proxy.method_call(
            "org.freedesktop.login1.Manager",
            "Inhibit",
            (
                "sleep",
                "Steam",
                format!("Downloading {}", app_name),
                "block",
            ),
        )?;
        let file = unsafe { File::from_raw_fd(fd.into_fd()) };
        self.fds.insert(appid, file);

        Ok(())
    }

    fn stop_inhibit(&mut self, appid: usize) {
        self.fds.remove(&appid);
        self.app_names.remove(&appid);
    }

    fn get_app_name(&mut self, appid: usize) -> Option<String> {
        match self.app_names.get(&appid) {
            Some(name) => Some(name.to_owned()),
            None => {
                let manifest = self
                    .libraries
                    .iter()
                    .map(|path| path.join(MANIFEST_PATH))
                    .map(|path| path.join(format!("appmanifest_{}.acf", appid)))
                    .filter(|file| file.is_file())
                    .take(1)
                    .next()?;
                let file = fs::read_to_string(manifest).ok()?;
                let content = vdf_reader::from_str::<AppManifest>(&file).ok()?;
                self.app_names.insert(appid, content.appstate.name.clone());
                Some(content.appstate.name)
            }
        }
    }
}
