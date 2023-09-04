use super::Solution;
use crate::types::{Color, ItemId, OptionId};
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
    /// Returns the number of options (rows) in the matrix.
    #[must_use]
    pub fn num_options(&self) -> usize {
        self.options.len()
    }

    /// Returns the option with the given number.
    #[must_use]
    pub fn get_option(&self, n: OptionId) -> &OptionData<T> {
        &self.options[n.index()]
    }

    /// Returns the number of items (columns) in the matrix.
    #[must_use]
    pub fn num_items(&self) -> usize {
        self.num_items
    }

    /// Returns the number of primary items (columns) in the matrix. These are
    /// the items that are required to be present in every solution.
    #[must_use]
    pub fn num_primary_items(&self) -> usize {
        self.num_primary_items
    }

    /// Solves the exact cover problem represented by this matrix, returning all
    /// solutions.
    ///
    /// # Example
    ///
    /// ```
    /// let mut matrix = xcc::samples::toy();
    /// let solutions = matrix.solve_all();
    /// assert_eq!(solutions.len(), 1);
    /// assert_eq!(solutions[0].meanings(&matrix), [&4, &2]);
    /// ```
    pub fn solve_all(&mut self) -> Vec<Solution> {
        let mut solver = super::Solver::new(self);
        solver.solve_all()
    }

    /// Solves the matrix, returning a unique solution if there is one, or
    /// `Unique::Ambiguous` if there are multiple solutions. If there are no
    /// solutions, `Unique::None` is returned.
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
    /// let mut matrix = builder.build().expect("could not build matrix");
    /// assert!(matrix.solve_unique().is_ambiguous());
    /// ```
    ///
    pub fn solve_unique(&mut self) -> Unique<Solution> {
        let mut solver = super::Solver::new(self);
        solver.solve_unique()
    }

    /// Solves the matrix, returning the first solution found, or `None` if
    /// there are no solutions.
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
    #[must_use]
    pub fn builder() -> Builder<T> {
        Builder::default()
    }

    /// Low-level constructor. You almost certainly want to use the
    /// `builder()` method instead of this.
    ///
    /// Creates a new matrix for DLX with the given number of primary and secondary items.
    #[must_use]
    pub fn new(num_primary_items: usize, num_secondary_items: usize) -> Self {
        let num_items = num_primary_items + num_secondary_items;
        Matrix {
            num_items,
            num_primary_items,
            options: vec![],
        }
    }

    /// Adds an option (row) to the DLX instance, returning the option number.
    pub fn add_option(&mut self, meaning: T, items: &[ColoredItem]) -> usize {
        let mut items_bitset = FixedBitSet::with_capacity(self.num_items);
        for ci in items {
            items_bitset.insert(ci.item().index());
        }

        let colors: HashMap<ItemId, Color> = items
            .iter()
            .filter_map(|ci| ci.color().map(|color| (ci.item(), color)))
            .collect();

        let option_id = self.options.len();
        self.options.push(OptionData {
            option_id: OptionId::new(self.options.len()),
            items: items_bitset,
            colors,
            meaning,
        });
        option_id
    }

    /// Returns the user-defined meaning of the given option.
    ///
    /// # Example
    ///
    /// ```
    /// use xcc::{Matrix, ColoredItem, ItemId};
    /// let mut matrix = Matrix::new(1, 0);
    /// let option_id = matrix.add_option(123, &[ColoredItem::new(ItemId::new(0))]);
    /// assert_eq!(&123, matrix.meaning(option_id));
    /// ```
    #[must_use]
    pub fn meaning(&self, option_number: usize) -> &T {
        &self.options[option_number].meaning
    }

    /// Returns an iterator over the options that contain the given item.
    /// The iterator yields `OptionData` objects.
    ///
    /// # Example
    ///
    /// ```
    /// use xcc::{Matrix, ColoredItem, ItemId};
    /// let mut matrix = Matrix::new(1, 0);
    /// let primary_item = ItemId::new(0);
    /// let option_id = matrix.add_option(123, &[ColoredItem::new(primary_item)]);
    /// let item_id = ItemId::new(0);
    /// assert_eq!(1, matrix.options_for_item(item_id).count());
    /// assert_eq!(123, matrix.options_for_item(item_id).next().unwrap().meaning);
    /// ```
    pub fn options_for_item(&self, item: ItemId) -> impl Iterator<Item = &OptionData<T>> + '_ {
        self.options
            .iter()
            .filter(move |option| option.items.contains(item.index()))
    }

    /// Returns an iterator over the items (columns) for a given option (row).
    ///
    /// # Arguments
    ///
    /// * `option` - The index of the option for which to retrieve items.
    ///
    /// # Returns
    ///
    /// An iterator over tuples, where the first element is the item index and the second element is an optional color.
    pub fn items_for_option(
        &self,
        option: OptionId,
    ) -> impl Iterator<Item = (ItemId, Option<Color>)> + '_ {
        let opt = &self.options[option.index()];
        opt.items.ones().map(ItemId::new).map(move |item| {
            for (colored_item, color) in &opt.colors {
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
    pub option_id: OptionId,
    // The items (primary and secondary) that take part in this option.
    pub items: FixedBitSet,
    // Map from item ID to color for colored items in this option.
    pub colors: HashMap<ItemId, Color>,
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

        let matrix = builder.build().unwrap();

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
        let mut matrix = builder.build().unwrap();

        assert_eq!(to_vec(&matrix.options[0].items), [0, 1, 3, 4]);
        assert_eq!(
            matrix.options[0].colors,
            hashmap![ItemId::new(4) => Color::new(0)]
        );

        assert_eq!(to_vec(&matrix.options[1].items), [0, 2, 3, 4]);
        assert_eq!(
            matrix.options[1].colors,
            hashmap![ItemId::new(3) => Color::new(0)]
        );

        assert_eq!(to_vec(&matrix.options[2].items), [0, 3]);
        assert_eq!(
            matrix.options[2].colors,
            hashmap![ItemId::new(3) => Color::new(1)]
        );

        assert_eq!(to_vec(&matrix.options[3].items), [1, 3]);
        assert_eq!(
            matrix.options[3].colors,
            hashmap![ItemId::new(3) => Color::new(0)]
        );

        assert_eq!(to_vec(&matrix.options[4].items), [2, 4]);
        assert_eq!(
            matrix.options[4].colors,
            hashmap![ItemId::new(4) => Color::new(1)]
        );

        let solutions = matrix
            .solve_all()
            .into_iter()
            .map(|s| s.meanings(&matrix))
            .collect::<Vec<_>>();

        assert_eq!(solutions, [[&"q x:A", &"p r x:A y"]]);
    }
}
