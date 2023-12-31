use super::Matrix;
use crate::{
    types::{Color, ItemId, OptionId},
    Unique,
};
use fixedbitset::FixedBitSet;
use std::collections::HashMap;

/// A solver for an exact cover problem with colored secondary items.
pub struct Solver<'a, T> {
    matrix: &'a Matrix<T>,
    /// Bitmask of items that can still be used.
    available_items: FixedBitSet,
    /// Bitmask of options that can still be used.
    available_options: FixedBitSet,
    /// Map of item => color that we have committed to
    committed_colors: HashMap<ItemId, Color>,
}

impl<'a, T> Solver<'a, T> {
    /// Creates a new solver for the given matrix.
    ///
    /// # Example
    ///
    /// ```
    /// use xcc::samples::toy;
    /// let matrix = toy();
    /// let solver = xcc::Solver::new(&matrix);
    /// ```
    #[must_use]
    pub fn new(matrix: &'a Matrix<T>) -> Self {
        let mut available_items = FixedBitSet::with_capacity(matrix.num_items());
        available_items.set_range(0..matrix.num_items(), true);
        let mut available_options = FixedBitSet::with_capacity(matrix.num_options());
        available_options.set_range(0..matrix.num_options(), true);
        Self {
            matrix,
            available_items,
            available_options,
            committed_colors: HashMap::new(),
        }
    }

    /// Solves the exact cover problem represented by this matrix, returning all
    /// solutions.
    pub fn solve_all(&mut self) -> Vec<Solution> {
        self.solve(Limit::All)
    }

    /// Solves the exact cover problem represented by this matrix, searching for
    /// up to two solutions.  If no solutions are found, returns `None`.  If one
    /// solution is found, returns `One(solution)`.  If two solutions are found,
    /// returns `Ambiguous(s1, s2)`.
    pub fn solve_unique(&mut self) -> Unique<Solution> {
        let mut solutions = self.solve(Limit::Max(2));
        let s1 = solutions.pop();
        let s2 = solutions.pop();

        match (s1, s2) {
            (Some(s1), Some(s2)) => Unique::Ambiguous(s1, s2),
            (Some(s1), None) => Unique::One(s1),
            (None, Some(_)) => unreachable!(),
            (None, None) => Unique::None,
        }
    }

    /// Solves the exact cover problem represented by this matrix, returning the
    /// first solution found.
    pub fn solve_once(&mut self) -> Option<Solution> {
        self.solve(Limit::Max(1)).pop()
    }

    /// Stack-based solver for the exact cover problem.
    ///
    /// # Arguments
    ///
    /// * `max_solutions` - The maximum number of solutions to return.  If `None`,
    ///  all solutions will be returned.
    ///
    /// # Returns
    ///
    /// A vector of solutions.  If `max_solutions` is `None`, this will be all
    /// solutions.  If `max_solutions` is `Some(n)`, this will be at most `n`
    /// solutions.
    ///
    /// # Example
    ///
    /// ```
    /// use xcc::samples::toy;
    /// use xcc::{Limit, Solver};
    ///
    /// let matrix = toy();
    /// let mut solver = Solver::new(&matrix);
    /// let solutions = solver.solve(Limit::All);
    /// assert_eq!(solutions.len(), 1);
    /// ```
    pub fn solve(&mut self, limit: Limit) -> Vec<Solution> {
        let mut results = Vec::new();
        let mut stack: Vec<(SavedState, Vec<OptionId>)> = vec![(self.save_state(), Vec::new())];

        while let Some((state, mut solution)) = stack.pop() {
            self.restore(state);
            match self.choose_next_item() {
                None => {
                    // We have a solution! Decode it and add it to the results.
                    let cells = solution.clone();
                    results.push(Solution { option_ids: cells });
                    if limit.reached(results.len()) {
                        break;
                    }
                }
                Some(item) => {
                    self.available_items.set(item.index(), false);
                    let option_ids = self.cover_item_and_its_options(item);

                    // We just covered some options, and now we're going to go
                    // through them one by one, and push the resulting states
                    // onto the stack.
                    let ss = self.save_state();
                    for option in option_ids {
                        self.restore(ss.clone());
                        self.commit(option);
                        solution.push(option);
                        let saved_state = self.save_state();
                        stack.push((saved_state, solution.clone()));
                        solution.pop();
                    }
                }
            }
        }

        results
    }

    /// Makes a provisional commitment to an option.
    fn commit(&mut self, option_id: OptionId) {
        let items: Vec<_> = self
            .matrix
            .items_for_option(option_id)
            .filter(|&(item, _)| self.available_items.contains(item.index()))
            .collect();
        for (item, color) in items {
            match color {
                None => {
                    self.cover_item_and_its_options(item);
                }
                Some(color) => {
                    if !self.committed_colors.contains_key(&item) {
                        self.purify(item, color);
                    }
                }
            }
            self.available_items.set(item.index(), false);
        }
    }

    /// Hide all visible options containing a given item, and return the option IDs.
    fn cover_item_and_its_options(&mut self, item_num: ItemId) -> Vec<OptionId> {
        let mut covered_options = Vec::new();
        for option in self.matrix.options_for_item(item_num) {
            if self.available_options.contains(option.option_id.index()) {
                self.available_options.set(option.option_id.index(), false);
                covered_options.push(option.option_id);
            }
        }
        self.available_items.set(item_num.index(), false);
        covered_options
    }

    /// Given the index of a secondary item cell plus its color, go through other
    /// options that contain that item and either mark the color as "known correct",
    /// or if the color is different, hide the option.
    ///
    /// This method is called during the commit process when the solver first assigns a color
    /// to a secondary item.
    fn purify(&mut self, item_num: ItemId, item_color: Color) {
        for option in self.matrix.options_for_item(item_num) {
            if option.colors.get(&item_num) == Some(&item_color) {
                self.committed_colors.insert(item_num, item_color);
            } else {
                self.available_options.set(option.option_id.index(), false);
            }
        }
    }

    /// Finds the uncovered primary item with the fewest remaining options, and
    /// returns its index.
    #[must_use]
    fn choose_next_item(&self) -> Option<ItemId> {
        let item_counts = self.count_items();
        self.available_items
            .ones()
            .take_while(|&i| i < self.matrix.num_primary_items())
            .min_by_key(|&i| item_counts[i])
            .map(ItemId::new)
    }

    /// Counts the number of available options for each available item.
    /// This is used to choose the next item to visit.
    #[must_use]
    fn count_items(&self) -> Vec<usize> {
        let mut item_counts = vec![0; self.matrix.num_items()];
        for option in self.available_options.ones().map(OptionId::new) {
            for (item, _) in self.matrix.items_for_option(option) {
                item_counts[item.index()] += 1;
            }
        }
        item_counts
    }

    fn save_state(&self) -> SavedState {
        SavedState {
            available_items: self.available_items.clone(),
            available_options: self.available_options.clone(),
            known_correct: self.committed_colors.clone(),
        }
    }

    fn restore(&mut self, state: SavedState) {
        self.available_items = state.available_items;
        self.available_options = state.available_options;
        self.committed_colors = state.known_correct;
    }
}

/// A limit on the number of solutions to return. This is used by
/// `Matrix::solve()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    /// Return at most this many solutions.
    Max(usize),
    /// Return all solutions.
    All,
}
impl Limit {
    /// Returns true if the limit has been reached.
    #[must_use]
    pub fn reached(&self, len: usize) -> bool {
        match self {
            Limit::Max(n) => len >= *n,
            Limit::All => false,
        }
    }
}

/// A solution to an exact cover problem.
///
/// The usual way to use this is to call `Matrix::solve_all()`, then for each
/// Solution returned, call `meanings()` to get the meanings of the options.
///
/// # Example
///
/// ```
/// let mut toy = xcc::samples::toy();
/// toy.solve_all().into_iter().for_each(|solution| {
///    println!("Solution: {:?}", solution.meanings(&toy));
/// });
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
    option_ids: Vec<OptionId>,
}

impl Solution {
    /// Returns the meanings of the options in this solution.  The meanings
    /// come from the parameter to `Builder::add_option()`.
    ///
    /// # Example
    ///
    /// ```
    /// use xcc::Matrix;
    /// let mut builder = Matrix::builder();
    /// builder.add_primary_items(["p", "q", "r"]);
    /// builder.add_secondary_items(["x", "y"]);
    /// builder.add_option("option one", ["p", "q", "x", "y:A"]);
    /// builder.add_option("option two", ["p", "r", "x:A", "y"]);
    /// builder.add_option("option three", ["p", "x:B"]);
    /// builder.add_option("option four", ["q", "x:A"]);
    /// builder.add_option("option five", ["r", "y:B"]);
    /// let mut matrix = builder.build().expect("could not build matrix");
    /// let solution = matrix.solve_all().pop().unwrap();
    /// assert_eq!(solution.meanings(&matrix), [&"option four", &"option two"]);
    /// ```
    #[must_use]
    pub fn meanings<'a, T>(&self, matrix: &'a Matrix<T>) -> Vec<&'a T> {
        self.option_ids
            .iter()
            .map(|&i| &matrix.get_option(i).meaning)
            .collect()
    }
}

#[derive(Clone)]
pub struct SavedState {
    available_items: FixedBitSet,
    available_options: FixedBitSet,
    known_correct: HashMap<ItemId, Color>,
}

impl std::fmt::Debug for SavedState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let available_items = self.available_items.ones().collect::<Vec<_>>();
        let available_options = self.available_options.ones().collect::<Vec<_>>();
        f.debug_struct("SavedState")
            .field("available_items", &available_items)
            .field("available_options", &available_options)
            .field("known_correct", &self.known_correct)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choose_next_item() {
        let mut builder = Matrix::builder();
        builder.add_primary_items(["a", "b", "c", "d"]);
        builder.add_option(1, ["a", "b"]);
        builder.add_option(2, ["a", "c"]);
        builder.add_option(3, ["a", "d"]);
        builder.add_option(4, ["b", "d"]);

        let matrix = builder.build().unwrap();
        let solver = Solver::new(&matrix);
        assert_eq!(solver.count_items(), [3, 2, 1, 2]);
        assert_eq!(
            solver.choose_next_item(),
            Some(ItemId::new(2)),
            "c should be chosen because it has the lowest count"
        );
    }

    #[test]
    fn test_simple_solve() {
        let mut builder = Matrix::builder();
        builder.add_primary_item("a");
        builder.add_primary_item("b");
        builder.add_option(1, ["a"]);
        builder.add_option(2, ["b"]);

        let mut matrix = builder.build().unwrap();
        let solutions = matrix
            .solve_all()
            .into_iter()
            .map(|s| s.meanings(&matrix))
            .collect::<Vec<_>>();
        assert_eq!(solutions, [vec![&1, &2]]);
    }

    #[test]
    fn test_simple_colored() {
        let mut builder = Matrix::builder();
        builder.add_primary_item("a");
        builder.add_primary_item("b");
        builder.add_secondary_item("c");
        builder.add_option(1, ["a", "c:1"]);
        builder.add_option(2, ["b", "c:2"]);
        builder.add_option(3, ["a", "b", "c:3"]);

        let mut matrix = builder.build().unwrap();
        let solutions = matrix
            .solve_all()
            .into_iter()
            .map(|s| s.meanings(&matrix))
            .collect::<Vec<_>>();

        // The only way to get both a and b is to take option 3.  In particular,
        // the solution cannot be [1, 2] because that would c to have two different
        // colors.
        assert_eq!(
            solutions.as_slice(),
            [vec![&3]],
            "Should only have [3] as a solution"
        );
    }
}
