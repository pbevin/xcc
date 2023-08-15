use super::Matrix;
use crate::ColoredItem;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("Item {0} is used in an option, but not declared")]
    ItemNotDeclared(String),
    #[error("Item {0} is declared as both primary and secondary")]
    ItemDeclaredTwice(String),
    #[error("No primary items declared")]
    NoPrimaryItems,
    #[error("Primary item {0} is not used in any option, so no solutions are possible.")]
    PrimaryItemNotUsed(String),
    #[error("No options declared")]
    NoOptions,
}

/// A builder for a matrix.
///
/// The usual way to use this is to call `Matrix::builder()` to get a Builder,
/// call `add_primary_items()`, `add_secondary_items()`, and `add_option()` to
/// configure the matrix, and finally call `build()` to get a Matrix.
///
/// The `add_option()` method takes a parameter of type `T`, which can carry any
/// data you want.  The solver will not look at the meanings, but simply returns
/// them to you when you call `meanings()` on a Solution.  Typically, this
/// meaning is a struct or enum that helps you reconstruct a solution from a
/// chosen set of options.  For example, if you are solving a Sudoku puzzle, the
/// meaning might be a struct that contains the row, column, and value of a
/// cell. You can then reconstruct the puzzle by starting from a blank grid and
/// filling in the cells.  See `examples/sudoku.rs` for an example.
///
/// Every option must contain at least one primary item; secondary items are
/// optional.  Every item must be declared as either primary or secondary before
/// calling `build()`, although it's OK to call `add_option()` before
/// `add_primary_items()` or `add_secondary_items()`.
///
///
///
/// # Example
/// ```
/// use xcc::Matrix;
///
/// let mut builder = Matrix::builder();
/// builder.add_primary_items(["p", "q", "r"]);
/// builder.add_secondary_items(["x", "y"]);
/// builder.add_option(1, ["p", "q", "x", "y:A"]);
/// builder.add_option(2, ["p", "r", "x:A", "y"]);
/// builder.add_option(3, ["p", "x:B"]);
/// builder.add_option(4, ["q", "x:A"]);
/// builder.add_option(5, ["r", "y:B"]);
/// let matrix = builder.build();
/// ```
///
#[derive(Debug, Clone)]
pub struct Builder<T> {
    primary_items: Vec<String>,
    secondary_items: Vec<String>,
    options: Vec<(T, Vec<String>)>,
}

impl<T> Default for Builder<T> {
    fn default() -> Self {
        Self {
            primary_items: Vec::new(),
            secondary_items: Vec::new(),
            options: Vec::new(),
        }
    }
}

impl<T> Builder<T> {
    /// Creates a new Builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds primary items to the matrix.
    pub fn add_primary_items<S: Display>(&mut self, items: impl IntoIterator<Item = S>) {
        self.primary_items
            .extend(items.into_iter().map(|t| t.to_string()));
    }

    /// Adds secondary items to the matrix.
    pub fn add_secondary_items<S: Display>(&mut self, items: impl IntoIterator<Item = S>) {
        self.secondary_items
            .extend(items.into_iter().map(|t| t.to_string()));
    }

    /// Adds a single primary item to the matrix.
    pub fn add_primary_item(&mut self, item: impl Display) {
        self.primary_items.push(item.to_string());
    }

    /// Adds a single secondary item to the matrix.
    pub fn add_secondary_item(&mut self, item: impl Display) {
        self.secondary_items.push(item.to_string());
    }

    /// Adds an option to the matrix.
    pub fn add_option<S: Display>(&mut self, meaning: T, items: impl IntoIterator<Item = S>) {
        let items: Vec<_> = items.into_iter().map(|i| i.to_string()).collect();
        for item in &items {
            if let Some((name, _color)) = item.split_once(':') {
                if self.primary_items.iter().any(|i| i == name) {
                    panic!("Primary items cannot be colored: {item} in {items:?}");
                }
            }
        }
        self.options.push((meaning, items));
    }

    /// Builds the matrix.  If there is a problem, this will panic.
    pub fn build(self) -> Matrix<T> {
        self.try_build().unwrap()
    }

    /// Builds the matrix, returning a Result. If there is a problem, this will
    /// return a {BuildError}.
    pub fn try_build(self) -> Result<Matrix<T>, BuildError> {
        let primary_items: &[String] = &self.primary_items;
        let secondary_items: &[String] = &self.secondary_items;
        let options = self.options;

        let header_names: HashMap<&str, usize> = primary_items
            .iter()
            .chain(secondary_items.iter())
            .enumerate()
            .map(|(i, s)| (s.as_ref(), i))
            .collect();

        let mut colors = HashMap::new();
        for (_, option) in &options {
            for item in option {
                if let Some((_name, color)) = item.split_once(':') {
                    let next_id = colors.len();
                    colors.entry(color.to_string()).or_insert(next_id);
                }
            }
        }

        // Build a list of all items (primary, then secondary)
        let mut matrix = Matrix::new(self.primary_items.len(), self.secondary_items.len());
        for (meaning, opt_items) in options {
            let mut parsed_items = Vec::new();

            for s in opt_items {
                let parsed_item = match s.split_once(':') {
                    Some((name, color)) => {
                        let item_id = *header_names
                            .get(name)
                            .unwrap_or_else(|| panic!("Item {:?} not found", name));
                        let color_id = colors[color];
                        ColoredItem::with_color(item_id, color_id)
                    }
                    None => {
                        let item_id = header_names[s.as_str()];
                        ColoredItem::new(item_id)
                    }
                };
                parsed_items.push(parsed_item);
            }
            matrix.add_option(&parsed_items, meaning);
        }
        Ok(matrix)
    }
}

impl<T: Debug> Builder<T> {
    /// Prints the configuration to stdout in a format that can be read by Knuth's dlx2 program.
    /// Only available if the type of meanings is Debug.
    pub fn dump_knuth_format(&self) {
        println!("| primary items: {}", self.primary_items.len());
        println!("| secondary items: {}", self.secondary_items.len());
        println!("| options: {}", self.options.len());
        print!("{}", self.primary_items.join(" "));
        if !self.secondary_items.is_empty() {
            print!(" | ");
            print!("{}", self.secondary_items.join(" "));
        }
        println!();
        for (i, (meaning, items)) in self.options.iter().enumerate() {
            println!("| Option {}: {:?}", i, meaning);
            println!("{}", items.join(" "));
        }
    }
}
