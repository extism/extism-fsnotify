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

#[function]
pub fn should_handle_file(file_name: String) -> PluginResult<WithStatus<()>> {
    // only handle .md files, ignore all others
    if file_name.ends_with(".md") {
        return Ok(WithStatus::new((), 0));
    }

    info!("plugin ignoring file: {file_name}");
    Ok(WithStatus::new((), 1))
}

#[function]
pub fn on_file_write(Json(input): Json<EventInput>) -> PluginResult<Json<EventOutput>> {
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
    info!("md2html output create new file: {}", &md_file_name);

    let out = EventOutput {
        op: String::from("create"),
        output_file_name: md_file_name,
        output_file_data: base64::encode(html_output),
    };

    Ok(Json(out))
}
