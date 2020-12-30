use crate::nodes::FileFuseNode;
use cached::proc_macro::cached;
use opencv::core::{Mat, Vector};
use opencv::imgcodecs::{imencode, imwrite};
use opencv::prelude::VideoCaptureTrait;
use opencv::videoio::{VideoCapture, CAP_PROP_FRAME_COUNT, CAP_PROP_POS_FRAMES};
use std::string::ToString;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, strum_macros::ToString, PartialEq, Eq, Hash)]
pub enum ImageType {
    #[strum(serialize = "jpg")]
    JPG,
    #[strum(serialize = "png")]
    PNG,
    #[strum(serialize = "bmp")]
    BMP,
    #[strum(serialize = "webp")]
    WEBP,
}

// TODO: move to appropriate module
// #[derive(Clone)]
pub struct FileInformation {
    pub name: String,
    pub data_fetcher: Option<Box<dyn Fn() -> Vec<u8>>>,
    pub data: Option<Vec<u8>>,
    pub initially_listed: bool,
    pub executable: bool,
}

impl FileInformation {
    pub fn new(
        name: &str,
        data_fetcher: Box<dyn Fn() -> Vec<u8>>,
        listed: bool,
        executable: bool,
    ) -> Self {
        FileInformation {
            name: name.to_string(),
            data_fetcher: Some(data_fetcher),
            data: None,
            initially_listed: listed,
            executable,
        }
    }

    pub fn new_with_data(name: &str, data: Vec<u8>, listed: bool, executable: bool) -> Self {
        FileInformation {
            name: name.to_string(),
            data_fetcher: None,
            data: Some(data),
            initially_listed: listed,
            executable,
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        match self.data_fetcher.is_some() {
            true => (self.data_fetcher.as_ref().unwrap())(),
            false => self.data.as_ref().unwrap().clone(),
        }
    }
}

pub fn get_number_of_frames(video_capture: &VideoCapture) -> u64 {
    video_capture.get(CAP_PROP_FRAME_COUNT).unwrap() as u64
}

pub fn open_video(file_name: &str) -> VideoCapture {
    VideoCapture::from_file(file_name, 0).unwrap()
}

pub fn close_video(mut video_capture: VideoCapture) {
    video_capture.release().unwrap();
}

pub fn get_frame(video_capture: &mut VideoCapture, frame_number: u64) -> Mat {
    video_capture
        .set(CAP_PROP_POS_FRAMES, frame_number as f64)
        .unwrap();
    return get_next_frame(video_capture);
}

pub fn save_frame(frame: &Mat, filename: &str) {
    let parameters = &Default::default();
    imwrite(filename, &mut frame.clone(), parameters).unwrap();
}

// TODO: cache size
#[cached]
pub fn get_frame_from_video(video_location: &'static str, frame_number: u64) -> Mat {
    let mut video_capture = open_video(video_location);
    let frame = get_frame(&mut video_capture, frame_number);
    close_video(video_capture);
    return frame;
}

// TODO: cache size
#[cached]
pub fn read_frame(
    video_location: &'static str,
    frame_number: u64,
    image_format: ImageType,
) -> Vec<u8> {
    let frame = get_frame_from_video(video_location, frame_number);
    return convert_frame(&frame, image_format);
}

pub fn convert_frame(frame: &Mat, convert_to: ImageType) -> Vec<u8> {
    let parameters = &Default::default();
    let buffer = &mut Vector::<u8>::new();

    // TODO: How expensive is this clone?
    imencode(
        &format!(".{}", convert_to.to_string()),
        &mut frame.clone(),
        buffer,
        parameters,
    )
    .unwrap();
    return buffer.to_vec();
}

fn get_next_frame(video_capture: &mut VideoCapture) -> Mat {
    let mut frame = opencv::core::Mat::default().unwrap();
    video_capture.read(&mut frame).unwrap();
    return frame;
}
