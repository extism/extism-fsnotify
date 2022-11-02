use extism_pdk::*;
use ril::Image;
use serde::{Deserialize, Serialize};

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

#[function]
pub fn should_handle_file(file_name: String) -> PluginResult<WithStatus<()>> {
    // only handle .png files, ignore all others
    if file_name.ends_with(".png") {
        return Ok(WithStatus::new((), 0));
    }

    Ok(WithStatus::new((), 1))
}

#[function]
pub fn on_file_write(Json(input): Json<EventInput>) -> PluginResult<Json<EventOutput>> {
    let bytes = base64::decode(input.event_file_data)?;

    let mut image: Image<ril::pixel::Rgba> =
        Image::from_bytes(ril::ImageFormat::Png, bytes).expect("image decode");
    image.invert();

    let mut dest = vec![];
    image
        .encode(ril::ImageFormat::Png, &mut dest)
        .expect("image encode");

    // write the bytes back to the host to be saved as the original file
    let out = EventOutput {
        op: String::from("overwrite"),
        output_file_name: input.event_file_name,
        output_file_data: base64::encode(dest),
    };
    return Ok(Json(out));
}
