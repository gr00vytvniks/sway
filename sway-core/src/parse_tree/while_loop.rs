use crate::{
    build_config::BuildConfig,
    error::{ok, CompileResult, ParserLifter},
    error_recovery_exp,
    parser::Rule,
    CodeBlock, Expression,
};

use sway_types::span::Span;

use pest::iterators::Pair;

/// A parsed while loop. Contains the `condition`, which is defined from an [Expression], and the `body` from a [CodeBlock].
#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Expression,
    pub body: CodeBlock,
}
