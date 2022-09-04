use extism_pdk::*;
use pulldown_cmark::{html, Options, Parser};
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

    // only handle .md files, ignore all others
    if input.event_file_name.ends_with(".md") {
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
    let out = EventOutput {
        op: String::from("create"),
        output_file_name: md_file_name,

        output_file_data: base64::encode(html_output),
    };
    host.output(&serde_json::to_string(&out).expect("output data to json"));

    return 0;
}
