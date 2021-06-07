use super::test_highlight;
use std::fmt::Write;
use std::io;
use std::io::ErrorKind;
use tree_sitter::{QueryError, QueryErrorKind};
use walkdir;

#[derive(Debug)]
pub struct Error(Option<Vec<String>>);

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn grammar(message: &str) -> Self {
        Error(Some(vec![format!("Grammar error: {}", message)]))
    }

    pub fn regex(mut message: String) -> Self {
        message.insert_str(0, "Regex error: ");
        Error(Some(vec![message]))
    }

    pub fn undefined_symbol(name: &str) -> Self {
        Error(Some(vec![format!("Undefined symbol `{}`", name)]))
    }

    pub fn new(message: String) -> Self {
        Error(Some(vec![message]))
    }

    pub fn new_ignored() -> Self {
        Self(None)
    }

    pub fn is_ignored(&self) -> bool {
        self.0.is_none()
    }

    pub fn err<T>(message: String) -> Result<T> {
        Err(Error::new(message))
    }

    pub fn wrap<E: Into<Self>, M: ToString, F: FnOnce() -> M>(
        message_fn: F,
    ) -> impl FnOnce(E) -> Self {
        |e| {
            let mut result = e.into();
            match result.0 {
                Some(ref mut e) => e.push(message_fn().to_string()),
                None => panic!("It's not allowed to wrap an ignored error"),
            }
            result
        }
    }

    pub fn message(&self) -> String {
        match self.0 {
            None => "Ignored error".to_string(),
            Some(ref e) => {
                let mut result = e.last().unwrap().clone();
                if e.len() > 1 {
                    result.push_str("\nDetails:\n");
                    for msg in e[0..e.len() - 1].iter().rev() {
                        writeln!(&mut result, "  {}", msg).unwrap();
                    }
                }
                result
            }
        }
    }
}

impl<'a> From<(&str, QueryError)> for Error {
    fn from((path, error): (&str, QueryError)) -> Self {
        let mut msg = format!("Query error at {}:{}. ", path, error.row + 1);
        match error.kind {
            QueryErrorKind::Capture => write!(&mut msg, "Invalid capture name {}", error.message),
            QueryErrorKind::Field => write!(&mut msg, "Invalid field name {}", error.message),
            QueryErrorKind::NodeType => write!(&mut msg, "Invalid node type {}", error.message),
            QueryErrorKind::Syntax => write!(&mut msg, "Invalid syntax:\n{}", error.message),
            QueryErrorKind::Structure => write!(&mut msg, "Impossible pattern:\n{}", error.message),
            QueryErrorKind::Predicate => write!(&mut msg, "Invalid predicate: {}", error.message),
        }
        .unwrap();
        Self::new(msg)
    }
}

impl<'a> From<tree_sitter_highlight::Error> for Error {
    fn from(error: tree_sitter_highlight::Error) -> Self {
        Error::new(format!("{:?}", error))
    }
}

impl<'a> From<tree_sitter_tags::Error> for Error {
    fn from(error: tree_sitter_tags::Error) -> Self {
        Error::new(format!("{}", error))
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::new(error.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        match error {
            x if x.kind() == ErrorKind::BrokenPipe => return Error::new_ignored(),
            _ => (),
        }
        Error::new(error.to_string())
    }
}

impl From<glob::PatternError> for Error {
    fn from(error: glob::PatternError) -> Self {
        Error::new(error.to_string())
    }
}

impl From<glob::GlobError> for Error {
    fn from(error: glob::GlobError) -> Self {
        Error::new(error.to_string())
    }
}

impl From<libloading::Error> for Error {
    fn from(error: libloading::Error) -> Self {
        Error::new(error.to_string())
    }
}

impl From<regex_syntax::ast::Error> for Error {
    fn from(error: regex_syntax::ast::Error) -> Self {
        Error::new(error.to_string())
    }
}

impl From<test_highlight::Failure> for Error {
    fn from(error: test_highlight::Failure) -> Self {
        Error::new(error.message())
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::new(error)
    }
}

impl From<walkdir::Error> for Error {
    fn from(error: walkdir::Error) -> Self {
        Error::new(error.to_string())
    }
}
