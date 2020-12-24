use crate::frames;
use crate::frames::{frame_to_jpg, get_number_of_frames, open_video};
use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc::ENOENT;
use opencv::core::MatTrait;
use opencv::videoio::VideoCapture;
use std::ffi::OsStr;
use std::time::{Duration, SystemTime};

const TTL: Duration = Duration::from_secs(1); // 1 second

const HELLO_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 3,
    blocks: 0,
    atime: SystemTime::UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: SystemTime::UNIX_EPOCH,
    ctime: SystemTime::UNIX_EPOCH,
    crtime: SystemTime::UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

const HELLO_TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 1089132,
    blocks: 1,
    atime: SystemTime::UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: SystemTime::UNIX_EPOCH,
    ctime: SystemTime::UNIX_EPOCH,
    crtime: SystemTime::UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

struct HelloFS {
    video: VideoCapture,
}

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == 1 && name.to_str() == Some("frame0.jpg") {
            reply.entry(&TTL, &HELLO_TXT_ATTR, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &HELLO_DIR_ATTR),
            2 => reply.attr(&TTL, &HELLO_TXT_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        if ino == 2 {
            let frame = frames::get_frame(0, &mut self.video);

            let data = &mut Default::default();
            frame_to_jpg(&frame, data);

            eprintln!("number_frames: {}", get_number_of_frames(&self.video));
            eprintln!("frame_data.len: {}", data.len());
            eprintln!("data.len: {}", data.len());
            eprintln!("offset: {}", offset);
            eprintln!("size: {}", size);

            // let custom_bytes = [155, 255, 87];
            // reply.data(&custom_bytes[offset as usize..]);

            reply.data(&data.as_slice()[offset as usize..(offset as usize + size as usize)]);

            eprintln!("Data replied")
        } else {
            eprintln!("error ENOENT");
            reply.error(ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let entries = vec![
            (1, FileType::Directory, "."),
            (1, FileType::Directory, ".."),
            (2, FileType::RegularFile, "frame0.jpg"),
        ];

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
        }
        reply.ok();
    }
}

pub(crate) fn main() {
    // let mountpoint = env::args_os().nth(1).unwrap();
    let mountpoint = "/tmp/mountpoint2";
    let options = ["-o", "ro", "-o", "fsname=hello"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(
        HelloFS {
            video: open_video("/Users/colin/Movies/crf0/ultrafast.mp4"),
        },
        &mountpoint,
        &options,
    )
    .unwrap();
}
