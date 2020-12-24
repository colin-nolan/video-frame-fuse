use opencv::core::{Mat, Vector};
use opencv::imgcodecs::{imencode, imwrite};
use opencv::prelude::VideoCaptureTrait;
use opencv::videoio::{VideoCapture, CAP_PROP_FRAME_COUNT, CAP_PROP_POS_FRAMES};
use std::env;

pub(crate) fn main() {
    let file_name = &env::args().nth(1).unwrap();
    let mut video_capture = open_video(file_name);

    eprintln!("Number of frames: {}", get_number_of_frames(&video_capture));

    let frame_number = 0;
    while video_capture.is_opened().unwrap() {
        get_frame(frame_number, &mut video_capture);
    }

    video_capture.release().unwrap();

    eprintln!("Complete");
}

pub fn get_number_of_frames(video_capture: &VideoCapture) -> i64 {
    video_capture.get(CAP_PROP_FRAME_COUNT).unwrap() as i64
}

pub fn open_video(file_name: &str) -> VideoCapture {
    VideoCapture::from_file(file_name, 0).unwrap()
}

pub fn close_video(mut video_capture: VideoCapture) {
    video_capture.release().unwrap();
}

pub fn get_frame(frame_number: i64, video_capture: &mut VideoCapture) -> Mat {
    video_capture
        .set(CAP_PROP_POS_FRAMES, frame_number as f64)
        .unwrap();
    return get_next_frame(video_capture);
}

pub fn get_next_frame(video_capture: &mut VideoCapture) -> Mat {
    let mut frame = opencv::core::Mat::default().unwrap();
    video_capture.read(&mut frame).unwrap();
    return frame;
}

pub fn save_frame(frame: &Mat, filename: &str) {
    let parameters = &Default::default();
    imwrite(filename, &mut frame.clone(), parameters).unwrap();
}

pub fn frame_to_jpg(frame: &Mat, buffer: &mut Vector<u8>) {
    let parameters = &Default::default();
    // TODO: How expensive is this clone?
    imencode(".jpg", &mut frame.clone(), buffer, parameters).unwrap();
    // return buffer.clone();
    // return buffer.to_owned();
}
