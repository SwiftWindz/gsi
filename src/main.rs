use std::fs;
use std::collections::HashSet;
use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use std::string::String;
use clap::{Args, Parser, Subcommand};
use serde::Deserialize;
use csv;

// SSH URL for GitHub
const GITHUB_URL: &str = "git@github.com";
// Expected location of student roster if none other is specified
const DEFAULT_ROSTER_FILENAME: &str = "src/roster.csv";
// Expected location of config if none other is specified
const DEFAULT_CONFIG_PATH: &str = "src/conf.toml";

#[derive(Subcommand, Debug, Deserialize)]
enum Commands {
    Clone(Clone),
}

// Struct containing student attributes
#[derive(Hash, Eq, PartialEq, Deserialize, Debug)]
struct Student {
    name: String,
    github_username: String
}

// Aggregated program settings
#[derive(Deserialize, Debug)]
struct Settings {
    // Settings input via the cli
    user_input: Input,
    // Settings from toml config
    config_file: Config
}

// Struct that contains sections of data in the TOML config
#[derive(Deserialize, Debug)]
struct TomlData {
    // Config section of TOML
    config: Config,
}

// Struct that contains information in Config section of TOML file
#[derive(Deserialize, Debug)]
struct Config {
    roster_path: String
}

#[derive(Parser, Deserialize, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Input {
    #[command(subcommand)]
    command: Commands
}

#[derive(Args, Debug, Deserialize)]
struct Clone {
    // GitHub classroom org
    org: String,
    // GitHub classroom assignment name
    assignment_name: String,
    // Where git should clone to
    location: String
}

fn read_roster(roster_path: &str) -> Result<HashSet<Student>, csv::Error> {
    let mut csv_reader = csv::Reader::from_path(Path::new(&roster_path))?;
    let mut students = HashSet::new();
    for result in csv_reader.deserialize() {
        let record: Student = result?;
        students.insert(record);
    }
    Ok(students)
}

fn create_output_directory(assignment_name: &String, output_path: &String) -> Result<String, Box<dyn Error>> {
    let dt = chrono::offset::Local::now();
    //   Python local_path = os.path.join(directory_path, name)
    let output_path = format!("{}//{}D{}T{}", output_path, assignment_name,
                              dt.date_naive().to_string(), dt.time().to_string());
    fs::create_dir(&output_path)?;
    Ok(output_path)
}

fn clone_repo(org_name: &String, assignment_name: &String, output_path: &String, student: Student) {
    let username = student.github_username;
    let remote_repo_url = format!("{GITHUB_URL}:{org_name}/{assignment_name}-{username}.git");
    let mut command = Command::new("git");
    command.current_dir(output_path)
        .args(["clone", &remote_repo_url])
        .stdout(Stdio::null());
    if let Ok(output) = command.output() {
        //TODO make output better
        println!("Child id {:?}", output);
        return;
    }
    else {
        println!("Error cloning {:?}", student.name);
        return;
    }
}

fn main() {
    let settings = Settings {
        user_input: Input::parse(),
        config_file: {
            let conf_str = fs::read_to_string(DEFAULT_CONFIG_PATH).unwrap();
            let toml_data: TomlData = toml::from_str(&conf_str).unwrap_or_else(|_| {
                println!("GSI: Config file could not be read. File most likely does not exist or GSI \
                does not have adequate permissions. Using default configuration...");
                TomlData {
                    config: Config {
                        roster_path: DEFAULT_ROSTER_FILENAME.to_string()
                    }
                }
            });
            println!("{:?}", &toml_data.config);
            toml_data.config
        },
    };
    match &settings.user_input.command {
        Commands::Clone(cmd) => {
            let students = read_roster(&settings.config_file.roster_path).unwrap();
            let path = create_output_directory(&cmd.assignment_name, &cmd.location).unwrap();
            for student in students {
                clone_repo(&cmd.org, &cmd.assignment_name, &path, student)
            }
        }
    }
}