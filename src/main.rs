use opencv::core::{Mat, ToOutputArray, Vector};
use opencv::imgcodecs::imwrite;
use opencv::prelude::VideoCaptureTrait;
use opencv::videoio::VideoCapture;
use std::borrow::Borrow;
use std::env;
use std::ops::Deref;
use std::process::exit;

fn main() {
    let mut video_capture = VideoCapture::from_file(&env::args().nth(1).unwrap(), 0).unwrap();

    // while video_capture.is_opened().unwrap() {
    let mut frame = opencv::core::Mat::default().unwrap();
    video_capture.read(&mut frame);
    let parameters = &Default::default();
    imwrite("test.jpg", &mut frame, parameters);

    video_capture.release();
    println!("Complete");

    exit(0);
    // }

    video_capture.release();

    println!("Complete");
}
