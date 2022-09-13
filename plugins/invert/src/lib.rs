use extism_pdk::*;
use ril::Image;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;

// Data provided to the Plug-in from the Host, deserialized from input bytes
#[derive(Deserialize)]
struct EventInput {
    pub event_file_name: String,
    pub event_file_data: String,
}

// Data returned from the Plug-in to the Host, serialized to output bytes
#[derive(Serialize)]
struct EventOutput {
    pub op: String,
    pub output_file_name: String,
    pub output_file_data: String,
}

#[no_mangle]
pub extern "C" fn should_handle_file() -> i32 {
    let host = Host::new();
    let file_name = host.input_str();

    // only handle .png files, ignore all others
    if file_name.ends_with(".png") {
        return 0;
    }

    return 1;
}

#[no_mangle]
pub extern "C" fn on_file_write() -> i32 {
    let host = Host::new();
    let file_data = host.input();
    let input = from_slice::<EventInput>(file_data).expect("json from host");
    let bytes = base64::decode(input.event_file_data).expect("decode png");

    let mut image: Image<ril::pixel::Rgba> =
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
    let bytes = serde_json::to_vec(&out).expect("json output to host");

    // enable allocation from host to be transferred to the caller directly avoiding a copy
    let output = host.alloc_bytes(&bytes);
    host.output_memory(&output);

    return 0;
}
