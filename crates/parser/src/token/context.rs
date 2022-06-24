use lazy_static::lazy_static;

use crate::{
    parser::Parser,
    token::{
        types::{get_token_types, TokenType},
        TokenValue,
    },
    whitespace::get_line_break_regex,
};

fn update_paren_r_context(parser: &Parser, _: &TokenType) {
    let mut contexts = parser.context.borrow_mut();
    if parser.context.borrow().len() == 1 {
        parser.expr_allowed.set(true);
        return;
    }

    let context_types = get_token_context_types();
    if let Some(out) = contexts.pop() {
        let current_context = parser.current_context();
        if out.eq(&context_types.b_stat)
            && current_context.is_some()
            && current_context.unwrap().token.eq("function")
        {
            if let Some(out) = contexts.pop() {
                parser.expr_allowed.set(!out.is_expr);
            }
        }
    }
}

fn update_brace_r_context(parser: &Parser, prev_token_type: &TokenType) {
    update_paren_r_context(parser, prev_token_type)
}

fn update_brace_l_context(parser: &Parser, prev_token_type: &TokenType) {
    parser
        .context
        .borrow_mut()
        .push(if parser.brace_is_block(prev_token_type) {
            get_token_context_types().b_stat.clone()
        } else {
            get_token_context_types().b_expr.clone()
        });
    parser.expr_allowed.set(true);
}

fn update_dollar_brace_l_context(parser: &Parser, _: &TokenType) {
    parser
        .context
        .borrow_mut()
        .push(get_token_context_types().b_tmpl.clone());
    parser.expr_allowed.set(true);
}

fn update_paren_l_context(parser: &Parser, prev_token_type: &TokenType) {
    let token_types = get_token_types();
    let statement_parens = prev_token_type.eq(&token_types._if)
        || prev_token_type.eq(&token_types._for)
        || prev_token_type.eq(&token_types._with)
        || prev_token_type.eq(&token_types._while);
    parser.context.borrow_mut().push(if statement_parens {
        get_token_context_types().p_stat.clone()
    } else {
        get_token_context_types().p_expr.clone()
    });
    parser.expr_allowed.set(true);
}

fn update_inc_dec_context(_: &Parser, _: &TokenType) {}

fn update_function_context(parser: &Parser, prev_token_type: &TokenType) {
    let token_types = get_token_types();
    let context_types = get_token_context_types();
    let input_start_index = parser.last_token_end.get() as usize;
    let input_end_index = parser.cur_token_start.get() as usize;
    let is_input_match_line_break = get_line_break_regex()
        .is_match(&parser.input[input_start_index..input_end_index])
        .unwrap_or(false);
    let current_context = parser.current_context();
    let current_context = current_context.as_ref();
    if prev_token_type.before_expr
        && prev_token_type.ne(&token_types._else)
        && !(prev_token_type.eq(&token_types.semi)
            && current_context.ne(&Some(&context_types.p_stat)))
        && !(prev_token_type.eq(&token_types._return) && is_input_match_line_break)
        && !((prev_token_type.eq(&token_types.colon) || prev_token_type.eq(&token_types.brace_l))
            && current_context.eq(&Some(&context_types.b_stat)))
    {
        parser
            .context
            .borrow_mut()
            .push(context_types.f_expr.clone());
    } else {
        parser
            .context
            .borrow_mut()
            .push(context_types.f_stat.clone());
    }
    parser.expr_allowed.set(false);
}

fn update_class_context(parser: &Parser, prev_token_type: &TokenType) {
    update_function_context(parser, prev_token_type);
}

fn update_back_quote_context(parser: &Parser, _: &TokenType) {
    let context_types = get_token_context_types();
    let current_context = parser.current_context();
    let current_context = current_context.as_ref();
    if current_context.eq(&Some(&context_types.q_tmpl)) {
        parser.context.borrow_mut().pop();
    } else {
        parser
            .context
            .borrow_mut()
            .push(context_types.q_tmpl.clone());
    }
    parser.expr_allowed.set(false);
}

fn update_star_context(parser: &Parser, prev_token_type: &TokenType) {
    let token_types = get_token_types();
    if prev_token_type.eq(&token_types._function) {
        let mut contexts = parser.context.borrow_mut();
        let context_types = get_token_context_types();
        let index = contexts.len() - 1;
        if contexts[index].eq(&context_types.f_expr) {
            contexts[index] = context_types.f_expr_gen.clone();
        } else {
            contexts[index] = context_types.f_gen.clone();
        }
    }
    parser.expr_allowed.set(true);
}

fn update_name_context(parser: &Parser, prev_token_type: &TokenType) {
    let mut expr_allowed = false;
    if parser.options.get_ecma_version_number() >= 6 && prev_token_type.ne(&get_token_types().dot) {
        if let TokenValue::String(value) = &*parser.cur_token_value.borrow() {
            if value.eq("of") && !parser.expr_allowed.get()
                || value.eq("yield") && parser.in_generator_context()
            {
                expr_allowed = true;
            }
        }
    }
    parser.expr_allowed.set(expr_allowed);
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

pub fn get_initial_context() -> Vec<TokenContext> {
    vec![TOKEN_CONTEXT_TYPES.b_stat.clone()]
}

pub trait TokenContextParser {
    fn current_context(&self) -> Option<TokenContext>;
    fn brace_is_block(&self, prev_token_type: &TokenType) -> bool;
    fn in_generator_context(&self) -> bool;
    fn update_context(&self, prev_token_type: &TokenType);
    fn override_context(&self, token_context: &TokenContext);
}

impl TokenContextParser for Parser {
    fn current_context(&self) -> Option<TokenContext> {
        self.context.borrow().last().map(|v| v.clone())
    }

    fn brace_is_block(&self, prev_token_type: &TokenType) -> bool {
        if let Some(parent) = self.current_context() {
            let context_types = get_token_context_types();
            if parent.eq(&context_types.f_expr) || parent.eq(&context_types.f_stat) {
                return true;
            }
            let token_types = get_token_types();
            if prev_token_type.eq(&token_types.colon)
                && (parent.eq(&context_types.b_stat) || parent.eq(&context_types.b_expr))
            {
                return !parent.is_expr;
            }

            // The check for `token_types.name && exprAllowed` detects whether we are
            // after a `yield` or `of` construct. See the `update_context` for
            // `token_types.name`.
            if prev_token_type.eq(&token_types._return)
                || prev_token_type.eq(&token_types.name) && self.expr_allowed.get()
            {
                let input_start_index = self.last_token_end.get() as usize;
                let input_end_index = self.cur_token_start.get() as usize;
                return get_line_break_regex()
                    .is_match(&self.input[input_start_index..input_end_index])
                    .unwrap_or(false);
            }
            if prev_token_type.eq(&token_types._else)
                || prev_token_type.eq(&token_types.semi)
                || prev_token_type.eq(&token_types.eof)
                || prev_token_type.eq(&token_types.paren_r)
                || prev_token_type.eq(&token_types.arrow)
            {
                return true;
            }
            if prev_token_type.eq(&token_types.brace_l) {
                return parent.eq(&context_types.b_stat);
            }
            if prev_token_type.eq(&token_types._var)
                || prev_token_type.eq(&token_types._const)
                || prev_token_type.eq(&token_types.name)
            {
                return false;
            }
            !self.expr_allowed.get()
        } else {
            false
        }
    }

    fn in_generator_context(&self) -> bool {
        for context in self.context.borrow().iter() {
            if context.token.eq("function") {
                return context.generator;
            }
        }
        false
    }

    fn update_context(&self, prev_token_type: &TokenType) {
        let token_types = get_token_types();
        let current_token_type = self.cur_token_type.borrow();

        if current_token_type.keyword.is_some() && prev_token_type.eq(&token_types.dot) {
            self.expr_allowed.set(false);
        } else if current_token_type.eq(&token_types.paren_r) {
            update_paren_r_context(self, prev_token_type);
        } else if current_token_type.eq(&token_types.brace_r) {
            update_brace_r_context(self, prev_token_type);
        } else if current_token_type.eq(&token_types.brace_l) {
            update_brace_l_context(self, prev_token_type);
        } else if current_token_type.eq(&token_types.dollar_brace_l) {
            update_dollar_brace_l_context(self, prev_token_type);
        } else if current_token_type.eq(&token_types.paren_l) {
            update_paren_l_context(self, prev_token_type);
        } else if current_token_type.eq(&token_types.inc_dec) {
            update_inc_dec_context(self, prev_token_type);
        } else if current_token_type.eq(&token_types._function) {
            update_function_context(self, prev_token_type);
        } else if current_token_type.eq(&token_types._const) {
            update_class_context(self, prev_token_type);
        } else if current_token_type.eq(&token_types.back_quote) {
            update_back_quote_context(self, prev_token_type);
        } else if current_token_type.eq(&token_types.star) {
            update_star_context(self, prev_token_type);
        } else if current_token_type.eq(&token_types.name) {
            update_name_context(self, prev_token_type);
        } else {
            self.expr_allowed.set(current_token_type.before_expr);
        }
    }

    /// Used to handle egde case when token context could not be inferred
    /// correctly in tokenize phase
    fn override_context(&self, token_context: &TokenContext) {
        let need_override = match self.current_context() {
            Some(context) => context.ne(token_context),
            None => true,
        };
        if need_override {
            let mut contexts = self.context.borrow_mut();
            let index = contexts.len() - 1;
            contexts[index] = token_context.clone();
        }
    }
}
