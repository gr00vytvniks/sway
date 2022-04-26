use crate::{build_config::BuildConfig, error::*, parser::Rule, CatchAll, CodeBlock};

use sway_types::{span, Span};

use pest::iterators::Pair;

use super::scrutinee::Scrutinee;
use super::{Expression, MatchCondition};

#[derive(Debug, Clone)]
pub struct MatchBranch {
    pub(crate) condition: MatchCondition,
    pub(crate) result: Expression,
    pub(crate) span: span::Span,
}
