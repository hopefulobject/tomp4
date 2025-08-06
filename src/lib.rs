#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use openh264::encoder::Encoder;
use openh264::formats::YUVBuffer;

#[napi]
pub fn plus_100(input: u32) -> u32 {
  input + 100
}

#[napi]
pub fn yuv_to_h264(buf: Buffer, width: u32, height: u32) -> Buffer {
  let buf: Vec<u8> = buf.into();

  // OpenH264 has minimum frame size requirements
  // Most H.264 encoders require at least 16x16 for proper macroblock alignment
  if width < 16 || height < 16 {
    panic!("Frame size {width}x{height} is too small. OpenH264 requires minimum 16x16 frame size for proper H.264 encoding");
  }

  // Use default encoder configuration
  let mut encoder = Encoder::new().unwrap();

  let yuv_data: &Vec<u8> = buf.as_ref();

  // For YUV420 format: Y plane (full size) + U plane (1/4 size) + V plane (1/4 size)
  let y_size = (width * height) as usize;
  let uv_size = (width * height / 4) as usize;
  let expected_size = y_size + uv_size + uv_size;

  // Validate input buffer size
  if yuv_data.len() != expected_size {
    panic!(
      "Invalid buffer size. Expected {} bytes for {}x{} YUV420 frame, got {} bytes",
      expected_size,
      width,
      height,
      yuv_data.len()
    );
  }

  let y_plane = &yuv_data[0..y_size];
  let u_plane = &yuv_data[y_size..y_size + uv_size];
  let v_plane = &yuv_data[y_size + uv_size..y_size + 2 * uv_size];

  let mut yuv_data_vec = Vec::with_capacity(expected_size);
  yuv_data_vec.extend_from_slice(y_plane);
  yuv_data_vec.extend_from_slice(u_plane);
  yuv_data_vec.extend_from_slice(v_plane);

  let yuv_buffer = YUVBuffer::from_vec(yuv_data_vec, width as usize, height as usize);

  // Add error handling for the encode call
  let bitstream = match encoder.encode(&yuv_buffer) {
    Ok(bs) => bs,
    Err(e) => panic!("Failed to encode YUV to H264: {e:?}"),
  };

  Buffer::from(bitstream.to_vec())
}
