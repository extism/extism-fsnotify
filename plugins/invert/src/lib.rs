use extism_pdk::*;
use ril::Image;
use serde::Deserialize;
use serde_json::from_slice;

#[derive(Deserialize)]
struct EventInput {
    pub event_file_name: String,
    pub event_file_data: String,
}

#[no_mangle]
pub extern "C" fn on_file_write() -> i32 {
    let host = Host::new();
    let file_data = host.input();
    let input = from_slice::<EventInput>(file_data).expect("json from host");

    // for now expect bytes to be in a png encoding
    if !input.event_file_name.ends_with(".png") {
        return 1;
    }

    let bytes = base64::decode(input.event_file_data).expect("decode png");
    let mut image: Image<ril::pixel::Rgb> =
        Image::from_bytes(ril::ImageFormat::Png, bytes).expect("parse png");

    image.invert();

    let mut dest = vec![];
    image
        .encode(ril::ImageFormat::Png, &mut dest)
        .expect("encode png");

    // write the bytes back to the host to be saved as the original file
    host.output(&dest);

    return 0;
}
