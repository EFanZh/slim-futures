pub use self::fold_state::FoldState;
#[allow(clippy::redundant_pub_crate)]
pub(crate) use self::fold_state::FoldStateProject;
pub use self::predicate_state::PredicateState;
#[allow(clippy::redundant_pub_crate)]
pub(crate) use self::predicate_state::{PredicateStateProject, PredicateStateReplace};
pub use self::two_phases::TwoPhases;

mod fold_state;
mod predicate_state;
mod two_phases;
