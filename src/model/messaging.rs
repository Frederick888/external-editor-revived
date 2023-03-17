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
const HEADER_DELIVERY_STATUS_NOTIFICATION: &str = "X-ExtEditorR-Delivery-Status-Notification";
const HEADER_LOWER_DELIVERY_STATUS_NOTIFICATION: &str = "x-exteditorr-delivery-status-notification"; // cspell: disable-line
const HEADER_RETURN_RECEIPT: &str = "X-ExtEditorR-Return-Receipt";
const HEADER_LOWER_RETURN_RECEIPT: &str = "x-exteditorr-return-receipt"; // cspell: disable-line
const HEADER_SEND_ON_EXIT: &str = "X-ExtEditorR-Send-On-Exit";
const HEADER_LOWER_SEND_ON_EXIT: &str = "x-exteditorr-send-on-exit"; // cspell: disable-line
const HEADER_ALLOW_X_HEADERS: &str = "X-ExtEditorR-Allow-X-Headers";
const HEADER_LOWER_ALLOW_X_HEADERS: &str = "x-exteditorr-allow-x-headers"; // cspell: disable-line
const HEADER_LOWER_ALLOW_CUSTOM_HEADERS: &str = "x-exteditorr-allow-custom-headers"; // cspell: disable-line
const HEADER_LOWER_CUSTOM_HEADER: &str = "x-exteditorr-custom-header"; // cspell: disable-line
const HEADER_LOWER_X_HEADER: &str = "x-exteditorr-x-header"; // cspell: disable-line
const HEADER_HELP: &str = "X-ExtEditorR-Help";
const HEADER_LOWER_HELP: &str = "x-exteditorr-help"; // cspell: disable-line
const HEADER_HELP_LINES: &[&str] = &[
    "Use one address per `To/Cc/Bcc/Reply-To` header",
    "    (e.g. two recipients require two `To:` headers).",
    "Remove surrounding brackets from header values",
    "    to override default settings.",
    "Custom header names must start with \"X-\".",
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
    pub allow_custom_headers: bool,
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
        if let Some(delivery_status_notification) =
            self.compose_details.delivery_status_notification
        {
            writeln_crlf!(
                w,
                "{}: {}",
                HEADER_DELIVERY_STATUS_NOTIFICATION,
                delivery_status_notification
            )?;
        }
        if let Some(return_receipt) = self.compose_details.return_receipt {
            writeln_crlf!(w, "{}: {}", HEADER_RETURN_RECEIPT, return_receipt)?;
        }
        writeln_crlf!(
            w,
            "{}: {}",
            HEADER_SEND_ON_EXIT,
            self.configuration.send_on_exit
        )?;
        writeln_crlf!(
            w,
            "{}: {}",
            HEADER_ALLOW_X_HEADERS,
            self.configuration.allow_custom_headers
                || !self.compose_details.custom_headers.is_empty()
        )?;
        for custom_header in &self.compose_details.custom_headers {
            writeln_crlf!(w, "{}: {}", custom_header.name, custom_header.value)?;
        }
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
        self.compose_details.custom_headers.clear();
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
                    HEADER_LOWER_DELIVERY_STATUS_NOTIFICATION => {
                        self.compose_details.delivery_status_notification =
                            Some(bool::from_str(header_value)?);
                    }
                    HEADER_LOWER_RETURN_RECEIPT => {
                        self.compose_details.return_receipt = Some(bool::from_str(header_value)?);
                    }
                    HEADER_LOWER_ALLOW_X_HEADERS | HEADER_LOWER_ALLOW_CUSTOM_HEADERS => {
                        self.configuration.allow_custom_headers = bool::from_str(header_value)?;
                    }
                    HEADER_LOWER_X_HEADER | HEADER_LOWER_CUSTOM_HEADER => {
                        self.compose_details
                            .custom_headers
                            .push(Self::parse_custom_header(header_value)?);
                    }
                    HEADER_LOWER_SEND_ON_EXIT => {
                        self.configuration.send_on_exit = header_value == "true"
                    }
                    HEADER_LOWER_HELP => {}
                    _ if header_name.starts_with("X-") || header_name.starts_with("x-") => {
                        // Thunderbird throws error if header name doesn't start with X-
                        self.compose_details
                            .custom_headers
                            .push(CustomHeader::new(header_name, header_value));
                    }
                    _ => {
                        unknown_headers.push(header_name.to_owned());
                    }
                }
            } else {
                eprintln!("ExtEditorR failed to process header {line}");
            }
            buf.clear();
        }
        if !self.configuration.allow_custom_headers {
            self.compose_details
                .custom_headers
                .drain(..)
                .for_each(|custom_header| unknown_headers.push(custom_header.name));
        }
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
        if header_value.starts_with('[') && header_value.ends_with(']') {
            Ok(None)
        } else {
            let parsed = T::from_str(header_value).map_err(|_| {
                anyhow!("ExtEditorR failed to parse {header_name} value: {header_value}")
            })?;
            Ok(Some(parsed))
        }
    }

    fn parse_custom_header(header_value: &str) -> Result<CustomHeader> {
        match header_value.split_once(':') {
            Some((custom_header_name, custom_header_value)) => {
                Ok(CustomHeader::new(custom_header_name, custom_header_value))
            }
            None => Err(anyhow!(
                "ExtEditorR failed to parse custom header: {header_value}"
            )),
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

        let output = to_eml_and_assert(&request);
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

        let output = to_eml_and_assert(&request);
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

        let output = to_eml_and_assert(&request);
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
        let output = to_eml_and_assert(&request);
        assert!(!output.contains("X-ExtEditorR-Delivery-Format:"));

        request.compose_details.delivery_format = Some(None);
        let output = to_eml_and_assert(&request);
        assert!(output.contains("X-ExtEditorR-Delivery-Format: [auto]"));

        request.compose_details.delivery_format = Some(Some(DeliveryFormat::Both));
        let output = to_eml_and_assert(&request);
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

        let output = to_eml_and_assert(&request);
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

        let output = to_eml_and_assert(&request);
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
    fn merge_delivery_status_notification_test() {
        let mut request = get_blank_compose();

        let output = to_eml_and_assert(&request);
        assert!(!output.contains("X-ExtEditorR-Delivery-Status-Notification:"));

        request.compose_details.delivery_status_notification = Some(false);
        let output = to_eml_and_assert(&request);
        assert!(output.contains("X-ExtEditorR-Delivery-Status-Notification: false"));

        let mut eml =
            "X-ExtEditorR-Delivery-Status-Notification: true\r\n\r\nThis is a test.\r\n".as_bytes();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert_eq!(
            Some(true),
            responses[0].compose_details.delivery_status_notification
        );
    }

    #[test]
    fn merge_return_receipt_test() {
        let mut request = get_blank_compose();

        let output = to_eml_and_assert(&request);
        assert!(!output.contains("X-ExtEditorR-Return-Receipt:"));

        request.compose_details.return_receipt = Some(false);
        let output = to_eml_and_assert(&request);
        assert!(output.contains("X-ExtEditorR-Return-Receipt: false"));

        let mut eml = "X-ExtEditorR-Return-Receipt: true\r\n\r\nThis is a test.\r\n".as_bytes();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert_eq!(Some(true), responses[0].compose_details.return_receipt);
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
        let mut eml = "Foo: hello\r\nX-ExtEditorR-Send-On-Exit: true\r\nX-Bar: world\r\n\r\nThis is a test.\r\n".as_bytes();
        let mut request = get_blank_compose();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert_eq!(1, responses[0].warnings.len());
        assert_eq!("Unknown header(s) found", responses[0].warnings[0].title);
        assert_eq!(
            "ExtEditorR did not recognise the following headers:\n- Foo\n- X-Bar",
            responses[0].warnings[0].message
        );
        assert!(!responses[0].configuration.send_on_exit);
    }

    #[test]
    fn custom_headers_test() {
        let mut request = get_blank_compose();

        let output = to_eml_and_assert(&request);
        assert!(output.contains("X-ExtEditorR-Allow-X-Headers: false"));

        request.configuration.allow_custom_headers = true;
        let output = to_eml_and_assert(&request);
        assert!(output.contains("X-ExtEditorR-Allow-X-Headers: true"));

        request.compose_details.custom_headers.push(CustomHeader {
            name: "X-Foo".to_owned(),
            value: "Hello, world!".to_owned(),
        });
        request.configuration.allow_custom_headers = false;
        let output = to_eml_and_assert(&request);
        assert!(output.contains("X-ExtEditorR-Allow-X-Headers: true"));
        assert!(output.contains("X-Foo: Hello, world!"));

        let mut eml =
            "X-Bar: Hello\r\nX-ExtEditorR-Allow-X-Headers: true\r\n\r\nThis is a test.\r\n"
                .as_bytes();
        request.configuration.allow_custom_headers = false;
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert!(responses[0].warnings.is_empty());
        assert_eq!(1, responses[0].compose_details.custom_headers.len());
        assert_eq!(
            "X-Bar",
            &responses[0].compose_details.custom_headers[0].name
        );
        assert_eq!(
            "Hello",
            &responses[0].compose_details.custom_headers[0].value
        );

        let eml = [
            "X-ExtEditorR-X-Header: X-ExtEditorR-Send-On-Exit: Hello",
            "X-ExtEditorR-Custom-Header: x-ExtEditorR-X-Header: Hello",
            "X-ExtEditorR-Allow-Custom-Headers: true",
            "",
            "This is a test.",
            "",
        ]
        .join("\r\n")
        .into_bytes();
        request.configuration.allow_custom_headers = false;
        request.compose_details.custom_headers.clear();
        let responses = request.merge_from_eml(&mut eml.as_slice(), 512).unwrap();
        assert_eq!(1, responses.len());
        assert!(responses[0].warnings.is_empty());
        assert_eq!(2, responses[0].compose_details.custom_headers.len());
        assert_eq!(
            "X-ExtEditorR-Send-On-Exit",
            &responses[0].compose_details.custom_headers[0].name
        );
        assert_eq!(
            "Hello",
            &responses[0].compose_details.custom_headers[0].value
        );
        assert_eq!(
            "X-ExtEditorR-X-Header",
            &responses[0].compose_details.custom_headers[1].name
        );
        assert_eq!(
            "Hello",
            &responses[0].compose_details.custom_headers[1].value
        );

        let mut eml = "X-Bar: Hello\r\n\r\nThis is a test.\r\n".as_bytes();
        request.configuration.allow_custom_headers = false;
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert_eq!(1, responses[0].warnings.len());
        assert_eq!("Unknown header(s) found", responses[0].warnings[0].title);
        assert_eq!(
            "ExtEditorR did not recognise the following headers:\n- X-Bar",
            responses[0].warnings[0].message
        );
        assert!(!responses[0].configuration.send_on_exit);

        let mut eml = "Bar: Hello\r\nX-ExtEditorR-Allow-X-Headers: true\r\n\r\nThis is a test.\r\n"
            .as_bytes();
        request.configuration.allow_custom_headers = false;
        request.warnings.clear();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert_eq!(1, responses[0].warnings.len());
        assert_eq!("Unknown header(s) found", responses[0].warnings[0].title);
        assert_eq!(
            "ExtEditorR did not recognise the following headers:\n- Bar",
            responses[0].warnings[0].message
        );
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
        let output = to_eml_and_assert(&request);
        assert!(output.contains("X-ExtEditorR-Help"));

        request.configuration.suppress_help_headers = true;
        let output = to_eml_and_assert(&request);
        assert!(!output.contains("X-ExtEditorR-Help"));
    }

    fn to_eml_and_assert(compose: &Compose) -> String {
        let mut buf = Vec::new();
        let result = compose.to_eml(&mut buf);
        assert!(result.is_ok());
        String::from_utf8(buf).unwrap()
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
                allow_custom_headers: false,
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
