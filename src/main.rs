#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate cached;

mod frames;
mod fuse_video;
mod nodes;

use fuse_video::VideoFileSystem;
use std::ffi::OsStr;

fn main() {
    // let mountpoint = env::args_os().nth(1).unwrap();
    let mountpoint = "/tmp/mountpoint";
    let options = ["-o", "ro", "-o", "fsname=hello"]
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
