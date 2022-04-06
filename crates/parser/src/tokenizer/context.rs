use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct TokenContext {
    pub token: String,
    pub is_expr: bool,
    pub preserve_space: bool,
    pub generator: bool,
}

impl TokenContext {
    pub fn new(token: &str, is_expr: bool, preserve_space: bool, generator: bool) -> Self {
        TokenContext {
            token: token.to_owned(),
            is_expr,
            preserve_space,
            generator,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenContextTypes {
    b_stat: TokenContext,
    b_expr: TokenContext,
    b_tmpl: TokenContext,
    p_stat: TokenContext,
    p_expr: TokenContext,
    q_tmpl: TokenContext,
    f_stat: TokenContext,
    f_expr: TokenContext,
    f_expr_gen: TokenContext,
    f_gen: TokenContext,
}

lazy_static! {
    static ref TOKEN_CONTEXT_TYPES: TokenContextTypes = TokenContextTypes {
        b_stat: TokenContext::new("{", false, false, false),
        b_expr: TokenContext::new("{", true, false, false),
        b_tmpl: TokenContext::new("${", false, false, false),
        p_stat: TokenContext::new("(", false, false, false),
        p_expr: TokenContext::new("(", true, false, false),
        q_tmpl: TokenContext::new("`", true, true, false),
        f_stat: TokenContext::new("function", false, false, false),
        f_expr: TokenContext::new("function", true, false, false),
        f_expr_gen: TokenContext::new("function", true, false, true),
        f_gen: TokenContext::new("function", false, false, true),
    };
}

pub fn get_token_context_types() -> &'static TokenContextTypes {
    &(*TOKEN_CONTEXT_TYPES)
}
