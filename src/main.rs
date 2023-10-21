mod compiler;
mod config;
mod utils;

use crate::compiler::compile;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use siena::providers::local::LocalProvider;
use siena::siena::{siena, Siena};
use std::path::Path;
use std::time::Duration;
use std::{env, io};
use thiserror::Error;

const ROOT_DIR: &str = "./";

fn store() -> Siena {
    siena(LocalProvider {
        directory: format!("{}data", ROOT_DIR).to_string(),
    })
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Watcher failed.")]
    Notify(#[from] notify::Error),
    #[error("Templating failed.")]
    Tera(#[from] tera::Error),
    #[error("File system failed.")]
    Io(#[from] io::Error),
    #[error("Config read failed.")]
    Toml(#[from] toml::de::Error),
    #[error("Regex failed.")]
    Regex(#[from] regex::Error),
}

fn watch() -> Result<(), Error> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_secs(1), tx)?;

    debouncer.watcher().watch(
        Path::new(&format!("{}templates", ROOT_DIR)),
        RecursiveMode::Recursive,
    )?;

    debouncer.watcher().watch(
        Path::new(&format!("{}data", ROOT_DIR)),
        RecursiveMode::Recursive,
    )?;

    for res in rx {
        return match res {
            Ok(_) => {
                compile()?;
                watch()?;

                Ok(())
            }
            Err(err) => Err(Error::Notify(err)),
        };
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    compile()?;

    if args.len() > 1 && args[1] == "--watch" {
        watch()?;
    }

    Ok(())
}
