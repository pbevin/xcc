//! Colored Exact Cover Solver
//!
//! This is a Rust implementation of Exact Cover, with the addition of the
//! ability to color secondary items. The algorithm is based on Donald Knuth's
//! Algorithm C, as described in _The Art of Computer Programming_, Volume 4B,
//! under "Color-controlled covering".
//!
//! The solver takes:
//! * a set of _primary items_;
//! * a set of _secondary items_;
//! * a set of _options_, which are subsets of the primary and secondary items.
//!
//! The solver's job is to find a subset of the options that
//! * includes each primary item once and only once, and
//! * colors each secondary item consistently.
//!
//! Options can contain secondary items with or without colors.  If a secondary
//! item has no color, then the solver will not use it more than once (so that
//! it defines a "zero or one" constraint).  If an option has a secondary item
//! with a color, then the solver can use it _with the same color_ as many times
//! as it wants, but not uncolored or with a different color.
//!
//! The solver can be used to solve many different kinds of problems:
//! - Sudoku-like puzzles
//! - Shape puzzles, such as "tile a 6x10 rectangle with the 12 pentominos"
//! - Word puzzles, such as "fill a 5x4 grid with words from a dictionary"
//! - Most Nikoli puzzles
//! - Graph coloring
//! - Scheduling
//! - Many more!
//!
//! There are many examples in the `examples` directory.
//!

mod builder;
mod color;
mod matrix;
pub mod samples;
mod solver;
mod unique;

pub use self::builder::Builder;
pub use self::color::ColoredItem;
pub use self::matrix::Matrix;
pub use self::solver::Solution;
pub use self::solver::Solver;
pub use self::unique::Unique;
