mod matcher;
mod normalizer;

pub(crate) use matcher::find_matches;
pub(crate) use normalizer::{normalize_symbols, NormalizedText, SourceSymbol};
