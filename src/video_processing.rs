use cached::proc_macro::cached;
use log::{info, warn};
use opencv::core::{Mat, MatTraitConst, Vector};
use opencv::imgcodecs::imencode;
use opencv::imgproc::{cvt_color, threshold, THRESH_BINARY, THRESH_OTSU};
use opencv::prelude::VectorToVec;
use opencv::prelude::{VideoCaptureTrait, VideoCaptureTraitConst};
use opencv::videoio::{VideoCapture, CAP_PROP_FRAME_COUNT, CAP_PROP_POS_FRAMES};
use opencv::{imgproc, Error};
use strum_macros::{Display, EnumIter};

#[derive(EnumIter, Clone, Copy, Display, PartialEq, Eq, Hash)]
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

pub fn get_number_of_frames(video_location: &str) -> u64 {
    let video_capture = open_video(video_location);
    let number_of_frames = video_capture.get(CAP_PROP_FRAME_COUNT).expect(&format!(
        "Error getting number of frames for video: {}",
        video_location
    )) as u64;
    close_video(video_capture);
    number_of_frames
}

// Note: the "cached" library does not offer a cache store that is able to be resized dynamically.
//       If a cached store becomes available, `name=` can be set or the name of the function in caps
//       can be used to refer to the cache.
#[cached(size = 25)]
pub(crate) fn get_frame_image(
    video_location: String,
    frame_number: u64,
    image_format: ImageType,
) -> Vec<u8> {
    let mut frame = get_frame_from_video(video_location, frame_number);
    frame_matrix_to_vec(&mut frame, image_format)
}

#[cached(size = 25)]
pub fn get_greyscale_frame_image(
    video_location: String,
    frame_number: u64,
    image_format: ImageType,
) -> Vec<u8> {
    let frame = get_frame_from_video(video_location.clone(), frame_number);
    let mut greyscale_frame = frame_to_greyscale(&frame).expect(&format!(
        "Could not create greyscale copy of frame number {} in video: {}",
        frame_number, video_location
    ));
    frame_matrix_to_vec(&mut greyscale_frame, image_format)
}

#[cached(size = 25)]
pub fn get_black_and_white_frame_image(
    video_location: String,
    frame_number: u64,
    threshold_at: Option<u8>,
    image_format: ImageType,
) -> Vec<u8> {
    info!(
        "Producing black and white {} image of frame {} from video \"{}\", thresholding at {:?}",
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
    let mut black_and_white_frame = frame_to_black_and_white(
        &greyscale_frame,
        match threshold_at {
            None => None,
            Some(x) => Some(x as f64),
        },
    )
    .expect("Could not convert to black and white");
    frame_matrix_to_vec(&mut black_and_white_frame, image_format)
}

pub fn frame_matrix_to_vec(frame: &mut Mat, convert_to: ImageType) -> Vec<u8> {
    let parameters = &Default::default();
    let buffer = &mut Vector::<u8>::new();

    if frame.empty() {
        warn!("Empty frame found");
        return Vec::new();
    }
    imencode(
        &format!(".{}", convert_to.to_string()),
        frame,
        buffer,
        parameters,
    )
    .expect(&format!(
        "Could not encode image into: {}",
        convert_to.to_string()
    ));
    buffer.to_vec()
}

fn open_video(file_name: &str) -> VideoCapture {
    VideoCapture::from_file(file_name, 0).expect(&format!(
        "Error creating VideoCapture from file: {}",
        file_name
    ))
}

fn close_video(mut video_capture: VideoCapture) {
    video_capture.release().expect("Error closing VideoCapture");
}

#[cached(size = 25)]
fn get_frame_from_video(video_location: String, frame_number: u64) -> Mat {
    let mut video_capture = open_video(&video_location);
    let frame = get_frame(&mut video_capture, frame_number);
    close_video(video_capture);
    frame
}

fn get_frame(video_capture: &mut VideoCapture, frame_number: u64) -> Mat {
    video_capture
        .set(CAP_PROP_POS_FRAMES, frame_number as f64)
        .unwrap();
    get_next_frame(video_capture)
}

fn get_next_frame(video_capture: &mut VideoCapture) -> Mat {
    let mut frame = opencv::core::Mat::default();
    video_capture.read(&mut frame).unwrap();
    frame
}

fn frame_to_greyscale(frame: &Mat) -> Result<Mat, Error> {
    let mut greyscale_frame = Mat::default();
    match cvt_color(frame, &mut greyscale_frame, imgproc::COLOR_BGR2GRAY, 0) {
        Ok(_) => Ok(greyscale_frame),
        Err(e) => Err(e),
    }
}

fn frame_to_black_and_white(frame: &Mat, threshold_at: Option<f64>) -> Result<Mat, Error> {
    let mut thresholding_type = THRESH_BINARY;
    let threshold_at = match threshold_at {
        None => {
            thresholding_type |= THRESH_OTSU;
            0.0
        }
        Some(x) => x,
    };
    let mut black_and_white_frame = Mat::default();
    match threshold(
        frame,
        &mut black_and_white_frame,
        threshold_at,
        255.0,
        thresholding_type,
    ) {
        Ok(x) => {
            info!("Used threshold: {}", x);
            Ok(black_and_white_frame)
        }
        Err(e) => Err(e),
    }
}
