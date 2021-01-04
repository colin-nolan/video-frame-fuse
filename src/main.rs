#[macro_use]
extern crate lazy_static;
extern crate cached;
extern crate log;
extern crate serde;
extern crate serde_yaml;

mod configuration;
mod file_system;
mod fuse_video;
mod video_processing;

use fuse;
use fuse_video::VideoFileSystem;
use std::ffi::OsStr;

fn main() {
    env_logger::init();

    // let mountpoint = env::args_os().nth(1).unwrap();
    let mountpoint = "/tmp/mountpoint";
    let options = ["-o", "fsname=hello"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();

    fuse::mount(
        VideoFileSystem::new("/Users/colin/Movies/crf0/ultrafast.mp4"),
        &mountpoint,
        &options,
    )
    .expect(&format!("Could not mount filesystem to: {}", mountpoint));
}
