use extism_pdk::*;
use ril::Image;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;

#[derive(Deserialize)]
struct EventInput {
    pub event_file_name: String,
    pub event_file_data: String,
}

#[derive(Serialize)]
struct EventOutput {
    pub op: String,
    pub output_file_name: String,
    pub output_file_data: String,
}

#[no_mangle]
pub extern "C" fn handle_file() -> i32 {
    let host = Host::new();
    let file_data = host.input();
    let input = from_slice::<EventInput>(file_data).expect("json from host");

    // only handle .png files, ignore all others
    if input.event_file_name.ends_with(".png") {
        host.output(&[0]);
        return 0;
    }

    host.output(&[]);
    return 1;
}

#[no_mangle]
pub extern "C" fn on_file_write() -> i32 {
    let host = Host::new();
    let file_data = host.input();
    let input = from_slice::<EventInput>(file_data).expect("json from host");
    let bytes = base64::decode(input.event_file_data).expect("decode png");

    let mut image: Image<ril::pixel::Rgb> =
        Image::from_bytes(ril::ImageFormat::Png, bytes).expect("parse png");

    image.invert();

    let mut dest = vec![];
    image
        .encode(ril::ImageFormat::Png, &mut dest)
        .expect("encode png");

    // write the bytes back to the host to be saved as the original file
    let out = EventOutput {
        op: String::from("overwrite"),
        output_file_name: input.event_file_name,
        output_file_data: base64::encode(dest),
    };
    host.output(&serde_json::to_string(&out).expect("json output to host"));

    return 0;
}
