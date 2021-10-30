use clap::{App, Arg};
use indoc::indoc;

const LOG_LOCATION: &str = "logfile";
const FOREGROUND_PARAMETER: &str = "foreground";
const VIDEO_LOCATION_PARAMETER: &str = "video-location";
const FUSE_MOUNT_LOCATION_PARAMETER: &str = "fuse-mount-location";

#[derive(Debug)]
pub struct Configuration {
    pub log_location: Option<String>,
    pub foreground: bool,
    pub video_location: String,
    pub fuse_mount_location: String,
}

pub fn parse_configuration() -> Configuration {
    let matches = App::new("Video Frame FUSE")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Colin Nolan <cn580@alumni.york.ac.uk>")
        // TODO
        // .about("Does awesome things")
        .arg(
            Arg::with_name(LOG_LOCATION)
                .long(&format!("--{}", LOG_LOCATION))
                .required(false)
                .takes_value(true)
                .conflicts_with(FOREGROUND_PARAMETER)
                .help("write logs to this location when demonized (not in foreground)"),
        )
        .arg(
            Arg::with_name(FOREGROUND_PARAMETER)
                .long(&format!("--{}", FOREGROUND_PARAMETER))
                .required(false)
                .help("run in foreground (default is to daemonize)"),
        )
        .arg(
            Arg::with_name(VIDEO_LOCATION_PARAMETER)
                .help("location of the video file to use")
                .required(true),
        )
        .arg(
            Arg::with_name(FUSE_MOUNT_LOCATION_PARAMETER)
                .help("location of directory to mount fuse (will create if does not exist)")
                .required(true),
        )
        .after_help(indoc! {"
            Setting RUST_LOG to one of {error, warn, info debug, trace} will set the logging \
            verbosity, e.g. RUST_LOG=info
        "})
        .get_matches();

    Configuration {
        log_location: matches.value_of(LOG_LOCATION).map(str::to_string),
        foreground: matches.is_present(FOREGROUND_PARAMETER),
        video_location: matches
            .value_of(VIDEO_LOCATION_PARAMETER)
            .unwrap()
            .to_string(),
        fuse_mount_location: matches
            .value_of(FUSE_MOUNT_LOCATION_PARAMETER)
            .unwrap()
            .to_string(),
    }
}
