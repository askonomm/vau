use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use regex::Regex;
use serde::Deserialize;
use siena::providers::local::LocalProvider;
use siena::siena::{siena, Record, RecordSortOrder, Siena};
use std::fs;
use std::path::Path;
use std::time::Duration;
use std::{env, io};
use tera::{Context, Tera};
use thiserror::Error;

const ROOT_DIR: &str = "./";

fn store() -> Siena {
    siena(LocalProvider {
        directory: format!("{}data", ROOT_DIR).to_string(),
    })
}

#[derive(Error, Debug)]
enum Error {
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

#[derive(Deserialize, Debug)]
struct ConfigDataDSLWhenIs {
    key: String,
    equals: String,
}

#[derive(Deserialize, Debug)]
struct ConfigDataDSLWhenIsNot {
    key: String,
    equals: String,
}

#[derive(Deserialize, Debug)]
struct ConfigDataDSLWhenHas {
    key: String,
}

#[derive(Deserialize, Debug)]
struct ConfigDataDSLWhenHasNot {
    key: String,
}

#[derive(Deserialize, Debug)]
struct ConfigDataDSLWhenMatches {
    key: String,
    regex: String,
}

#[derive(Deserialize, Debug)]
struct ConfigDataDSLSort {
    key: String,
    order: String,
}

#[derive(Deserialize, Debug)]
struct ConfigDataDSL {
    name: String,
    collection: String,
    when_is: Option<ConfigDataDSLWhenIs>,
    when_is_not: Option<ConfigDataDSLWhenIsNot>,
    when_has: Option<ConfigDataDSLWhenHas>,
    when_has_not: Option<ConfigDataDSLWhenHasNot>,
    when_matches: Option<ConfigDataDSLWhenMatches>,
    sort: Option<ConfigDataDSLSort>,
    limit: Option<usize>,
}

#[derive(Deserialize, Debug)]
struct ConfigPagesDSLPage {
    path: String,
}

#[derive(Deserialize, Debug)]
struct ConfigPagesDSL {
    collection: Option<String>,
    page: ConfigPagesDSLPage,
    template: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    data: Option<Vec<ConfigDataDSL>>,
    pages: Option<Vec<ConfigPagesDSL>>,
}

fn read_config() -> Result<Config, Error> {
    let config_path = format!("{}config.toml", ROOT_DIR);
    let config = fs::read_to_string(config_path)?;
    let parsed_config: Config = toml::from_str(&config)?;

    Ok(parsed_config)
}

fn delete_public_dir() -> Result<(), Error> {
    if Path::new(&format!("{}public/", ROOT_DIR)).exists() {
        fs::remove_dir_all(&format!("{}public/", ROOT_DIR))?;
    }

    Ok(())
}

fn compose_tera_context() -> Result<Context, Error> {
    let config = read_config()?;
    let mut context = Context::new();

    if config.data.is_some() {
        for data in config.data.unwrap() {
            let mut records = store().collection(&data.collection);

            // when_is
            if data.when_is.is_some() {
                let when_is = data.when_is.unwrap();
                records = records.when_is(&when_is.key, &when_is.equals);
            }

            // when_is_not
            if data.when_is_not.is_some() {
                let when_isnt = data.when_is_not.unwrap();
                records = records.when_isnt(&when_isnt.key, &when_isnt.equals);
            }

            // when_has
            if data.when_has.is_some() {
                let when_has = data.when_has.unwrap();
                records = records.when_has(&when_has.key);
            }

            // when_has_not
            if data.when_has_not.is_some() {
                let when_has_not = data.when_has_not.unwrap();
                records = records.when_hasnt(&when_has_not.key);
            }

            // when_matches
            if data.when_matches.is_some() {
                let when_matches = data.when_matches.unwrap();
                records = records.when_matches(&when_matches.key, &when_matches.regex);
            }

            // sort
            if data.sort.is_some() {
                let sort = data.sort.unwrap();
                let order = match sort.order.clone().as_str() {
                    "asc" => RecordSortOrder::Asc,
                    "desc" => RecordSortOrder::Desc,
                    &_ => RecordSortOrder::Asc,
                };

                records = records.sort(&sort.key, order);
            }

            // limit
            if data.limit.is_some() {
                records = records.limit(data.limit.unwrap());
            }

            context.insert(data.name, &records.get_all());
        }
    }

    Ok(context)
}

// Helper function to get everything before a character
fn str_before_char(input: &str, char: &str) -> String {
    let parts: Vec<&str> = input.split(char).collect();

    parts[0..parts.len() - 1].join(char)
}

fn parse_page_path(path: &str, record: &Record) -> Result<String, Error> {
    let mut parsed_path = path.to_string();
    let vars_in_path_re = Regex::new(r"\{(\w+)}")?;

    // Parse path
    for (_, [var]) in vars_in_path_re.captures_iter(&path).map(|c| c.extract()) {
        let needle = &format!("{{{}}}", var);
        let val = record.data.get(var).unwrap();

        parsed_path = parsed_path.replace(needle, val);
    }

    Ok(parsed_path)
}

fn compile_collection_pages(tera: &Tera, dsl: &ConfigPagesDSL) -> Result<(), Error> {
    let mut context = compose_tera_context()?;
    let records = store()
        .collection(&dsl.collection.as_ref().unwrap())
        .get_all();

    for record in records {
        context.insert("record", &record);

        let path = parse_page_path(&dsl.page.path, &record)?;
        let rendered = tera.render(&dsl.template, &context)?;
        let dir_path = format!("{}public/{}", ROOT_DIR, str_before_char(&path, "/"));
        let file_path = format!("{}public/{}", ROOT_DIR, path);

        println!("Compiling {}", file_path);

        fs::create_dir_all(dir_path)?;
        fs::write(file_path, rendered)?;
    }

    Ok(())
}

fn compile_page(tera: &Tera, dsl: &ConfigPagesDSL) -> Result<(), Error> {
    let context = compose_tera_context()?;
    let path = &dsl.page.path;
    let rendered = tera.render(&dsl.template, &context)?;
    let dir_path = format!("{}public/{}", ROOT_DIR, str_before_char(&path, "/"));
    let file_path = format!("{}public/{}", ROOT_DIR, path);

    println!("Compiling {}", file_path);

    fs::create_dir_all(dir_path)?;
    fs::write(file_path, rendered)?;

    Ok(())
}

fn compile_pages(tera: &Tera) -> Result<(), Error> {
    let config = read_config()?;

    if config.pages.is_some() {
        for page in config.pages.unwrap() {
            // A whole collection
            if page.collection.is_some() {
                compile_collection_pages(&tera, &page)?;
            }
            // Otherwise just a single page
            else {
                compile_page(&tera, &page)?
            }
        }
    }

    Ok(())
}

fn compile() -> Result<(), Error> {
    let tera = Tera::new(&format!("{}templates/**/*", ROOT_DIR))?;

    delete_public_dir()?;
    compile_pages(&tera)?;

    Ok(())
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
