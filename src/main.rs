use std::fs;
use std::collections::HashSet;
use std::error::Error;
use std::path::Path;
use std::process::{Command, Stdio};
use std::string::String;
use clap::Parser;
use serde::Deserialize;
use csv;

const GITHUB_URL: &str = "git@github.com";
const ROSTER_FILENAME: &str = "src/test.csv";
const DIRECTORY_NAME: &str = "classroom_repos_";

#[derive(Hash, Eq, PartialEq, Deserialize, Debug)]
struct Student {
    name: String,
    github_username: String
}

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Input {
    /// The pattern to look for
    org: String,
    /// The path to the file to read
    assignment_name: String,
    // Path to desired output location
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

fn create_output_directory(org_name: &String, assignment_name: &String, output_path: &String) -> Result<(String), Box<dyn Error>> {
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
    command.current_dir(output_path);
    if let Ok(output) = command.args(["clone", &remote_repo_url]).stdout(Stdio::null()).output() {
        println!("Child id {:?}", output);
        return;
    }
    else {
        println!("Error cloning {:?}", student.name);
        return;
    }
}

fn main() {
    let args = Input::parse();
    let students = read_roster(ROSTER_FILENAME).unwrap();
    let path = create_output_directory(&args.org, &args.assignment_name, &args.location).unwrap();
    for student in students {
        clone_repo(&args.org, &args.assignment_name, &path, student)
    }
}