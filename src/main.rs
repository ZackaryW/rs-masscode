use clap::{Command, Arg, ArgMatches};
use rs_masscode;

fn main() {
    let app = Command::new("rs-masscode")
        .version("1.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("query")
            .about("Executes a query on a dataset")
            .arg(Arg::new("type")
                .help("The type of query to execute: snippet, folder, or tag")
                .required(true)
                .index(1))
            .arg(Arg::new("query")
                .help("The query string")
                .required(true)
                .index(2))
            .arg(Arg::new("tag")
                .help("Specifies a tag for snippet querying")
                .long("tag")
            )
            .arg(Arg::new("folder")
                .help("Specifies a folder for snippet querying")
                .long("folder")
            ));
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("query", sub_matches)) => handle_query(sub_matches),
        _ => unreachable!(), // Due to `subcommand_required(true)`
    }
}

fn handle_query(matches: &ArgMatches) {
    let query_type = matches.get_one::<String>("type").unwrap();
    if query_type != "snippet" && query_type != "folder" && query_type != "tag" {
        eprintln!("Invalid query type: {}", query_type);
        return;
    }
    let query = matches.get_one::<String>("query").unwrap();
    let tag = matches.get_one::<String>("tag");
    let folder = matches.get_one::<String>("folder");

    // if tag or folder specified not on sippet
    if (tag.is_some() || folder.is_some()) && query_type != "snippet" {
        eprintln!("Only snippets can have tags or folders as subquery specifiers");
        return;
    }
    let app_loader = rs_masscode::apploader::Loader::new(None);
    let mut engine = rs_masscode::qry::QueryEngine::new("data", None, Some(&app_loader.load_db_path())).unwrap();

    let res : String;
    if query_type == "tag" {
        res = rs_masscode::query_tags(&mut engine, query);
    } else if query_type == "folder" {
        res = rs_masscode::query_folders(&mut engine, query);
    } else {
        res = rs_masscode::query_snippets(&mut engine, query);
    }

    println!("{}", res);
}
