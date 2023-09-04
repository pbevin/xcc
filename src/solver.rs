use super::Matrix;
use crate::Unique;
use fixedbitset::FixedBitSet;
use std::collections::HashMap;

pub struct Solver<'a, T> {
    matrix: &'a Matrix<T>,
    /// Bitmask of items that can still be used.
    available_items: FixedBitSet,
    /// Bitmask of options that can still be used.
    available_options: FixedBitSet,
    /// Map of item => color that we have committed to
    committed_colors: HashMap<usize, usize>,
}

impl<'a, T> Solver<'a, T> {
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

    pub fn solve_all(&mut self) -> Vec<Solution> {
        self.solve(None)
    }

    pub fn solve_unique(&mut self) -> Unique<Solution> {
        let mut solutions = self.solve(Some(2));
        let s1 = solutions.pop();
        let s2 = solutions.pop();

        match (s1, s2) {
            (Some(s1), Some(s2)) => Unique::Ambiguous(s1, s2),
            (Some(s1), None) => Unique::One(s1),
            (None, Some(_)) => unreachable!(),
            (None, None) => Unique::None,
        }
    }

    pub fn solve_once(&mut self) -> Option<Solution> {
        self.solve(Some(1)).pop()
    }

    /// Stack-based solver for the exact cover problem.
    pub fn solve(&mut self, max_solutions: Option<usize>) -> Vec<Solution> {
        let mut results = Vec::new();
        let mut stack: Vec<(SavedState, Vec<usize>)> = vec![(self.save_state(), Vec::new())];

        while let Some((state, mut solution)) = stack.pop() {
            self.restore(state);
            match self.choose_next_item() {
                None => {
                    // All visited
                    let cells = solution.to_vec();
                    results.push(Solution { cells });
                    if results.len() == max_solutions.unwrap_or(usize::MAX) {
                        break;
                    }
                }
                Some(item) => {
                    self.available_items.set(item, false);
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
    pub fn commit(&mut self, option_id: usize) {
        let items: Vec<_> = self
            .matrix
            .items_for_option(option_id)
            .filter(|&(item, _)| self.available_items.contains(item))
            .collect();
        for (item, color) in items {
            match color {
                None => {
                    self.cover_item_and_its_options(item);
                }
                Some(color) => {
                    if !self.committed_colors.contains_key(&item) {
                        self.purify(item, color)
                    }
                }
            }
            self.available_items.set(item, false);
        }
    }

    // Hide all visible options containing a given item, and return the option IDs.
    pub fn cover_item_and_its_options(&mut self, item_num: usize) -> Vec<usize> {
        let mut covered_options = Vec::new();
        for option in self.matrix.options_for_item(item_num) {
            if self.available_options.contains(option.option_id) {
                self.available_options.set(option.option_id, false);
                covered_options.push(option.option_id);
            }
        }
        self.available_items.set(item_num, false);
        covered_options
    }

    /// Given the index of a secondary item cell plus its color, go through other
    /// options that contain that item and either mark the color as "known correct",
    /// or if the color is different, hide the option.
    ///
    /// This method is called during the commit process when the solver first assigns a color
    /// to a secondary item.
    pub fn purify(&mut self, item_num: usize, item_color: usize) {
        for option in self.matrix.options_for_item(item_num) {
            if option.colors.get(&item_num) == Some(&item_color) {
                self.committed_colors.insert(item_num, item_color);
            } else {
                self.available_options.set(option.option_id, false);
            }
        }
    }

    /// Finds the uncovered primary item with the fewest remaining options, and
    /// returns its index.
    pub fn choose_next_item(&self) -> Option<usize> {
        let item_counts = self.count_items();
        self.available_items
            .ones()
            .take_while(|&i| i < self.matrix.num_primary_items())
            .min_by_key(|&i| item_counts[i])
    }

    /// Counts the number of available options for each available item.
    /// This is used to choose the next item to visit.
    pub fn count_items(&self) -> Vec<usize> {
        let mut item_counts = vec![0; self.matrix.num_items()];
        for option in self.available_options.ones() {
            for (item, _) in self.matrix.items_for_option(option) {
                item_counts[item] += 1;
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
    cells: Vec<usize>,
}

impl Solution {
    /// Returns the option numbers for the options in this solution. These are
    /// the 0-based indices of the options in the order they were originally added.
    /// ```
    /// use xcc::Matrix;
    /// let mut builder = Matrix::builder();
    /// builder.add_primary_items(["p", "q", "r"]);
    /// builder.add_secondary_items(["x", "y"]);
    /// builder.add_option(1, ["p", "q", "x", "y:A"]);
    /// builder.add_option(2, ["p", "r", "x:A", "y"]);
    /// builder.add_option(3, ["p", "x:B"]);
    /// builder.add_option(4, ["q", "x:A"]);
    /// builder.add_option(5, ["r", "y:B"]);
    /// let mut matrix = builder.build();
    /// let solution = matrix.solve_all().pop().unwrap();
    /// assert_eq!(solution.option_numbers(&matrix), [3, 1]);
    /// ```
    pub fn option_numbers<T>(&self, matrix: &Matrix<T>) -> Vec<usize> {
        self.cells
            .iter()
            .map(|&i| matrix.get_option(i).option_id)
            .collect()
    }

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
    /// let mut matrix = builder.build();
    /// let solution = matrix.solve_all().pop().unwrap();
    /// assert_eq!(solution.meanings(&matrix), [&"option four", &"option two"]);
    /// ```
    pub fn meanings<'a, T>(&self, matrix: &'a Matrix<T>) -> Vec<&'a T> {
        self.cells
            .iter()
            .map(|&i| &matrix.get_option(i).meaning)
            .collect()
    }
}

#[derive(Clone)]
pub struct SavedState {
    available_items: FixedBitSet,
    available_options: FixedBitSet,
    known_correct: HashMap<usize, usize>,
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

        let matrix = builder.build();
        let solver = Solver::new(&matrix);
        assert_eq!(matrix.item_counts(), [3, 2, 1, 2]);
        assert_eq!(
            solver.choose_next_item(),
            Some(2),
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

        let mut matrix = builder.build();
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

        let mut matrix = builder.build();
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
