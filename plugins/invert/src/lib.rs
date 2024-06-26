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

#[plugin_fn]
pub fn should_handle_file(file_name: String) -> FnResult<i32> {
    // only handle .png files, ignore all others
    if file_name.ends_with(".png") {
        return Ok(0);
    }

    return Ok(1);
}

#[plugin_fn]
pub fn on_file_write(Json(input): Json<EventInput>) -> FnResult<Json<EventOutput>> {
    let bytes = base64::decode(input.event_file_data)
        .map_err(|e| WithReturnCode::new(Error::msg(e.to_string()), -1))?;

    let mut image: Image<ril::pixel::Rgba> = Image::from_bytes(ril::ImageFormat::Png, bytes)
        .map_err(|e| WithReturnCode::new(Error::msg(e.to_string()), -2))?;

    image.invert();

    let mut dest = vec![];
    image
        .encode(ril::ImageFormat::Png, &mut dest)
        .map_err(|e| WithReturnCode::new(Error::msg(e.to_string()), -3))?;

    // write the bytes back to the host to be saved as the original file
    let out = EventOutput {
        op: String::from("overwrite"),
        output_file_name: input.event_file_name,
        output_file_data: base64::encode(dest),
    };

    Ok(Json(out))
}
