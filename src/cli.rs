use clap::{App, Arg};
use indoc::indoc;

const VIDEO_LOCATION_PARAMETER: &str = "video-location";
const FUSE_MOUNT_LOCATION_PARAMETER: &str = "fuse-mount-location";

#[derive(Debug)]
pub struct Configuration {
    pub video_location: String,
    pub fuse_mount_location: String,
}

pub fn parse_configuration() -> Configuration {
    let matches = App::new("Video Frame FUSE")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Colin Nolan <cn580@alumni.york.ac.uk>")
        // TODO
        // .about("Does awesome things")
        .after_help(indoc! {"
            Setting RUST_LOG to on of {error, warn, info debug, trace} will set the logging \
            verbosity, e.g. RUST_LOG=info
        "})
        .arg(
            Arg::with_name(VIDEO_LOCATION_PARAMETER)
                .help("location of the video file to use")
                .required(true),
        )
        .arg(
            Arg::with_name(FUSE_MOUNT_LOCATION_PARAMETER)
                .help("location of directory to mount fuse")
                .required(true),
        )
        .get_matches();

    Configuration {
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
