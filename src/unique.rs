/// The value returned from `DLX::solve_unique`.
///
/// There is a large class of problems that involve searching for a puzzle with
/// a unique solution, and this enum is used to return the result of such a
/// search. It distinguishes three cases:
///
/// - `Unique::None` means that the problem is unsolvable.
/// - `Unique::One(solution)` means that the problem has exactly one solution:
///    after finding it, the solver exhaustively searched for other solutions,
///    and found none.
/// - `Unique::Many(solution1, solution2)` means that the problem has at least
///     two solutions, and two such solutions are `solution1` and `solution2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unique<T> {
    /// The problem is unsolvable.
    None,
    /// The problem has a unique solution, which is `T`.
    One(T),
    /// The problem has at least two distinct solutions.
    Ambiguous(T, T),
}

impl<T> Unique<T> {
    /// Returns the unique value, if there is one.  If there are multiple values,
    /// it returns `None`.
    pub fn unique(&self) -> Option<&T> {
        if let Unique::One(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns `true` if there is a unique solution.
    pub fn is_unique(&self) -> bool {
        matches!(self, Unique::One(_))
    }

    /// Returns `true` if there were multiple solutions to the problem.
    pub fn is_ambiguous(&self) -> bool {
        matches!(self, Unique::Ambiguous(_, _))
    }

    /// Transforms the `Unique` value by applying a function to its contained value.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes a value of type `T` and returns a value of type `U`.
    ///
    /// # Returns
    ///
    /// A `Unique` instance that contains the result of applying `f` to the original value.
    pub fn map<U>(self, f: impl Fn(T) -> U) -> Unique<U> {
        match self {
            Unique::None => Unique::None,
            Unique::One(s) => Unique::One(f(s)),
            Unique::Ambiguous(s1, s2) => Unique::Ambiguous(f(s1), f(s2)),
        }
    }
}
