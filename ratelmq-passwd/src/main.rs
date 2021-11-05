use clap::{App, Arg, ArgMatches};
use regex::RegexBuilder;

use ratelmq::config::build_info::BUILD_INFO;

const ARGUMENT_NAME_FILE: &str = "file";
const ARGUMENT_NAME_USER: &str = "user";
const ARGUMENT_NAME_PASSWORD: &str = "password";
const ARGUMENT_NAME_DELETE: &str = "delete";

fn main() {
    env_logger::init();

    let version = format!("v{}({})", BUILD_INFO.version, &BUILD_INFO.commit_hash[..10]);

    let arguments = build_arguments(version);

    let file_name = arguments.value_of(ARGUMENT_NAME_FILE).unwrap();
    let user_name = arguments.value_of(ARGUMENT_NAME_USER).unwrap();
    let password = arguments.value_of(ARGUMENT_NAME_PASSWORD);
    let delete = arguments.is_present(ARGUMENT_NAME_DELETE);

    let regexp_format = format!(r"{}{}", regex::escape(user_name), r":.*\n");
    let regexp = RegexBuilder::new(regexp_format.as_str())
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .unwrap();

    let credentials = std::fs::read_to_string(file_name).unwrap();

    let entry_exists = regexp.is_match(credentials.as_str());
    let updated_credentials = if entry_exists {
        let text = match delete {
            true => "".to_string(),
            false => build_entry(user_name, password),
        };

        Some(
            regexp
                .replace(credentials.as_str(), text.as_str())
                .into_owned(),
        )
    } else if !delete {
        Some(credentials + build_entry(user_name, password).as_str())
    } else {
        None
    };

    if let Some(new_file_content) = updated_credentials {
        std::fs::write(file_name, new_file_content).unwrap();
    }
}

fn build_arguments(version: String) -> ArgMatches {
    App::new("ratelmq-passwd")
        .version(version.as_str())
        .about("Tool for RatelMQ password file management.")
        .arg(
            Arg::new(ARGUMENT_NAME_FILE)
                .short('f')
                .long(ARGUMENT_NAME_FILE)
                .value_name("PASSWD_FILE")
                .about("RatelMQ passwd file to modify")
                .default_value("/etc/ratelmq/passwd")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::new(ARGUMENT_NAME_USER)
                .short('u')
                .long(ARGUMENT_NAME_USER)
                .value_name("USER_NAME")
                .about("User name to modify")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::new(ARGUMENT_NAME_PASSWORD)
                .short('p')
                .long(ARGUMENT_NAME_PASSWORD)
                .value_name("PASSWORD")
                .about("Password to be set")
                .required_unless_present(ARGUMENT_NAME_DELETE)
                .conflicts_with(ARGUMENT_NAME_DELETE)
                .takes_value(true),
        )
        .arg(
            Arg::new(ARGUMENT_NAME_DELETE)
                .short('d')
                .long(ARGUMENT_NAME_DELETE)
                .about("Delete the entry if exists")
                .required(false)
                .takes_value(false),
        )
        .get_matches()
}

fn build_entry(user_name: &str, password: Option<&str>) -> String {
    format!("{}:{}\n", user_name, password.unwrap())
}
