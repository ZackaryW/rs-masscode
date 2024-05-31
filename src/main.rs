use clap::{Arg, ArgMatches, Command};
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
                .index(1)
            )
            .arg(Arg::new("query")
                .help("The query string")
                .required(true)
                .index(2)
            )
            
            .arg(Arg::new("tag")
                .help("Specifies a tag for snippet querying")
                .long("tag")
            )
            .arg(Arg::new("folder")
                .help("Specifies a folder for snippet querying")
                .long("folder")
            )
            .arg(
                Arg::new("id")
                    .help("Return ids instead of snippets")
                    .long("id")
                    .short('i')
                    .action(clap::ArgAction::SetTrue),
            )
        );
            
    let matches = app.get_matches();
    let app_loader = rs_masscode::apploader::Loader::new(None);
    match matches.subcommand() {
        Some(("query", sub_matches)) => handle_query(sub_matches, app_loader),
        _ => {}
    }
}

fn handle_query(matches: &ArgMatches, app_loader: rs_masscode::apploader::Loader) {
    let query_type = matches.get_one::<String>("type").unwrap();
    if query_type != "snippet" && query_type != "folder" && query_type != "tag" {
        eprintln!("Invalid query type: {}", query_type);
        return;
    }
    let query = matches.get_one::<String>("query").unwrap();

    // get options
    let tag = matches.get_one::<String>("tag");
    let folder = matches.get_one::<String>("folder");
    let return_ids = matches.get_flag("id");

    // if tag or folder specified not on sippet
    if (tag.is_some() || folder.is_some()) && query_type != "snippet" {
        eprintln!("Only snippets can have tags or folders as subquery specifiers");
        return;
    }
    
    let mut engine = rs_masscode::qry::QueryEngine::new("data", None, Some(&app_loader.load_db_path())).unwrap();

    let res : String;
    if query_type == "tag" {
        res = rs_masscode::query_tags(&mut engine, query, return_ids);
    } else if query_type == "folder" {
        res = rs_masscode::query_folders(&mut engine, query, return_ids);
    } else {
        res = rs_masscode::query_snippets(&mut engine, query, return_ids, folder, tag);
    }

    println!("{}", res);
}


#[test]
fn query_snippets(){
    let loader = rs_masscode::apploader::Loader::new(None);
    let mut engine = rs_masscode::qry::QueryEngine::new("data", None, Some(&loader.load_db_path())).unwrap();
    let tag_query = String::from("tag.name.startsWith('s')");
    let res = rs_masscode::query_snippets(&mut engine, "snippet", true, None, Some(&tag_query));
    println!("{}", res);
}