use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io;

use super::thunderbird::*;

pub const MAX_BODY_LENGTH: usize = 768 * 1024;

const HEADER_SEND_ON_EXIT: &str = "X-ExtEditorR-Send-On-Exit";
const HEADER_LOWER_SEND_ON_EXIT: &str = "x-exteditorr-send-on-exit"; // cspell: disable-line
const HEADER_HELP: &str = "X-ExtEditorR-Help";
const HEADER_LOWER_HELP: &str = "x-exteditorr-help"; // cspell: disable-line

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
    pub send_on_exit: bool,
    #[serde(default)]
    pub suppress_help_headers: bool,
    #[serde(default)]
    pub bypass_version_check: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Exchange {
    pub configuration: Configuration,
    pub tab: Tab,
    #[serde(rename = "composeDetails")]
    pub compose_details: ComposeDetails,
}

impl Exchange {
    pub fn to_eml<W>(&self, w: &mut W) -> Result<()>
    where
        W: io::Write,
    {
        writeln!(w, "From: {}", self.compose_details.from.to_header_value()?)?;
        if !self.configuration.suppress_help_headers {
            writeln!(
                w,
                "{}: You can add or delete To/Cc/Bcc/Reply-To headers",
                HEADER_HELP
            )?;
            writeln!(
                w,
                "{}: One header line can only have one email/contact/mailing list",
                HEADER_HELP
            )?;
            writeln!(
                w,
                "{}: You can though, for example, have two To: header lines if there are two recipients",
                HEADER_HELP
            )?;
        }
        Self::compose_recipient_list_to_eml(w, "To", &self.compose_details.to)?;
        Self::compose_recipient_list_to_eml(w, "Cc", &self.compose_details.cc)?;
        Self::compose_recipient_list_to_eml(w, "Bcc", &self.compose_details.bcc)?;
        Self::compose_recipient_list_to_eml(w, "Reply-To", &self.compose_details.reply_to)?;
        writeln!(w, "Subject: {}", self.compose_details.subject)?;
        if !self.configuration.suppress_help_headers {
            writeln!(
                w,
                "{}: If you update header below to {}: true, ExtEditorR will try sending out this email immediately after the editor exits",
                HEADER_HELP, HEADER_SEND_ON_EXIT
            )?;
        }
        writeln!(
            w,
            "{}: {}",
            HEADER_SEND_ON_EXIT, self.configuration.send_on_exit
        )?;
        if !self.configuration.suppress_help_headers {
            writeln!(
                w,
                "{}: Do NOT remove the blank line below separating headers from main body",
                HEADER_HELP
            )?;
        }
        writeln!(w)?;
        writeln!(w, "{}", self.compose_details.get_body())?;
        Ok(())
    }

    pub fn merge_from_eml<R>(&mut self, r: &mut R, max_body_length: usize) -> Result<Vec<Self>>
    where
        R: io::BufRead,
    {
        let mut compose_details_list: Vec<ComposeDetails> = Vec::new();

        // reset all ComposeRecipientList fields to empty ComposeRecipientList::Multiple
        self.compose_details.to = ComposeRecipientList::Multiple(Vec::new());
        self.compose_details.cc = ComposeRecipientList::Multiple(Vec::new());
        self.compose_details.bcc = ComposeRecipientList::Multiple(Vec::new());
        self.compose_details.reply_to = ComposeRecipientList::Multiple(Vec::new());

        let mut buf = String::new();
        // read headers
        while let Ok(length) = r.read_line(&mut buf) {
            if length == 0 {
                break;
            }
            let line = buf.trim();
            if line.is_empty() {
                break;
            }
            if let Some((header_name, header_value)) = line.split_once(':') {
                let header_name = header_name.trim().to_lowercase();
                let header_value = header_value.trim();
                match header_name.as_str() {
                    "from" => self.compose_details.from = ComposeRecipient::from_header_value(header_value)?,
                    "to" => match &mut self.compose_details.to {
                        ComposeRecipientList::Multiple(recipients) => recipients.push(ComposeRecipient::from_header_value(header_value)?),
                        ComposeRecipientList::Single(_) => { return Err(anyhow!("ComposeDetails field To is Single when merging EML back. This shouldn't have happened!")) },
                    },
                    "cc" => match &mut self.compose_details.cc {
                        ComposeRecipientList::Multiple(recipients) => recipients.push(ComposeRecipient::from_header_value(header_value)?),
                        ComposeRecipientList::Single(_) => { return Err(anyhow!("ComposeDetails field Cc is Single when merging EML back. This shouldn't have happened!")) },
                    },
                    "bcc" => match &mut self.compose_details.bcc {
                        ComposeRecipientList::Multiple(recipients) => recipients.push(ComposeRecipient::from_header_value(header_value)?),
                        ComposeRecipientList::Single(_) => { return Err(anyhow!("ComposeDetails field Bcc is Single when merging EML back. This shouldn't have happened!")) },
                    },
                    "reply-to" => match &mut self.compose_details.reply_to {
                        ComposeRecipientList::Multiple(recipients) => recipients.push(ComposeRecipient::from_header_value(header_value)?),
                        ComposeRecipientList::Single(_) => { return Err(anyhow!("ComposeDetails field Reply-To is Single when merging EML back. This shouldn't have happened!")) },
                    },
                    "subject" => self.compose_details.subject = header_value.to_string(),
                    HEADER_LOWER_SEND_ON_EXIT => self.configuration.send_on_exit = header_value == "true",
                    HEADER_LOWER_HELP => {},
                    _ => eprintln!("ExtEditorR encountered unknown header {} when processing temporary file", header_name),
                }
            } else {
                eprintln!("ExtEditorR failed to process header {}", line);
            }
            buf.clear();
        }
        // read body
        self.compose_details.body.clear();
        self.compose_details.plain_text_body.clear();
        buf.clear();
        r.read_to_string(&mut buf)?;
        let mut chunk = String::new();
        for c in buf.chars() {
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
        // remove redundant carriage returns / line breaks from last chunk
        if let Some(compose_details) = compose_details_list.last_mut() {
            if compose_details.is_plain_text {
                compose_details.plain_text_body =
                    compose_details.plain_text_body.trim_end().to_owned();
            }
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
                writeln!(w, "{}: {}", name, recipient.to_header_value()?)?;
            }
            ComposeRecipientList::Multiple(recipients) => {
                for recipient in recipients {
                    writeln!(w, "{}: {}", name, recipient.to_header_value()?)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Error {
    pub tab: Tab,
    pub title: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_to_eml_test() {
        let mut request = get_blank_request();
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
        assert!(output.contains("X-ExtEditorR-Send-On-Exit: false"));
        assert!(output.ends_with(&format!("{}\n", request.compose_details.plain_text_body)));
        assert!(!output.contains(&request.compose_details.body));
    }

    #[test]
    fn merge_subject_and_body_test() {
        let mut eml = "Subject: Hello, world! \r\n\r\nThis is a test.\r\n".as_bytes();
        let mut request = get_blank_request();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert_eq!("Hello, world!", &responses[0].compose_details.subject);
        assert_eq!(
            "This is a test.",
            &responses[0].compose_details.plain_text_body
        );
    }

    #[test]
    fn merge_from_and_to_test() {
        let mut eml = "From: foo@example.com\r\nTo: foo@instance.com\r\nTo: {\"id\":\"bar\",\"type\":\"mailingList\"}\r\n\r\nThis is a test.\r\n".as_bytes();
        let mut request = get_blank_request();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
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
        let mut request = get_blank_request();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
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
    fn chunked_response_test() {
        let mut eml =
            "From: foo@example.com\r\n\r\nHello, world! Hello, world! Hello!\r\n".as_bytes();
        let mut request = get_blank_request();
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
        assert_eq!("Hello!", &responses[2].compose_details.plain_text_body);
    }

    #[test]
    fn merge_send_on_exit_test() {
        let mut eml = "X-ExtEditorR-Send-On-Exit: true\r\n\r\nThis is a test.\r\n".as_bytes();
        let mut request = get_blank_request();
        let responses = request.merge_from_eml(&mut eml, 512).unwrap();
        assert_eq!(1, responses.len());
        assert!(responses[0].configuration.send_on_exit);
    }

    #[test]
    fn help_headers_test() {
        let mut request = get_blank_request();
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

    fn get_blank_request() -> Exchange {
        Exchange {
            configuration: Configuration {
                version: "0.0.0".to_owned(),
                sequence: 0,
                total: 0,
                shell: "".to_owned(),
                template: "".to_owned(),
                send_on_exit: false,
                suppress_help_headers: false,
                bypass_version_check: false,
            },
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
            compose_details: ComposeDetails {
                from: ComposeRecipient::Email("someone@example.com".to_owned()),
                to: ComposeRecipientList::Single(ComposeRecipient::Email(
                    "someone@example.com".to_owned(),
                )),
                cc: ComposeRecipientList::Multiple(Vec::new()),
                bcc: ComposeRecipientList::Multiple(Vec::new()),
                compose_type: ComposeType::New,
                related_message_id: None,
                reply_to: ComposeRecipientList::Multiple(Vec::new()),
                follow_up_to: ComposeRecipientList::Multiple(Vec::new()),
                newsgroups: Newsgroups::Multiple(Vec::new()),
                subject: "".to_owned(),
                is_plain_text: true,
                body: "".to_owned(),
                plain_text_body: "".to_owned(),
                attachments: Vec::new(),
            },
        }
    }
}
