#[derive(Debug)]
pub(crate) struct TokenContext {
    pub(crate) token: String,
    pub(crate) preserve_space: bool,
}

impl TokenContext {
    pub(crate) fn new(token: &str) -> Self {
        TokenContext {
            token: token.to_owned(),
            preserve_space: false,
        }
    }
}
