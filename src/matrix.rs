use super::Solution;
use crate::Builder;
use crate::ColoredItem;
use crate::Unique;
use fixedbitset::FixedBitSet;
use std::collections::HashMap;

/// A compiled specification of an exact cover problem with colored items.
///
/// To build a matrix, use the `builder()` method:
/// ```
/// use xcc::Matrix;
/// let mut builder = Matrix::builder();
/// builder.add_primary_item("a");
/// builder.add_option(1, ["a"]);
/// let matrix = builder.build();
/// ```
///
#[derive(Debug)]
pub struct Matrix<T> {
    num_items: usize,
    num_primary_items: usize,
    options: Vec<OptionData<T>>,
}

impl<T> Matrix<T> {
    pub fn num_items(&self) -> usize {
        self.num_items
    }

    pub fn num_primary_items(&self) -> usize {
        self.num_primary_items
    }

    /// Solves the exact cover problem represented by this matrix, returning all solutions.
    ///
    /// # Example
    ///
    /// ```
    /// let mut matrix = xcc::samples::toy();
    /// let solutions = matrix.solve_all();
    /// assert_eq!(solutions.len(), 1);
    /// assert_eq!(solutions[0].option_numbers(&matrix), [3, 1]);
    /// ```
    pub fn solve_all(&mut self) -> Vec<Solution> {
        let mut solver = super::Solver::new(self);
        solver.solve_all()
    }

    /// Solves the matrix, returning a unique solution if there is one, or `Unique::Ambiguous` if there are multiple
    /// solutions. If there are no solutions, `Unique::None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use xcc::{Matrix, Unique};
    ///
    /// // The following problem is ambiguous, because options
    /// // a and b are identical.
    /// let mut builder = Matrix::builder();
    /// builder.add_primary_item("x");
    /// builder.add_primary_item("y");
    /// builder.add_option("a", vec!["x", "y"]);
    /// builder.add_option("b", vec!["x", "y"]);
    /// let mut matrix = builder.build();
    /// assert!(matrix.solve_unique().is_ambiguous());
    /// ```
    ///
    pub fn solve_unique(&mut self) -> Unique<Solution> {
        let mut solver = super::Solver::new(self);
        solver.solve_unique()
    }

    pub fn solve_once(&mut self) -> Option<Solution> {
        let mut solver = super::Solver::new(self);
        solver.solve_once()
    }

    /// Creates a `Builder` to configure a matrix.
    ///
    /// # Example
    ///
    /// ```
    /// use xcc::Matrix;
    /// let mut builder = Matrix::builder();
    /// builder.add_primary_item("a");
    /// builder.add_option(1, ["a"]);
    /// let matrix = builder.build();
    ///
    pub fn builder() -> Builder<T> {
        Builder::default()
    }

    /// Low-level constructor. You almost certainly want to use the
    /// `builder()` method instead of this.
    ///
    /// Creates a new matrix for DLX with the given number of primary and secondary items.
    pub fn new(num_primary_items: usize, num_secondary_items: usize) -> Self {
        let num_items = num_primary_items + num_secondary_items;
        Matrix {
            num_items,
            num_primary_items,
            options: vec![],
        }
    }

    /// Adds an option (row) to the DLX instance.
    pub fn add_option(&mut self, items: &[ColoredItem], meaning: T) {
        let mut items_bitset = FixedBitSet::with_capacity(self.num_items);
        for ci in items {
            items_bitset.insert(ci.item());
        }

        let colors: HashMap<usize, usize> = items
            .iter()
            .filter_map(|ci| ci.color().map(|color| (ci.item(), color)))
            .collect();

        self.options.push(OptionData {
            option_id: self.options.len(),
            items: items_bitset,
            colors,
            meaning,
        });
    }

    pub fn meaning(&self, option_number: usize) -> &T {
        &self.options[option_number].meaning
    }

    pub fn options_for_item(&self, item: usize) -> impl Iterator<Item = &OptionData<T>> + '_ {
        self.options
            .iter()
            .filter(move |option| option.items.contains(item))
    }

    pub fn item_counts(&self) -> Vec<usize> {
        let mut counts = vec![0; self.num_items];
        for option in &self.options {
            for item in option.items.ones() {
                counts[item] += 1;
            }
        }
        counts
    }

    pub fn num_options(&self) -> usize {
        self.options.len()
    }

    pub fn get_option(&self, n: usize) -> &OptionData<T> {
        &self.options[n]
    }

    pub fn items_for_option(
        &self,
        option: usize,
    ) -> impl Iterator<Item = (usize, Option<usize>)> + '_ {
        let opt = &self.options[option];
        opt.items.ones().map(move |item| {
            for (colored_item, color) in opt.colors.iter() {
                if *colored_item == item {
                    return (item, Some(*color));
                }
            }
            (item, None)
        })
    }
}

#[derive(Debug)]
pub struct OptionData<T> {
    // The option number (row number) in the matrix.
    pub option_id: usize,
    // The items (primary and secondary) that take part in this option.
    pub items: FixedBitSet,
    // Map from item ID to color for colored items in this option.
    pub colors: HashMap<usize, usize>,
    // The user-defined meaning of this option.
    pub meaning: T,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! hashmap {
        ($($x:expr => $y:expr),*) => {{
            let mut map = HashMap::new();
            $(
                map.insert($x, $y);
            )*
            map
        }};
    }

    fn to_vec(set: &FixedBitSet) -> Vec<usize> {
        set.ones().collect()
    }

    #[test]
    fn test_init_no_colors() {
        // This is the example shown in Table 1 (page 68):
        let mut builder = Matrix::builder();
        builder.add_primary_items(["a", "b", "c", "d", "e", "f", "g"]);
        builder.add_option(Some(0), ["c", "e"]);
        builder.add_option(Some(1), vec!["a", "d", "g"]);
        builder.add_option(Some(2), vec!["b", "c", "f"]);
        builder.add_option(Some(3), vec!["a", "d", "f"]);
        builder.add_option(Some(4), vec!["b", "g"]);
        builder.add_option(Some(5), vec!["d", "e", "g"]);

        let matrix = builder.build();

        assert_eq!(to_vec(&matrix.options[0].items), [2, 4]);
        assert_eq!(to_vec(&matrix.options[1].items), [0, 3, 6]);
        assert_eq!(to_vec(&matrix.options[2].items), [1, 2, 5]);
        assert_eq!(to_vec(&matrix.options[3].items), [0, 3, 5]);
        assert_eq!(to_vec(&matrix.options[4].items), [1, 6]);
        assert_eq!(to_vec(&matrix.options[5].items), [3, 4, 6]);
    }

    #[test]
    fn test_colored_items() {
        // p q x y:A
        // p r x:A y
        // p x:B
        // q x:A
        // r y:B
        let mut builder = Matrix::builder();

        builder.add_primary_items(["p", "q", "r"]);
        builder.add_secondary_items(["x", "y"]);
        builder.add_option("p q x y:A", ["p", "q", "x", "y:A"]);
        builder.add_option("p r x:A y", ["p", "r", "x:A", "y"]);
        builder.add_option("p x:B", ["p", "x:B"]);
        builder.add_option("q x:A", ["q", "x:A"]);
        builder.add_option("r y:B", ["r", "y:B"]);
        let mut matrix = builder.build();

        assert_eq!(to_vec(&matrix.options[0].items), [0, 1, 3, 4]);
        assert_eq!(matrix.options[0].colors, hashmap![4 => 0]);

        assert_eq!(to_vec(&matrix.options[1].items), [0, 2, 3, 4]);
        assert_eq!(matrix.options[1].colors, hashmap![3 => 0]);

        assert_eq!(to_vec(&matrix.options[2].items), [0, 3]);
        assert_eq!(matrix.options[2].colors, hashmap![3 => 1]);

        assert_eq!(to_vec(&matrix.options[3].items), [1, 3]);
        assert_eq!(matrix.options[3].colors, hashmap![3 => 0]);

        assert_eq!(to_vec(&matrix.options[4].items), [2, 4]);
        assert_eq!(matrix.options[4].colors, hashmap![4 => 1]);

        let solutions = matrix
            .solve_all()
            .into_iter()
            .map(|s| s.meanings(&matrix))
            .collect::<Vec<_>>();

        assert_eq!(solutions, [[&"q x:A", &"p r x:A y"]]);
    }
}
