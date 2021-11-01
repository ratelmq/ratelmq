use std::fs::OpenOptions;
use std::io::Write;

use clap::{App, Arg};

use ratelmq::config::build_info::BUILD_INFO;

fn main() {
    env_logger::init();

    let version = format!("v{}({})", BUILD_INFO.version, &BUILD_INFO.commit_hash[..10]);

    const ARGUMENT_NAME_FILE: &str = "file";
    const ARGUMENT_NAME_USER: &str = "user";
    const ARGUMENT_NAME_PASSWORD: &str = "password";

    let arguments = App::new("ratelmq-passwd")
        .version(version.as_str())
        .about("Tool for RatelMQ password file management.")
        .arg(
            Arg::new(ARGUMENT_NAME_FILE)
                .short('f')
                .long(ARGUMENT_NAME_FILE)
                .value_name("PASSWD_FILE")
                .about("RatelMQ passwd file to modify")
                .default_value("/etc/ratelmq/passwd")
                .required(true)
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
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let file_name = arguments.value_of(ARGUMENT_NAME_FILE).unwrap();
    let user_name = arguments.value_of(ARGUMENT_NAME_USER).unwrap();
    let password = arguments.value_of(ARGUMENT_NAME_PASSWORD).unwrap();

    let file_entry = format!("{}:{}", user_name, password);

    let mut file = OpenOptions::new().append(true).open(file_name).unwrap();
    writeln!(&mut file, "{}", file_entry.as_str()).unwrap();
}
