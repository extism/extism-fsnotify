use std::collections::HashMap;

use extism_pdk::*;
use ril::Image;
use serde::Deserialize;
use serde_json::from_slice;

#[derive(Deserialize)]
struct EventInput {
    pub event_file_name: String,
    pub event_file_data: String,
    pub dir_entry_files: HashMap<String, String>,
}

// impl EventInput {
//     fn target(&self) -> impl GenericImage<Pixel = Rgba<u8>> {
//         let b = base64::decode(self.event_file_data.clone()).expect("target base64 decode");
//         image::load_from_memory(&b).expect("load target from decoded b64")
//     }

//     fn watermark(&self) -> Option<impl GenericImageView<Pixel = Rgba<u8>>> {
//         if let Some((_, watermark)) = self
//             .dir_entry_files
//             .iter()
//             .find(|(name, _)| name.ends_with("/watermark.png"))
//         {
//             let b = base64::decode(watermark).expect("watermark base64 decode");
//             Some(image::load_from_memory(&b).expect("load watermark from decoded b64"))
//         } else {
//             None
//         }
//     }
// }

#[no_mangle]
pub extern "C" fn on_file_write() -> i32 {
    let host = Host::new();
    let file_data = host.input();
    let input = from_slice::<EventInput>(file_data).expect("json from host");

    if !input.event_file_name.ends_with(".png") {
        return 1;
    }

    // let mut target = input.target();
    // let watermark = input.watermark().expect("watermark.png file");

    let bytes = base64::decode(input.event_file_data).expect("decode png");
    let mut image: Image<ril::pixel::Rgb> =
        Image::from_bytes(ril::ImageFormat::Png, bytes).expect("parse png");

    // for now expect bytes to be in a png encoding
    // overlay(&mut target, &watermark, 0, 0);
    image.invert();

    let mut dest = vec![];
    image
        .encode(ril::ImageFormat::Png, &mut dest)
        .expect("encode png");

    // write the bytes
    host.output(&dest);

    return 0;
}
