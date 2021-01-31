#[macro_use]
extern crate lazy_static;
extern crate cached;
extern crate log;
extern crate serde;
extern crate serde_yaml;

mod cli;
mod fuse_fs;
mod video_processing;

use crate::cli::{parse_configuration, Configuration};
use crate::fuse_fs::create_video_filesystem;
use crate::fuse_fs::fs::VideoFileSystem;
use log::{debug, error, info};
use std::ffi::OsStr;
use std::fs::{create_dir_all, File};
use std::path::Path;
use std::process::exit;

use daemonize::Daemonize;
use std::ops::Deref;
use std::{env, process};

#[repr(i32)]
enum StatusCode {
    InvalidVideoLocation = 10,
}

fn main() {
    let configuration = initialise();
    let filesystem = create_video_filesystem(&configuration.video_location);
    if !configuration.foreground {
        daemonize(
            configuration.log_location.as_ref().map(Deref::deref),
            configuration.log_location.as_ref().map(Deref::deref),
        );
    }
    mount_filesystem(filesystem, &configuration);
}

pub fn initialise() -> Configuration {
    env_logger::init();
    let configuration = parse_configuration();
    debug!("{:?}", configuration);
    validate_configuration(&configuration);
    return configuration;
}

pub fn mount_filesystem(filesystem: VideoFileSystem, configuration: &Configuration) {
    let options = ["-o", "fsname=video-fuse-system"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();

    create_dir_all(&configuration.fuse_mount_location).expect(&format!(
        "Could not create fuse mount location: {}",
        &configuration.fuse_mount_location
    ));

    match fuse::mount(filesystem, &configuration.fuse_mount_location, &options) {
        Ok(_) => {}
        Err(e) => {
            let error_string = e.to_string();
            error!(
                "Could not mount filesystem: {} (additional error messages may be printed above)",
                error_string
            );
        }
    };
}

pub fn daemonize(stdout_location: Option<&str>, stderr_location: Option<&str>) {
    debug!("Daemonizing...");
    let mut daemon = Daemonize::new().working_directory(env::current_dir().unwrap());

    if stdout_location.is_some() {
        daemon = daemon.stdout(File::create(stdout_location.unwrap()).unwrap());
    }
    if stderr_location.is_some() {
        daemon = daemon.stderr(File::create(stderr_location.unwrap()).unwrap());
    }

    // Bypass OSX fork safety check to stop this from crashing on OSX
    // https://blog.phusion.nl/2017/10/13/why-ruby-app-servers-break-on-macos-high-sierra-and-what-can-be-done-about-it/
    env::set_var("OBJC_DISABLE_INITIALIZE_FORK_SAFETY", "YES");

    match daemon.start() {
        Ok(_) => {
            info!("Successfully daemonized (pid={})", process::id())
        }
        Err(e) => panic!(e),
    }
}

fn validate_configuration(configuration: &Configuration) {
    if !Path::new(configuration.video_location.as_str()).exists() {
        error!(
            "Video location does not exist: {}",
            configuration.video_location
        );
        exit(StatusCode::InvalidVideoLocation as i32);
    }
}
