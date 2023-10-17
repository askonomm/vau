use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
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
    let mut context = tera::Context::new();

    // Latest posts
    let posts = store()
        .collection("posts")
        .when_is("status", "published")
        .sort("date", RecordSortOrder::Desc)
        .limit(10)
        .get_all();

    context.insert("posts", &posts);

    // Projects
    let projects = store()
        .collection("projects")
        .sort("order", RecordSortOrder::Asc)
        .get_all();

    context.insert("projects", &projects);

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
