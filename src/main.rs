mod model;
mod util;

use model::app_manifest::AppManifest;
use model::messaging::{self, Exchange};
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process;
use std::thread;

const TEMPLATE_TEMP_FILE_NAME: &str = "/path/to/temp.eml";

fn handle(request: Exchange, temp_filename: &Path) -> Result<(), messaging::Error> {
    if request.configuration.version != env!("CARGO_PKG_VERSION") {
        if request.configuration.bypass_version_check {
            eprintln!(
                "Bypassing version check: Thunderbird extension is {} while native messaging host is {}.",
                request.configuration.version,
                env!("CARGO_PKG_VERSION")
            );
        } else {
            return Err(messaging::Error{
                tab: request.tab.clone(),
                title: "ExtEditorR version mismatch!".to_owned(),
                message: format!(
                    "Thunderbird extension is {} while native messaging host is {}. The request has been discarded.",
                    request.configuration.version,
                    env!("CARGO_PKG_VERSION")
                ),
            });
        }
    }

    {
        let mut temp_file = fs::File::create(&temp_filename).map_err(|e| messaging::Error {
            tab: request.tab.clone(),
            title: "ExtEditorR failed to create temporary file".to_owned(),
            message: e.to_string(),
        })?;
        request
            .to_eml(&mut temp_file)
            .map_err(|e| messaging::Error {
                tab: request.tab.clone(),
                title: "ExtEditorR failed to write to temporary file".to_owned(),
                message: e.to_string(),
            })?;
    }

    let command = request
        .configuration
        .template
        .replace(TEMPLATE_TEMP_FILE_NAME, &temp_filename.to_string_lossy());
    let mut proc = process::Command::new(&request.configuration.shell)
        .arg("-c")
        .arg(command)
        .spawn()
        .map_err(|e| messaging::Error {
            tab: request.tab.clone(),
            title: "ExtEditorR failed to start editor".to_owned(),
            message: e.to_string(),
        })?;

    proc.wait().map_err(|e| messaging::Error {
        tab: request.tab.clone(),
        title: "ExtEditorR encountered error from external editor".to_owned(),
        message: util::error_message_with_path(e, temp_filename),
    })?;

    let mut response = request;

    {
        let temp_file = fs::File::open(&temp_filename).map_err(|e| messaging::Error {
            tab: response.tab.clone(),
            title: "ExtEditorR failed to read from temporary file".to_owned(),
            message: util::error_message_with_path(e, temp_filename),
        })?;

        let mut reader = io::BufReader::new(temp_file);
        let responses = response
            .merge_from_eml(&mut reader, messaging::MAX_BODY_LENGTH)
            .map_err(|e| messaging::Error {
                tab: response.tab.clone(),
                title: "ExtEditorR failed to process temporary file".to_owned(),
                message: util::error_message_with_path(e, temp_filename),
            })?;

        for response in responses {
            if let Err(e) = web_ext_native_messaging::write_message(&response) {
                eprint!("ExtEditorR failed to send response to Thunderbird: {}", e);
            }
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    if env::args().count() == 1 {
        // Thunderbird calls us with: /path/to/external-editor-revived /path/to/native-messaging-hosts/external_editor_revived.json external-editor-revived@tsundere.moe
        match env::current_exe() {
            Ok(program_path) => {
                let native_app_manifest = AppManifest::new(&program_path.to_string_lossy());
                eprintln!(
                    "Please create '{}.json' manifest file with the JSON below.",
                    native_app_manifest.name
                );
                eprintln!(
                    "Consult https://wiki.mozilla.org/WebExtensions/Native_Messaging for its location.\n"
                );
                println!("{}", serde_json::to_string_pretty(&native_app_manifest)?);
            }
            Err(e) => eprint!("Failed to determine program path: {}", e),
        }
        return Ok(());
    }

    loop {
        let request = web_ext_native_messaging::read_message::<Exchange>()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        thread::spawn(move || {
            let temp_filename = util::get_temp_filename(&request.tab);
            if let Err(e) = handle(request, &temp_filename) {
                if let Err(write_error) = web_ext_native_messaging::write_message(&e) {
                    eprint!(
                        "ExtEditorR failed to send response to Thunderbird: {}",
                        write_error
                    );
                }
            } else if let Err(remove_error) = fs::remove_file(&temp_filename) {
                eprint!(
                    "ExtEditorR failed to remove temporary file {}: {}",
                    temp_filename.to_string_lossy(),
                    remove_error
                );
            }
        });
    }
}
