use clap::{App, Arg};
use dotenv::dotenv;

use ratelmq::config::build_info::BUILD_INFO;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let version = format!("v{}({})", BUILD_INFO.version, &BUILD_INFO.commit_hash[..10]);

    let argument_name_config = "config";

    let arguments = App::new("RatelMQ")
        .version(version.as_str())
        .about("Efficient, reliable & scalable MQTT broker.")
        .arg(
            Arg::new(argument_name_config)
                .short('c')
                .long(argument_name_config)
                .value_name("FILE")
                .default_value("/etc/ratelmq/ratelmq.conf")
                .about("Sets a configuration file")
                .required(false)
                .takes_value(true),
        )
        .get_matches();

    let config_filename = arguments.value_of(argument_name_config).unwrap();
    ratelmq::run(config_filename).await;
}
