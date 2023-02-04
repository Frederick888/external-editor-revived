#[cfg(test)]
use mockall::automock;
use webextension_native_messaging::MessagingError;

#[cfg_attr(test, automock)]
pub trait Transport {
    fn read_message<D>() -> Result<D, MessagingError>
    where
        D: 'static + for<'a> serde::Deserialize<'a>;
    fn write_message<S>(message: &S) -> Result<(), MessagingError>
    where
        S: 'static + serde::Serialize;
}

pub struct ThunderbirdTransport {}

impl Transport for ThunderbirdTransport {
    fn read_message<D>() -> Result<D, MessagingError>
    where
        D: for<'a> serde::Deserialize<'a>,
    {
        webextension_native_messaging::read_message()
    }

    fn write_message<S>(message: &S) -> Result<(), MessagingError>
    where
        S: serde::Serialize,
    {
        webextension_native_messaging::write_message(message)
    }
}
