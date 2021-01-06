use crate::fuse_fs::fs::VideoFileSystem;
use crate::fuse_fs::nodes::create_default_video_nodes;

pub(crate) mod fs;
mod models;
mod nodes;

pub fn create_video_filesystem(video_location: &str) -> VideoFileSystem {
    let nodes = create_default_video_nodes(video_location);
    VideoFileSystem { nodes }
}
