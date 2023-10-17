use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use serde::Deserialize;
use siena::providers::local::LocalProvider;
use siena::siena::{siena, RecordSortOrder, Siena};
use std::env;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tera::Tera;

const ROOT_DIR: &str = "./";

fn store() -> Siena {
    siena(LocalProvider {
        directory: format!("{}data", ROOT_DIR).to_string(),
    })
}

#[derive(Deserialize, Debug)]
struct ConfigDataDSLWhenIs {
    key: String,
    equals: String,
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
    sort: Option<ConfigDataDSLSort>,
    limit: Option<usize>,
}

#[derive(Deserialize, Debug)]
struct Config {
    data: Option<Vec<ConfigDataDSL>>,
}

fn read_config() -> Config {
    let config_path = format!("{}config.toml", ROOT_DIR);
    let config = fs::read_to_string(config_path).unwrap();
    let parsed_config: Config = toml::from_str(&config).unwrap();

    parsed_config
}

fn delete_public_dir() {
    if Path::new(&format!("{}public/", ROOT_DIR)).exists() {
        fs::remove_dir_all(&format!("{}public/", ROOT_DIR))
            .expect("Could not remove public directory.");
    }
}

fn compose_blog_posts(tera: &Tera) {
    let posts = store()
        .collection("posts")
        .sort("date", RecordSortOrder::Desc)
        .get_all();

    for post in posts {
        match post.data.get("slug") {
            None => continue,
            Some(_) => {
                let mut context = tera::Context::new();
                context.insert("post", &post);

                let rendered = tera.render("post.html.tera", &context).unwrap();

                let dir_path =
                    format!("{}public/blog/{}", ROOT_DIR, post.data.get("slug").unwrap());

                let path = format!(
                    "{}public/blog/{}/index.html",
                    ROOT_DIR,
                    post.data.get("slug").unwrap()
                );

                println!(
                    "Compiling {} ...",
                    format!("blog/{}", post.data.get("slug").unwrap())
                );

                fs::create_dir_all(dir_path).unwrap();
                fs::write(path, rendered).unwrap();
            }
        }
    }
}

fn compose_home(tera: &Tera) {
    let config = read_config();
    let mut context = tera::Context::new();

    if config.data.is_some() {
        for data in config.data.unwrap() {
            let mut records = store().collection(&data.collection);

            // when_is
            if data.when_is.is_some() {
                let when_is = data.when_is.unwrap();
                records = records.when_is(&when_is.key, &when_is.equals);
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

    let rendered = tera.render("index.html.tera", &context).unwrap();

    println!("Compiling index ...");

    fs::create_dir_all(&format!("{}public/", ROOT_DIR)).unwrap();
    fs::write(&format!("{}public/index.html", ROOT_DIR), rendered).unwrap();
}

fn compile() {
    let tera = Tera::new(&format!("{}templates/**/*", ROOT_DIR)).unwrap();

    delete_public_dir();
    compose_blog_posts(&tera);
    compose_home(&tera);
}

fn watch() {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_secs(1), tx).unwrap();

    debouncer
        .watcher()
        .watch(
            Path::new(&format!("{}templates", ROOT_DIR)),
            RecursiveMode::Recursive,
        )
        .unwrap();

    debouncer
        .watcher()
        .watch(
            Path::new(&format!("{}data", ROOT_DIR)),
            RecursiveMode::Recursive,
        )
        .unwrap();

    for res in rx {
        match res {
            Ok(_) => compile(),
            Err(error) => println!("{}", error),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    compile();

    if args.len() > 1 && args[1] == "--watch" {
        watch();
    }
}
