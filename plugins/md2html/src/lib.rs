use extism_pdk::*;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};

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

#[plugin_fn]
pub fn should_handle_file(file_name: String) -> FnResult<i32> {
    // only handle .md files, ignore all others
    if file_name.ends_with(".md") {
        return Ok(0);
    }

    log!(LogLevel::Info, "plugin ignoring file: {}", file_name);

    return Ok(1);
}

#[plugin_fn]
pub fn on_file_write(Json(input): Json<EventInput>) -> FnResult<Json<EventOutput>> {
    let bytes = base64::decode(input.event_file_data).expect("decode png");

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let md = String::from_utf8(bytes).expect("text from file");
    let parser = Parser::new_ext(&md, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let md_file_name = format!(
        "{}.html",
        input
            .event_file_name
            .strip_suffix(".md")
            .expect("filename has .md suffix")
    );

    // log to the host runtime (written to the host logfile)
    log!(
        LogLevel::Info,
        "md2html output create new file: {}",
        &md_file_name,
    );

    let out = EventOutput {
        op: String::from("create"),
        output_file_name: md_file_name,
        output_file_data: base64::encode(html_output),
    };

    Ok(Json(out))
}
