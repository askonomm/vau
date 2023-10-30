mod compiler;
pub mod components;
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
use vizia::prelude::*;

const ROOT_DIR: &str = "./";

fn store() -> Siena {
    siena(LocalProvider {
        directory: format!("{}data", ROOT_DIR).to_string(),
    })
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Watcher failed: {0}")]
    Notify(#[from] notify::Error),
    #[error("Templating failed: {0}")]
    Tera(#[from] tera::Error),
    #[error("File system failed: {0}")]
    Io(#[from] io::Error),
    #[error("Config read failed: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Regex failed: {0}")]
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

    if args.len() > 1 && args[1] == "--gui" {
        Application::new(|cx| {
            HStack::new(cx, |cx| {
                components::sidebar(cx);
                Label::new(cx, "content area");
            });
        })
        .title("Vau")
        .inner_size((840, 600))
        .run();

        return Ok(());
    }

    compile()?;

    if args.len() > 1 && args[1] == "--watch" {
        watch()?;
    }

    Ok(())
}
