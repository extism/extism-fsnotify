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
pub extern "C" fn should_handle_file() -> i32 {
    // access the host for input data, to which plugin has exclusive access
    let host = Host::new();
    // load the "input" from the caller
    let file_name = host.input_str();

    // only handle .md files, ignore all others
    if file_name.ends_with(".md") {
        return 0;
    }

    host.log(
        LogLevel::Info,
        &format!("plugin ignoring file: {}", file_name),
    );
    return 1;
}

#[no_mangle]
pub extern "C" fn on_file_write() -> i32 {
    let host = Host::new();
    let file_data = host.input();

    // input is raw bytes, but host should make schema/encoding known to plug-in author,
    // here we use json in the form of the `EventInput` type
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

    // log to the host runtime (written to the host logfile)
    host.log(
        LogLevel::Info,
        &format!("md2html output create new file: {}", &md_file_name),
    );

    let out = EventOutput {
        op: String::from("create"),
        output_file_name: md_file_name,
        output_file_data: base64::encode(html_output),
    };

    // write output to host using json encoded bytes
    host.output(&serde_json::to_string(&out).expect("output data to json"));

    return 0;
}
