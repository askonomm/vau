use crate::config::{read_config, ConfigPagesDSL};
use crate::{store, utils, Error, ROOT_DIR};
use regex::Regex;
use siena::siena::{Record, RecordData, RecordSortOrder};
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

pub fn delete_public_dir() -> Result<(), Error> {
    if Path::new(&format!("{}public/", ROOT_DIR)).exists() {
        fs::remove_dir_all(&format!("{}public/", ROOT_DIR))?;
    }

    Ok(())
}

fn compose_tera_context() -> Result<Context, Error> {
    let config = read_config()?;
    let mut context = Context::new();

    let Some(data_items) = config.data else {
        return Ok(context);
    };

    for data in data_items {
        let mut records = store().collection(&data.collection);

        // when_is
        if data.when_is.is_some() {
            let when_is = data.when_is.unwrap();
            records = records.when_is(&when_is.key, &when_is.equals);
        }

        // when_is_not
        if data.when_is_not.is_some() {
            let when_isnt = data.when_is_not.unwrap();
            records = records.when_is_not(&when_isnt.key, &when_isnt.equals);
        }

        // when_has
        if data.when_has.is_some() {
            let when_has = data.when_has.unwrap();
            records = records.when_has(&when_has.key);
        }

        // when_has_not
        if data.when_has_not.is_some() {
            let when_has_not = data.when_has_not.unwrap();
            records = records.when_has_not(&when_has_not.key);
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

        if data.first.is_some() {
            context.insert(data.name, &records.get_first());
        } else if data.last.is_some() {
            context.insert(data.name, &records.get_last());
        } else {
            context.insert(data.name, &records.get_all());
        }
    }

    Ok(context)
}

fn parse_page_path(path: &str, record: &Record) -> Result<String, Error> {
    let mut parsed_path = path.to_string();
    let vars_in_path_re = Regex::new(r"\{(\w+)}")?;

    // Parse path
    for (_, [var]) in vars_in_path_re.captures_iter(&path).map(|c| c.extract()) {
        let needle = &format!("{{{}}}", var);

        if record.data.get(var).is_none() {
            continue;
        }

        let val = record.data.get(var).unwrap();

        match val {
            RecordData::Str(s) => parsed_path = parsed_path.replace(needle, s),
            RecordData::Num(n) => parsed_path = parsed_path.replace(needle, &n.to_string()),
            _ => (),
        }
    }

    Ok(parsed_path)
}

fn compile_collection_pages(tera: &Tera, dsl: &ConfigPagesDSL, ctx: &Context) -> Result<(), Error> {
    let mut context = ctx.clone();
    let records = store()
        .collection(&dsl.collection.as_ref().unwrap())
        .get_all();

    for record in records {
        context.insert("record", &record);

        let path = parse_page_path(&dsl.page.path, &record)?;
        let rendered = tera.render(&dsl.template, &context)?;
        let dir_path = format!("{}public/{}", ROOT_DIR, utils::str_before_char(&path, "/"));
        let file_path = format!("{}public/{}", ROOT_DIR, path);

        println!("Compiling {}", file_path);

        fs::create_dir_all(dir_path)?;
        fs::write(file_path, rendered)?;
    }

    Ok(())
}

fn compile_page(tera: &Tera, dsl: &ConfigPagesDSL, ctx: &Context) -> Result<(), Error> {
    let path = &dsl.page.path;
    let rendered = tera.render(&dsl.template, ctx)?;
    let dir_path = format!("{}public/{}", ROOT_DIR, utils::str_before_char(&path, "/"));
    let file_path = format!("{}public/{}", ROOT_DIR, path);

    println!("Compiling {}", file_path);

    fs::create_dir_all(dir_path)?;
    fs::write(file_path, rendered)?;

    Ok(())
}

fn compile_pages(tera: &Tera) -> Result<(), Error> {
    let config = read_config()?;
    let context = compose_tera_context()?;

    let Some(pages) = config.pages else {
        return Ok(());
    };

    for page in pages {
        // A whole collection
        if page.collection.is_some() {
            compile_collection_pages(&tera, &page, &context)?;
        }
        // Otherwise just a single page
        else {
            compile_page(&tera, &page, &context)?
        }
    }

    Ok(())
}

pub fn compile() -> Result<(), Error> {
    let tera = Tera::new(&format!("{}templates/**/*", ROOT_DIR))?;

    delete_public_dir()?;
    compile_pages(&tera)?;

    Ok(())
}
