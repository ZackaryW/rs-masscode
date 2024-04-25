
use std::collections::HashMap;

use clap;
use rs_masscode::model;


fn santize<T: serde::Serialize>(data : &HashMap<String, T>){
    // create string using serde.serialize
    let mut vec = Vec::new();
    for (_, folder) in data.iter() {
        vec.push(folder);
    }
    serde_json::to_string_pretty(&vec.clone()).unwrap();
}

fn setup_command() -> clap::Command {
    clap::Command::new("rs-masscode")
    .version("1.0")
    .author("ZackaryW")
    .about("Handles various resources")
    .arg(clap::Arg::new("query")
        .short('q')
        .num_args(0..)
        .long("query")
        .help("Query to use for filtering results")
        .action(clap::ArgAction::Set)
        .value_name("QUERY")
    )       
    .arg(clap::Arg::new("all")
        .long("all")
        .action(clap::ArgAction::SetTrue)
        .help("Returns a dictionary copy of everything"))
    .arg(clap::Arg::new("folders")
        .long("folders")
        .action(clap::ArgAction::SetTrue)
        .help("Returns list of folders"))
    .arg(clap::Arg::new("tags")
        .long("tags")
        .action(clap::ArgAction::SetTrue)
        .help("Returns list of tags"))
    .arg(clap::Arg::new("snippets")
        .long("snippets")
        .action(clap::ArgAction::SetTrue)
        .help("Returns list of snippets"))
    .arg(clap::Arg::new("id")
        .long("id")
        .action(clap::ArgAction::SetTrue)
        .help("Return only a list of ids"))
    .arg(clap::Arg::new("content")
        .long("content")
        .action(clap::ArgAction::SetTrue)
        .help("Returns the content of a snippet")
    )
}

fn execute( matches: &clap::ArgMatches) {
    let _only_ids = matches.get_flag("id");
    let _content = matches.get_flag("content");
    // if debug
    // merge all query together
    let mut basequery = String::new();
    
    // if got query
    if matches.get_many::<String>("query").is_some() {
        for q in matches.get_many::<String>("query").unwrap() {
            if basequery.is_empty() {
                basequery = String::from(q);
            } else {
                basequery = format!("( {} ) & ( {} )", basequery, String::from(q));
            }
        }
        
    }
    

    //println!("Base query: {}", basequery);
    //println!("All flag: {:?}", matches.get_flag("all"));
    //println!("Folders flag: {:?}", matches.get_flag("folders"));
    //println!("Tags flag: {:?}", matches.get_flag("tags"));
    //println!("Snippets flag: {:?}", matches.get_flag("snippets"));
    //println!("ID flag: {:?}", matches.get_flag("id"));
    //println!("Content flag: {:?}", matches.get_flag("content"));


    let mut loader = rs_masscode::Loader::new(None, None);

    println!("Target DB: {}", loader.db_path().display());
    let db : &model::StorageData = loader.db_content().unwrap();


    // if not query
    if  basequery.is_empty() {
        if matches.get_flag("all") {
            println!("{:#?}", santize(&db.snippets));
            println!("{:#?}", santize(&db.folders));
            println!("{:#?}", santize(&db.tags));
        }
        else if matches.get_flag("folders") {
            println!("{:#?}", santize(&db.folders));
        }
        else if matches.get_flag("tags") {
            println!("{:#?}", santize(&db.tags));
        }
        else if matches.get_flag("snippets") {
            println!("{:#?}", santize(&db.snippets));
        }
        else {
            println!("No option selected");
        }
    } else {
        

        if matches.get_flag("all") {
            println!("Error: --all cannot be used with --query");
        }
        else if matches.get_flag("folders") {
            let queryed = loader.query_folders(&basequery);
            if _only_ids {
                let ids : Vec<String> = queryed.iter().map(|f| f.id.clone()).collect();
                println!("{}", serde_json::to_string_pretty(&ids).unwrap());
            } else {
                println!("{}", serde_json::to_string_pretty(&queryed).unwrap());
            }
        }
        else if matches.get_flag("tags") {
            let queryed = loader.query_tags(&basequery);
            if _only_ids {
                let ids : Vec<String> = queryed.iter().map(|f| f.id.clone()).collect();
                println!("{}", serde_json::to_string_pretty(&ids).unwrap());
            } else {
                println!("{}", serde_json::to_string_pretty(&queryed).unwrap());
            }
        }
        else if matches.get_flag("snippets") {
            let queryed = loader.query_snippets(&basequery);
            if _only_ids {
                let ids : Vec<String> = queryed.iter().map(|f| f.id.clone()).collect();
                println!("{:?}", ids);
            } else if _content {
                // if not only 1 
                if queryed.len() == 1 {
                    println!("{}", serde_json::to_string_pretty(&queryed[0].content).unwrap());
                }
                else {
                    println!("{}", serde_json::to_string_pretty(&queryed).unwrap());
                }
            }
        }

        else {
            println!("No option selected");
            return;
        }

    
    }
}


fn main() {
    let matches = setup_command().get_matches();
    execute(&matches);
}

//test
#[test]
fn test_main() {
    let matches = setup_command().get_matches_from(vec![ "rs-masscode", "--folders", "--query", "name @SW g"]);
    execute(&matches);
}