mod model;
mod transport;
mod util;

use model::app_manifest::AppManifest;
use model::messaging::{self, Compose, Exchange, Ping};
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process;
use std::thread;
use transport::Transport;

const TEMPLATE_TEMP_FILE_NAME: &str = "/path/to/temp.eml";
const DEFAULT_SHELL_ARGS: &[&str] = &["-c"];
const DEFAULT_SHELL_ARGS_MACOS: &[&str] = &["-i", "-l", "-c"];

fn handle_ping<T>(mut request: Ping)
where
    T: transport::Transport,
{
    request.pong = request.ping;
    request.host_version = env!("CARGO_PKG_VERSION").to_string();
    request.compatible = util::is_extension_compatible(env!("CARGO_PKG_VERSION"), &request.version);
    if let Err(write_error) = T::write_message(&request) {
        eprintln!("ExtEditorR failed to send response to Thunderbird: {write_error}");
    }
}

fn handle_compose<T>(request: Compose)
where
    T: transport::Transport,
{
    let temp_filename = util::get_temp_filename(&request);
    if let Err(e) = handle_eml::<T>(request, &temp_filename) {
        eprintln!("{}: {}", e.title, e.message);
        if let Err(write_error) = T::write_message(&e) {
            eprintln!("ExtEditorR failed to send response to Thunderbird: {write_error}");
        }
    } else if let Err(remove_error) = fs::remove_file(&temp_filename) {
        eprintln!(
            "ExtEditorR failed to remove temporary file {}: {}",
            temp_filename.to_string_lossy(),
            remove_error
        );
    }
}

fn handle_eml<T>(request: Compose, temp_filename: &Path) -> Result<(), messaging::Error>
where
    T: transport::Transport,
{
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
                reset: false,  // users may want to enable bypass_version_check *and* reload
                               // ExtEditorR to recover
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
        let mut temp_file = fs::File::create(temp_filename).map_err(|e| messaging::Error {
            tab: request.tab.clone(),
            reset: true,
            title: "ExtEditorR failed to create temporary file".to_owned(),
            message: e.to_string(),
        })?;
        request
            .to_eml(&mut temp_file)
            .map_err(|e| messaging::Error {
                tab: request.tab.clone(),
                reset: true,
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
            reset: true,
            title: "ExtEditorR failed to start editor".to_owned(),
            message: e.to_string(),
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_string();
        return Err(messaging::Error {
            tab: request.tab,
            reset: false,
            title: "ExtEditorR encountered error from external editor".to_owned(),
            message: util::error_message_with_path(stderr, temp_filename),
        });
    }

    let mut response = request;

    {
        let temp_file = fs::File::open(temp_filename).map_err(|e| messaging::Error {
            tab: response.tab.clone(),
            reset: false,
            title: "ExtEditorR failed to read from temporary file".to_owned(),
            message: util::error_message_with_path(e, temp_filename),
        })?;

        let mut reader = io::BufReader::new(temp_file);
        let responses = response
            .merge_from_eml(&mut reader, messaging::MAX_BODY_LENGTH)
            .map_err(|e| messaging::Error {
                tab: response.tab.clone(),
                reset: false,
                title: "ExtEditorR failed to process temporary file".to_owned(),
                message: util::error_message_with_path(e, temp_filename),
            })?;

        for response in responses {
            if let Err(e) = T::write_message(&response) {
                eprintln!("ExtEditorR failed to send response to Thunderbird: {e}");
            }
        }
    }

    Ok(())
}

fn print_help() -> anyhow::Result<()> {
    match env::current_exe() {
        Ok(program_path) => {
            let native_app_manifest = AppManifest::new(&program_path.to_string_lossy());
            let app_name = native_app_manifest.name;
            eprintln!("Please create '{app_name}.json' manifest file with the JSON below.");
            if cfg!(target_os = "macos") {
                eprintln!(
                    "Under macOS this is usually ~/Library/Mozilla/NativeMessagingHosts/{app_name}.json,\n\
                    or /Library/Application Support/Mozilla/NativeMessagingHosts/{app_name}.json for global visibility."
                );
            } else {
                eprintln!(
                    "Consult https://wiki.mozilla.org/WebExtensions/Native_Messaging for its location."
                );
            }
            eprintln!();
            println!("{}", serde_json::to_string_pretty(&native_app_manifest)?);
        }
        Err(e) => eprintln!("Failed to determine program path: {e}"),
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

    type Tr = transport::ThunderbirdTransport;
    loop {
        let request = Tr::read_message::<Exchange>()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        thread::spawn(move || match request {
            Exchange::Ping(ping) => handle_ping::<Tr>(ping),
            Exchange::Compose(compose) => handle_compose::<Tr>(compose),
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use model::messaging::tests::get_blank_compose;

    type MockTr = transport::MockTransport;
    static WRITE_MESSAGE_CONTEXT_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn ping_pong_test() {
        let ping_json = r#"{"ping": 123456}"#;
        let ping: Ping = serde_json::from_str(ping_json).unwrap();

        let _guard = WRITE_MESSAGE_CONTEXT_LOCK.lock().unwrap();
        let ctx = MockTr::write_message_context();
        ctx.expect::<Ping>()
            .withf(|p: &Ping| {
                p.ping == 123456
                    && p.pong == 123456
                    && !p.compatible
                    && p.host_version == env!("CARGO_PKG_VERSION")
            })
            .returning(|&_| Ok(()));
        handle_ping::<MockTr>(ping);
        ctx.checkpoint();
    }

    #[test]
    fn ping_pong_successful_version_check_test() {
        let host_version = env!("CARGO_PKG_VERSION");
        let ping_json = format!(r#"{{"ping": 123456, "version": "{}"}}"#, host_version);
        let ping: Ping = serde_json::from_str(&ping_json).unwrap();

        let _guard = WRITE_MESSAGE_CONTEXT_LOCK.lock().unwrap();
        let ctx = MockTr::write_message_context();
        ctx.expect::<Ping>()
            .withf(|p: &Ping| {
                let host_version = host_version.to_string();
                p.ping == 123456
                    && p.pong == 123456
                    && p.compatible
                    && p.host_version == host_version
            })
            .returning(|&_| Ok(()));
        handle_ping::<MockTr>(ping);
        ctx.checkpoint();
    }

    #[test]
    fn ping_pong_failed_version_check_test() {
        let host_version = env!("CARGO_PKG_VERSION");
        let ping_json = r#"{"ping": 123456, "version": "0.0.0.0"}"#;
        let ping: Ping = serde_json::from_str(ping_json).unwrap();

        let _guard = WRITE_MESSAGE_CONTEXT_LOCK.lock().unwrap();
        let ctx = MockTr::write_message_context();
        ctx.expect::<Ping>()
            .withf(|p: &Ping| {
                let host_version = host_version.to_string();
                p.ping == 123456
                    && p.pong == 123456
                    && !p.compatible
                    && p.host_version == host_version
            })
            .returning(|&_| Ok(()));
        handle_ping::<MockTr>(ping);
        ctx.checkpoint();
    }

    #[test]
    fn echo_compose_test() {
        let mut compose = get_blank_compose();
        compose.configuration.version = env!("CARGO_PKG_VERSION").to_owned();
        compose.configuration.shell = "sh".to_string();
        compose.configuration.template = r#"cat "/path/to/temp.eml""#.to_owned();
        compose.configuration.temporary_directory = ".".to_owned();
        compose.tab.id = 1;
        compose.compose_details.plain_text_body = "Hello, world!\r\n".to_owned();

        let _guard = WRITE_MESSAGE_CONTEXT_LOCK.lock().unwrap();
        let ctx = MockTr::write_message_context();
        ctx.expect::<Compose>()
            .withf(|c: &Compose| {
                c.compose_details.plain_text_body == "Hello, world!\r\n"
                    && c.configuration.total == 1
            })
            .returning(|&_| Ok(()));
        handle_compose::<MockTr>(compose);
        ctx.checkpoint();
    }
}
