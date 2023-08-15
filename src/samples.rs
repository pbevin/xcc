//! Builders for some common types of XCC problems.

use crate::Matrix;

/// Builds a matrix for the toy problem in equation (49)
/// of Knuth 7.2.2.1.
///
/// The problem has 3 primary items, `p`, `q`, and `r`, and 2 secondary items,
/// `x` and `y`.  The options are:
/// - `p q x y:A`
/// - `p r x:A y`
/// - `p x:B`
/// - `q x:A`
/// - `r y:B`
///
/// # Example
///
/// ```
/// use xcc::samples::toy;
///
/// let mut matrix = toy();
/// let solutions = matrix.solve_all();
/// assert_eq!(solutions.len(), 1);
/// ```
pub fn toy() -> Matrix<usize> {
    let mut builder = Matrix::builder();
    builder.add_primary_items(["p", "q", "r"]);
    builder.add_secondary_items(["x", "y"]);
    builder.add_option(1, ["p", "q", "x", "y:A"]);
    builder.add_option(2, ["p", "r", "x:A", "y"]);
    builder.add_option(3, ["p", "x:B"]);
    builder.add_option(4, ["q", "x:A"]);
    builder.add_option(5, ["r", "y:B"]);
    builder.build()
}
