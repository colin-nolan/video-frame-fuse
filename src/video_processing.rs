use std::string::ToString;

use cached::proc_macro::cached;
use log::info;
use opencv::core::{Mat, Vector};
use opencv::imgcodecs::imencode;
use opencv::imgproc::{cvt_color, threshold, THRESH_BINARY};
use opencv::prelude::VideoCaptureTrait;
use opencv::videoio::{VideoCapture, CAP_PROP_FRAME_COUNT, CAP_PROP_POS_FRAMES};
use opencv::{imgproc, Error};
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

pub fn get_number_of_frames(video_capture: &VideoCapture) -> u64 {
    video_capture.get(CAP_PROP_FRAME_COUNT).unwrap() as u64
}

pub fn open_video(file_name: &str) -> VideoCapture {
    VideoCapture::from_file(file_name, 0).unwrap()
}

pub fn close_video(mut video_capture: VideoCapture) {
    video_capture.release().unwrap();
}

// TODO: cache size
#[cached]
pub fn get_frame_from_video(video_location: String, frame_number: u64) -> Mat {
    let mut video_capture = open_video(&video_location);
    let frame = get_frame(&mut video_capture, frame_number);
    close_video(video_capture);
    return frame;
}

// TODO: cache size
#[cached]
pub fn get_frame_image(
    video_location: String,
    frame_number: u64,
    image_format: ImageType,
) -> Vec<u8> {
    let frame = get_frame_from_video(video_location, frame_number);
    return convert_frame(&frame, image_format);
}

// TODO: cache size
#[cached]
pub fn get_greyscale_frame_image(
    video_location: String,
    frame_number: u64,
    image_format: ImageType,
) -> Vec<u8> {
    let frame = get_frame_from_video(video_location.clone(), frame_number);
    let greyscale_frame = frame_to_greyscale(&frame).expect(&format!(
        "Could not create greyscale copy of frame number {} in video: {}",
        frame_number, video_location
    ));
    return convert_frame(&greyscale_frame, image_format);
}

// TODO: cache size
#[cached]
pub fn get_black_and_white_frame_image(
    video_location: String,
    frame_number: u64,
    threshold_at: u8,
    image_format: ImageType,
) -> Vec<u8> {
    info!(
        "Producing black and white {} image of frame {} from video \"{}\", thresholding at {}",
        image_format.to_string(),
        frame_number,
        video_location,
        threshold_at
    );
    let frame = get_frame_from_video(video_location.clone(), frame_number);
    let greyscale_frame = frame_to_greyscale(&frame).expect(&format!(
        "Could not create greyscale copy of frame number {} in video: {}",
        frame_number, video_location
    ));
    let black_and_white_frame = frame_to_black_and_white(&greyscale_frame, threshold_at.into())
        .expect("Could not convert to black and white");
    return convert_frame(&black_and_white_frame, image_format);
}

fn frame_to_black_and_white(frame: &Mat, threshold_at: f64) -> Result<Mat, Error> {
    let mut black_and_white_frame = Mat::default().unwrap();
    match threshold(
        frame,
        &mut black_and_white_frame,
        threshold_at,
        255.0,
        THRESH_BINARY,
    ) {
        Ok(_) => Ok(black_and_white_frame),
        Err(e) => Err(e),
    }
}

fn frame_to_greyscale(frame: &Mat) -> Result<Mat, Error> {
    let mut greyscale_frame = Mat::default().unwrap();
    match cvt_color(frame, &mut greyscale_frame, imgproc::COLOR_BGR2GRAY, 0) {
        Ok(_) => Ok(greyscale_frame),
        Err(e) => Err(e),
    }
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

fn get_frame(video_capture: &mut VideoCapture, frame_number: u64) -> Mat {
    video_capture
        .set(CAP_PROP_POS_FRAMES, frame_number as f64)
        .unwrap();
    return get_next_frame(video_capture);
}

fn get_next_frame(video_capture: &mut VideoCapture) -> Mat {
    let mut frame = opencv::core::Mat::default().unwrap();
    video_capture.read(&mut frame).unwrap();
    return frame;
}
