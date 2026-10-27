use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Diagnostic>;

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("{code}: {message}")]
pub struct Diagnostic {
    pub code: String,
    pub message: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub injection_id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<Box<str>>,
}
impl Diagnostic {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into().into_boxed_str(),
            file: None,
            mod_id: None,
            injection_id: None,
            target: None,
            selector: None,
        }
    }
    pub fn file(mut self, file: impl AsRef<std::path::Path>) -> Self {
        self.file = Some(file.as_ref().display().to_string().into());
        self
    }
    pub fn injection(mut self, mod_id: &str, injection: &crate::manifest::Injection) -> Self {
        self.mod_id = Some(mod_id.into());
        self.injection_id = Some(injection.id.clone().into());
        self.target = Some(injection.target.clone().into());
        self.selector = injection.selector.as_deref().map(Into::into);
        self
    }
}
pub fn io(error: std::io::Error) -> Diagnostic {
    Diagnostic::new("E_IO", error.to_string())
}
