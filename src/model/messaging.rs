use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::{io, str::FromStr};

use super::thunderbird::*;
use crate::writeln_crlf;

pub const MAX_BODY_LENGTH: usize = 768 * 1024;

const HEADER_PRIORITY: &str = "X-ExtEditorR-Priority";
const HEADER_LOWER_PRIORITY: &str = "x-exteditorr-priority"; // cspell: disable-line
const HEADER_DELIVERY_FORMAT: &str = "X-ExtEditorR-Delivery-Format";
const HEADER_LOWER_DELIVERY_FORMAT: &str = "x-exteditorr-delivery-format"; // cspell: disable-line
const HEADER_ATTACH_VCARD: &str = "X-ExtEditorR-Attach-vCard";
const HEADER_LOWER_ATTACH_VCARD: &str = "x-exteditorr-attach-vcard"; // cspell: disable-line
const HEADER_SEND_ON_EXIT: &str = "X-ExtEditorR-Send-On-Exit";
const HEADER_LOWER_SEND_ON_EXIT: &str = "x-exteditorr-send-on-exit"; // cspell: disable-line
const HEADER_HELP: &str = "X-ExtEditorR-Help";
const HEADER_LOWER_HELP: &str = "x-exteditorr-help"; // cspell: disable-line
const HEADER_HELP_LINES: &[&str] = &[
    "Use one address per `To/Cc/Bcc/Reply-To` header",
    "(e.g. two recipients require two `To:` headers).",
    "Remove brackets from `X-ExtEditorR-Attach-vCard`",
    "to override identity's default setting.",
    "KEEP blank line below to separate headers from body.",
];

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Exchange {
    Ping(Ping),
    Compose(Compose),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ping {
    pub ping: u64,
    #[serde(default)]
    pub pong: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub version: String,
    #[serde(default)]
    pub sequence: usize,
    #[serde(default)]
    pub total: usize,
    #[serde(skip_serializing)]
    pub shell: String,
    #[serde(skip_serializing)]
    pub template: String,
    #[serde(default)]
    pub temporary_directory: String,
    #[serde(default)]
    pub send_on_exit: bool,
    #[serde(default)]
    pub suppress_help_headers: bool,
    #[serde(default)]
    pub bypass_version_check: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Compose {
    pub configuration: Configuration,
    #[serde(default)]
    pub warnings: Vec<Warning>,
    pub tab: Tab,
    #[serde(rename = "composeDetails")]
    pub compose_details: ComposeDetails,
}

impl Compose {
    pub fn to_eml<W>(&self, w: &mut W) -> Result<()>
    where
        W: io::Write,
    {
        writeln_crlf!(w, "From: {}", self.compose_details.from.to_header_value()?)?;
        Self::compose_recipient_list_to_eml(w, "To", &self.compose_details.to)?;
        Self::compose_recipient_list_to_eml(w, "Cc", &self.compose_details.cc)?;
        Self::compose_recipient_list_to_eml(w, "Bcc", &self.compose_details.bcc)?;
        Self::compose_recipient_list_to_eml(w, "Reply-To", &self.compose_details.reply_to)?;
        writeln_crlf!(w, "Subject: {}", self.compose_details.subject)?;
        if let Some(ref priority) = self.compose_details.priority {
            writeln_crlf!(w, "{}: {}", HEADER_PRIORITY, priority)?;
        }
        if let Some(ref delivery_format) = self.compose_details.delivery_format {
            match delivery_format {
                Some(delivery_format) => {
                    writeln_crlf!(w, "{}: [{}]", HEADER_DELIVERY_FORMAT, delivery_format)?
                }
                None => writeln_crlf!(w, "{}: [{}]", HEADER_DELIVERY_FORMAT, DeliveryFormat::Auto)?,
            }
        }
        if let Some(attach_vcard) = self.compose_details.attach_vcard.inner {
            writeln_crlf!(w, "{}: [{}]", HEADER_ATTACH_VCARD, attach_vcard)?;
        }
        writeln_crlf!(
            w,
            "{}: {}",
            HEADER_SEND_ON_EXIT,
            self.configuration.send_on_exit
        )?;
        if !self.configuration.suppress_help_headers {
            Self::write_help_headers(w)?;
        }
        writeln_crlf!(w)?;
        write!(w, "{}", self.compose_details.get_body())?;
        Ok(())
    }

    pub fn merge_from_eml<R>(&mut self, r: &mut R, max_body_length: usize) -> Result<Vec<Self>>
    where
        R: io::BufRead,
    {
        let mut compose_details_list: Vec<ComposeDetails> = Vec::new();

        self.compose_details.clear_recipients();
        self.configuration.send_on_exit = false;

        let mut buf = Vec::new();
        // read headers
        let mut unknown_headers = Vec::new();
        while let Ok(length) = r.read_until(b'\n', &mut buf) {
            if length == 0 {
                break;
            }
            let line = String::from_utf8_lossy(&buf).trim().to_owned();
            if line.is_empty() {
                break;
            }
            if let Some((header_name, header_value)) = line.split_once(':') {
                let header_name_lower = header_name.trim().to_lowercase();
                let header_value = header_value.trim();
                if header_value.is_empty() {
                    buf.clear();
                    continue;
                }
                match header_name_lower.as_str() {
                    "from" => {
                        self.compose_details.from =
                            ComposeRecipient::from_header_value(header_value)?
                    }
                    "to" => self
                        .compose_details
                        .add_to(ComposeRecipient::from_header_value(header_value)?),
                    "cc" => self
                        .compose_details
                        .add_cc(ComposeRecipient::from_header_value(header_value)?),
                    "bcc" => self
                        .compose_details
                        .add_bcc(ComposeRecipient::from_header_value(header_value)?),
                    "reply-to" => self
                        .compose_details
                        .add_reply_to(ComposeRecipient::from_header_value(header_value)?),
                    "subject" => self.compose_details.subject = header_value.to_string(),
                    HEADER_LOWER_PRIORITY => {
                        self.compose_details.priority = Some(Priority::from_str(header_value)?)
                    }
                    HEADER_LOWER_DELIVERY_FORMAT => {
                        if let Some(delivery_format) = Self::parse_optional_header::<DeliveryFormat>(
                            HEADER_DELIVERY_FORMAT,
                            header_value,
                        )? {
                            self.compose_details.delivery_format = Some(Some(delivery_format));
                        }
                    }
                    HEADER_LOWER_ATTACH_VCARD => {
                        if let Some(attach_vcard) =
                            Self::parse_optional_header::<bool>(HEADER_ATTACH_VCARD, header_value)?
                        {
                            self.compose_details.attach_vcard.set(attach_vcard);
                        }
                    }
                    HEADER_LOWER_SEND_ON_EXIT => {
                        self.configuration.send_on_exit = header_value == "true"
                    }
                    HEADER_LOWER_HELP => {}
                    _ => {
                        unknown_headers.push(header_name.to_owned());
                        eprintln!("ExtEditorR encountered unknown header {header_name} when processing temporary file");
                    }
                }
            } else {
                eprintln!("ExtEditorR failed to process header {line}");
            }
            buf.clear();
        }
        // warning for unknown headers
        if !unknown_headers.is_empty() {
            let mut message = "ExtEditorR did not recognise the following headers:\n".to_string();
            message += &unknown_headers
                .iter()
                .map(|h| "- ".to_owned() + h)
                .collect::<Vec<String>>()
                .join("\n");
            let warning = Warning {
                title: "Unknown header(s) found".to_owned(),
                message,
            };
            self.warnings.push(warning);
        }
        // disable send-on-exit if there are warnings
        if !self.warnings.is_empty() {
            self.configuration.send_on_exit = false;
        }
        // read body
        self.compose_details.body.clear();
        self.compose_details.plain_text_body.clear();
        buf.clear();
        r.read_to_end(&mut buf)?;
        let body = String::from_utf8_lossy(&buf);
        let mut chunk = String::new();
        for c in body.chars() {
            chunk.push(c);
            if chunk.len() > max_body_length {
                self.compose_details.set_body(chunk.clone());
                compose_details_list.push(self.compose_details.clone());
                chunk.clear();
            }
        }
        self.compose_details.set_body(chunk.clone());
        if !chunk.is_empty() || compose_details_list.is_empty() {
            compose_details_list.push(self.compose_details.clone());
        }

        let mut responses: Vec<Self> = compose_details_list
            .into_iter()
            .map(|compose_details| {
                let mut response = self.clone();
                response.compose_details = compose_details;
                response
            })
            .collect();
        let responses_len = responses.len();
        for (i, response) in responses.iter_mut().enumerate() {
            response.configuration.sequence = i;
            response.configuration.total = responses_len;
        }
        Ok(responses)
    }

    fn compose_recipient_list_to_eml<W>(
        w: &mut W,
        name: &str,
        list: &ComposeRecipientList,
    ) -> Result<()>
    where
        W: io::Write,
    {
        match list {
            ComposeRecipientList::Single(recipient) => {
                writeln_crlf!(w, "{}: {}", name, recipient.to_header_value()?)?;
            }
            ComposeRecipientList::Multiple(recipients) if recipients.is_empty() => {
                writeln_crlf!(w, "{}: ", name)?;
            }
            ComposeRecipientList::Multiple(recipients) => {
                for recipient in recipients {
                    writeln_crlf!(w, "{}: {}", name, recipient.to_header_value()?)?;
                }
            }
        }
        Ok(())
    }

    fn write_help_headers<W>(w: &mut W) -> Result<()>
    where
        W: io::Write,
    {
        for line in HEADER_HELP_LINES {
            writeln_crlf!(w, "{}: {}", HEADER_HELP, line)?;
        }
        Ok(())
    }

    fn parse_optional_header<T>(header_name: &str, header_value: &str) -> Result<Option<T>>
    where
        T: FromStr,
        <T as FromStr>::Err: StdError + 'static,
    {
        match header_value {
            _ if header_value.starts_with('[') && header_value.ends_with(']') => Ok(None),
            header_value => {
                let parsed = T::from_str(header_value).map_err(|_| {
                    anyhow!("ExtEditorR failed to parse {header_name} value: {header_value}")
                })?;
                Ok(Some(parsed))
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Error {
    pub tab: Tab,
    pub reset: bool,
    pub title: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Warning {
    pub title: String,
    pub message: String,
}

#[cfg(test)]
pub mod tests {
    use base64::Engine;

    use super::*;
    use crate::model::thunderbird::tests::get_blank_compose_details;

    #[test]
    fn write_to_eml_test() {
        let mut request = get_blank_compose();
        request.compose_details.cc = ComposeRecipientList::Multiple(vec![
            ComposeRecipient::Email("foo@example.com".to_owned()),
            ComposeRecipient::Email("bar@example.com".to_owned()),
        ]);
        request.compose_details.subject =
            "Greetings! This is composed using External Editor Revived!".to_owned();
        request.compose_details.body = "<html>".to_owned();
        request.compose_details.plain_text_body = "Hello, world!".to_owned();

        let mut buf = Vec::new();
        let result = request.to_eml(&mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("From: someone@example.com"));
        assert!(output.contains("Cc: foo@example.com"));
        assert!(output.contains("Cc: bar@example.com"));
        assert!(output.contains(&format!("Subject: {}", request.compose_details.subject)));
        assert!(!output.contains(&format!("{HEADER_ATTACH_VCARD}:")));
        assert!(!output.contains(&format!("{HEADER_PRIORITY}:")));
        assert!(output.contains("X-ExtEditorR-Send-On-Exit: false"));
        assert!(output.ends_with(&request.compose_details.plain_text_body));
        assert!(!output.contains(&request.compose_details.body));
        assert_eq!(output.matches('\r').count(), output.matches('\n').count());
    }

    #[test]
    fn header_placeholder_test() {
        let mut request = get_blank_compose();
        request.compose_details.is_plain_text = true;
        request.compose_details.plain_text_body = "Hello, world!".to_owned();

        let mut buf = Vec::new();
        let result = request.to_eml(&mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("From: "));
        assert!(output.contains("To: "));
        assert!(output.contains("Cc: "));
        assert!(output.contains("Bcc: "));
        assert!(output.contains("Reply-To: "));
        assert!(output.contains("Subject: "));
    }

    #[test]
    fn omit_header_placeholder_when_given_test() {
        let mut request = get_blank_compose();
        request.compose_details.cc = ComposeRecipientList::Multiple(vec![
            ComposeRecipient::Email("foo@example.com".to_owned()),
            ComposeRecipient::Email("bar@example.com".to_owned()),
        ]);
        request.compose_details.is_plain_text = true;
        request.compose_details.plain_text_body = "Hello, world!".to_owned();

        let mut buf = Vec::new();
        let result = request.to_eml(&mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(2, output.matches("Cc:").count());
        assert!(output.contains("Cc: foo@example.com"));
        assert!(output.contains("Cc: bar@example.com"));
    }

    #[test]
    fn merge_subject_and_body_test() {
        let mut eml = "Subject: Hello, world! \r\n\r\nThis is a test.\r\n".as_bytes();
        let mut request = get_blank_compose();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert!(responses[0].warnings.is_empty());
        assert_eq!("Hello, world!", &responses[0].compose_details.subject);
        assert_eq!(
            "This is a test.\r\n",
            &responses[0].compose_details.plain_text_body
        );
    }

    #[test]
    fn merge_from_and_to_test() {
        let mut eml = "From: foo@example.com\r\nTo: foo@instance.com\r\nTo: {\"id\":\"bar\",\"type\":\"mailingList\"}\r\n\r\nThis is a test.\r\n".as_bytes();
        let mut request = get_blank_compose();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert!(responses[0].warnings.is_empty());
        assert_eq!(
            ComposeRecipient::Email("foo@example.com".to_owned()),
            responses[0].compose_details.from
        );
        assert_eq!(
            ComposeRecipientList::Multiple(vec![
                ComposeRecipient::Email("foo@instance.com".to_owned()),
                ComposeRecipient::Node(ComposeRecipientNode {
                    id: "bar".to_owned(),
                    node_type: ComposeRecipientNodeType::MailingList
                }),
            ]),
            responses[0].compose_details.to
        );
    }

    #[test]
    fn merge_from_and_to_lower_cases_test() {
        let mut eml = "from: foo@example.com\r\nto: foo@instance.com\r\nTo: {\"id\":\"bar\",\"type\":\"mailingList\"}\r\n\r\nThis is a test.\r\n".as_bytes();
        let mut request = get_blank_compose();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert!(responses[0].warnings.is_empty());
        assert_eq!(
            ComposeRecipient::Email("foo@example.com".to_owned()),
            responses[0].compose_details.from
        );
        assert_eq!(
            ComposeRecipientList::Multiple(vec![
                ComposeRecipient::Email("foo@instance.com".to_owned()),
                ComposeRecipient::Node(ComposeRecipientNode {
                    id: "bar".to_owned(),
                    node_type: ComposeRecipientNodeType::MailingList
                }),
            ]),
            responses[0].compose_details.to
        );
    }

    #[test]
    fn merge_with_header_placeholder_test() {
        let mut eml = "From: foo@example.com\r\nTo: bar@example.com\r\nCc: \r\nBcc: \r\nReply-To: another@example.com\r\n\r\nThis is a test.\r\n".as_bytes();
        let mut request = get_blank_compose();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert_eq!(
            ComposeRecipient::Email("foo@example.com".to_owned()),
            responses[0].compose_details.from
        );
        assert_eq!(
            ComposeRecipientList::Multiple(vec![ComposeRecipient::Email(
                "bar@example.com".to_owned()
            )]),
            responses[0].compose_details.to
        );
        assert_eq!(
            ComposeRecipientList::Multiple(vec![]),
            responses[0].compose_details.cc
        );
        assert_eq!(
            ComposeRecipientList::Multiple(vec![]),
            responses[0].compose_details.bcc
        );
        assert_eq!(
            ComposeRecipientList::Multiple(vec![ComposeRecipient::Email(
                "another@example.com".to_owned()
            )]),
            responses[0].compose_details.reply_to
        );
    }

    #[test]
    fn chunked_response_test() {
        let mut eml =
            "From: foo@example.com\r\n\r\nHello, world! Hello, world! Hello!\r\n".as_bytes();
        let mut request = get_blank_compose();
        let responses = request.merge_from_eml(&mut eml, 13).unwrap();
        assert_eq!(3, responses.len());
        assert_eq!(
            "Hello, world! ",
            &responses[0].compose_details.plain_text_body
        );
        assert_eq!(
            "Hello, world! ",
            &responses[1].compose_details.plain_text_body
        );
        assert_eq!("Hello!\r\n", &responses[2].compose_details.plain_text_body);
    }

    #[test]
    fn merge_delivery_format_test() {
        let mut request = get_blank_compose();
        let mut buf = Vec::new();
        let result = request.to_eml(&mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.contains("X-ExtEditorR-Delivery-Format:"));

        request.compose_details.delivery_format = Some(None);
        let mut buf = Vec::new();
        let result = request.to_eml(&mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("X-ExtEditorR-Delivery-Format: [auto]"));

        request.compose_details.delivery_format = Some(Some(DeliveryFormat::Both));
        let mut buf = Vec::new();
        let result = request.to_eml(&mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("X-ExtEditorR-Delivery-Format: [both]"));

        let mut eml = "X-ExtEditorR-Delivery-Format: [hello]\r\n\r\nThis is a test.\r\n".as_bytes();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert_eq!(
            &DeliveryFormat::Both,
            responses[0]
                .compose_details
                .delivery_format
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
        );

        request.compose_details.delivery_format = None;
        let mut eml =
            "X-ExtEditorR-Delivery-Format: plaintext\r\n\r\nThis is a test.\r\n".as_bytes();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert_eq!(
            &DeliveryFormat::PlainText,
            responses[0]
                .compose_details
                .delivery_format
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
        );
    }

    #[test]
    fn merge_priority_test() {
        let mut request = get_blank_compose();
        request.compose_details.priority = Some(Priority::Normal);

        let mut buf = Vec::new();
        let result = request.to_eml(&mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("X-ExtEditorR-Priority: normal"));

        let mut eml = "X-ExtEditorR-Priority: high\r\n\r\nThis is a test.\r\n".as_bytes();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert_eq!(
            &Priority::High,
            responses[0].compose_details.priority.as_ref().unwrap()
        );
    }

    #[test]
    fn merge_attach_vcard_test() {
        let mut request = get_blank_compose();
        request.compose_details.attach_vcard = TrackedOptionBool::new(false);

        let mut buf = Vec::new();
        let result = request.to_eml(&mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("X-ExtEditorR-Attach-vCard: [false]"));

        let mut eml = "X-ExtEditorR-Attach-vCard: [false]\r\n\r\nThis is a test.\r\n".as_bytes();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert!(responses[0].compose_details.attach_vcard.is_unchanged());
        assert!(!responses[0].compose_details.attach_vcard.inner.unwrap());

        let mut eml = "X-ExtEditorR-Attach-vCard: true\r\n\r\nThis is a test.\r\n".as_bytes();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert!(!responses[0].compose_details.attach_vcard.is_unchanged());
        assert!(responses[0].compose_details.attach_vcard.inner.unwrap());

        let mut eml = "X-ExtEditorR-Attach-vCard: yes\r\n\r\nThis is a test.\r\n".as_bytes();
        request.compose_details.attach_vcard = TrackedOptionBool::new(false);
        request.configuration.send_on_exit = true;
        let responses = request.merge_from_eml(&mut eml, 512);
        assert!(responses.is_err());
        let err = responses.unwrap_err();
        assert!(err
            .to_string()
            .contains("ExtEditorR failed to parse X-ExtEditorR-Attach-vCard value: yes"));
    }

    #[test]
    fn merge_send_on_exit_test() {
        let mut eml = "X-ExtEditorR-Send-On-Exit: true\r\n\r\nThis is a test.\r\n".as_bytes();
        let mut request = get_blank_compose();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert!(responses[0].configuration.send_on_exit);
    }

    #[test]
    fn unknown_headers_test() {
        let mut eml = "Foo: hello\r\nX-ExtEditorR-Send-On-Exit: true\r\nBar: world\r\n\r\nThis is a test.\r\n".as_bytes();
        let mut request = get_blank_compose();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert_eq!(1, responses[0].warnings.len());
        assert_eq!("Unknown header(s) found", responses[0].warnings[0].title);
        assert_eq!(
            "ExtEditorR did not recognise the following headers:\n- Foo\n- Bar",
            responses[0].warnings[0].message
        );
        assert!(!responses[0].configuration.send_on_exit);
    }

    #[test]
    fn delete_send_on_exit_header_test() {
        let mut eml = "Subject: Hello\r\n\r\nThis is a test.\r\n".as_bytes();
        let mut request = get_blank_compose();
        request.configuration.send_on_exit = true;
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert!(!responses[0].configuration.send_on_exit);
    }

    #[test]
    fn invalid_utf8_test() {
        let eml = {
            let mut result = "Subject: Hello\r\n\r\n".as_bytes().to_vec();
            // https://github.com/Frederick888/external-editor-revived/issues/65#issuecomment-1276693030
            let body_b64 = "PiB0aGlzIGNoYXJhY3RlciBjYXVzZXMgYmFkbmVzczoNCj4gICCVDQo=";
            let body = base64::engine::general_purpose::STANDARD
                .decode(body_b64)
                .unwrap();
            result.extend(&body);
            result
        };

        let mut request = get_blank_compose();
        request.configuration.send_on_exit = true;
        let responses = request.merge_from_eml(&mut &eml[..], 512).unwrap();
        assert_eq!(1, responses.len());
        assert!(responses[0]
            .compose_details
            .plain_text_body
            .contains("> this character causes badness:"));
        assert_eq!(
            2,
            responses[0]
                .compose_details
                .plain_text_body
                .matches("\r\n")
                .count()
        );
    }

    #[test]
    fn help_headers_test() {
        let mut request = get_blank_compose();
        let mut buf = Vec::new();
        let result = request.to_eml(&mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("X-ExtEditorR-Help"));

        request.configuration.suppress_help_headers = true;
        let mut buf = Vec::new();
        let result = request.to_eml(&mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.contains("X-ExtEditorR-Help"));
    }

    pub fn get_blank_compose() -> Compose {
        Compose {
            configuration: Configuration {
                version: "0.0.0".to_owned(),
                sequence: 0,
                total: 0,
                shell: "".to_owned(),
                template: "".to_owned(),
                temporary_directory: "".to_owned(),
                send_on_exit: false,
                suppress_help_headers: false,
                bypass_version_check: false,
            },
            warnings: Vec::new(),
            tab: Tab {
                id: 0,
                index: 0,
                window_id: 0,
                highlighted: false,
                active: false,
                status: TabStatus::Complete,
                width: 0,
                height: 0,
                tab_type: TabType::MessageCompose,
                mail_tab: false,
            },
            compose_details: get_blank_compose_details(),
        }
    }
}
