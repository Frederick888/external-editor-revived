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
const DEFAULT_SHELL_ARGS: &[&str] = &["-c"];
const DEFAULT_SHELL_ARGS_MACOS: &[&str] = &["-i", "-l", "-c"];

fn handle(request: Exchange, temp_filename: &Path) -> Result<(), messaging::Error> {
    if !util::is_extension_compatible(env!("CARGO_PKG_VERSION"), &request.configuration.version) {
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

    let command = if cfg!(target_os = "windows") {
        request.configuration.template.replace(
            TEMPLATE_TEMP_FILE_NAME,
            &temp_filename.to_string_lossy().replace('\\', "\\\\"),
        )
    } else {
        request
            .configuration
            .template
            .replace(TEMPLATE_TEMP_FILE_NAME, &temp_filename.to_string_lossy())
    };
    let output = process::Command::new(&request.configuration.shell)
        .args(if cfg!(target_os = "macos") {
            DEFAULT_SHELL_ARGS_MACOS
        } else {
            DEFAULT_SHELL_ARGS
        })
        .arg(command)
        .output()
        .map_err(|e| messaging::Error {
            tab: request.tab.clone(),
            title: "ExtEditorR failed to start editor".to_owned(),
            message: e.to_string(),
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_string();
        return Err(messaging::Error {
            tab: request.tab,
            title: "ExtEditorR encountered error from external editor".to_owned(),
            message: util::error_message_with_path(stderr, temp_filename),
        });
    }

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

fn print_help() -> anyhow::Result<()> {
    match env::current_exe() {
        Ok(program_path) => {
            let native_app_manifest = AppManifest::new(&program_path.to_string_lossy());
            eprintln!(
                "Please create '{}.json' manifest file with the JSON below.",
                native_app_manifest.name
            );
            eprintln!(
                "Consult https://wiki.mozilla.org/WebExtensions/Native_Messaging for its location."
            );
            if cfg!(target_os = "macos") {
                eprintln!(
                    "Under macOS this is usually ~/Library/Mozilla/NativeMessagingHosts/{}.json.",
                    native_app_manifest.name
                );
            }
            eprintln!();
            println!("{}", serde_json::to_string_pretty(&native_app_manifest)?);
        }
        Err(e) => eprint!("Failed to determine program path: {}", e),
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    if env::args().count() == 1 {
        // Thunderbird calls us with: /path/to/external-editor-revived /path/to/native-messaging-hosts/external_editor_revived.json external-editor-revived@tsundere.moe
        return print_help();
    }
    if let Some(arg) = env::args().nth(1) {
        match arg.as_str() {
            "-v" | "--version" => {
                println!(
                    "External Editor Revived native messaging host for {} ({}) v{}",
                    env::consts::OS,
                    env::consts::ARCH,
                    env!("CARGO_PKG_VERSION")
                );
                return Ok(());
            }
            "-h" | "--help" => {
                return print_help();
            }
            _ => {}
        }
    }

    loop {
        let request = web_ext_native_messaging::read_message::<Exchange>()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        thread::spawn(move || {
            let temp_filename = util::get_temp_filename(&request.tab);
            if let Err(e) = handle(request, &temp_filename) {
                eprintln!("{}: {}", e.title, e.message);
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
