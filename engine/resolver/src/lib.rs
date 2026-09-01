pub mod error;
pub mod grouper;
pub mod identity;
pub mod normalize;
pub mod signal;

pub use error::ResolverError;
pub use grouper::{GroupingConfig, group_candidates, resolve_applications};
pub use identity::{ApplicationIdentity, CandidateRef, ResolvedApplication};
pub use signal::{MatchSignal, SignalExtractor, extract_signals};
