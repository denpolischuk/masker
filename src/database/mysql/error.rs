use std::fmt::Display;

#[derive(Debug)]
enum AdapterErrorKind {
    MissmatchedEntries,
    NothinToMask,
}

impl Display for AdapterErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AdapterErrorKind::MissmatchedEntries => "MissmatchedEntries",
                AdapterErrorKind::NothinToMask => "NothinToMask",
            }
        )
    }
}

#[derive(Debug)]
struct AdapterError {
    kind: AdapterErrorKind,
}

impl Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl std::error::Error for AdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }

    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {}
}
