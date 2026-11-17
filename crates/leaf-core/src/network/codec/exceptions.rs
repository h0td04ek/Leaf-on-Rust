//! Netty-аналоги `io.netty.handler.codec.DecoderException` /
//! `EncoderException` — ошибки кодирования пакетов.

/// `DecoderException`.
#[derive(Debug)]
pub struct DecoderException {
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl DecoderException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), source: None }
    }
    pub fn with_cause(cause: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self { message: cause.to_string(), source: Some(Box::new(cause)) }
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + Send + Sync)> {
        self.source.as_deref()
    }
}

impl std::fmt::Display for DecoderException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for DecoderException {}

/// `EncoderException`.
#[derive(Debug)]
pub struct EncoderException {
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl EncoderException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), source: None }
    }
    pub fn with_cause(cause: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self { message: cause.to_string(), source: Some(Box::new(cause)) }
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for EncoderException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for EncoderException {}
