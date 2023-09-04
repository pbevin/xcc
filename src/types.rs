/// ID of an option (row) in the matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OptionId(usize);

impl OptionId {
    /// Creates a new `OptionId`.
    #[must_use]
    pub fn new(id: usize) -> Self {
        OptionId(id)
    }

    /// Returns the index of the option in the matrix.
    #[must_use]
    pub(crate) fn index(self) -> usize {
        self.0
    }
}

/// ID of an item (column) in the matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId(usize);

impl ItemId {
    /// Creates a new `ItemId`.
    #[must_use]
    pub fn new(id: usize) -> Self {
        ItemId(id)
    }

    /// Returns the index of the item in the matrix.
    #[must_use]
    pub(crate) fn index(self) -> usize {
        self.0
    }
}

/// Color of an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Color(usize);

impl Color {
    /// Creates a new `Color`.
    #[must_use]
    pub fn new(id: usize) -> Self {
        Color(id)
    }
}

/// Represents an item in the Dancing Links data structure that may or may not have
/// a color assigned to it.
///
/// A color here is just an integer.
///
/// # Examples
///
/// ```
/// use xcc::{Color, ColoredItem, ItemId};
///
/// let item = ColoredItem::new(ItemId::new(1));
/// assert_eq!(item.color(), None);
///
/// let item = ColoredItem::with_color(ItemId::new(1), Color::new(100));
/// assert_eq!(item.color(), Some(Color::new(100)));
/// ```
#[derive(Clone, Copy)]
pub struct ColoredItem {
    item_id: ItemId,
    color: Option<Color>,
}

impl ColoredItem {
    /// Creates a new `ColoredItem` with no color.
    ///
    /// # Examples
    /// ```
    /// use xcc::{ColoredItem, ItemId};
    ///
    /// let item = ColoredItem::new(ItemId::new(42));
    /// assert_eq!(item.item(), ItemId::new(42));
    /// assert_eq!(item.color(), None);
    /// ```
    #[must_use]
    pub fn new(item_id: ItemId) -> Self {
        ColoredItem {
            item_id,
            color: None,
        }
    }

    /// Returns the index of the header cell for this item.
    #[must_use]
    pub fn item(&self) -> ItemId {
        self.item_id
    }

    /// Returns the color of this item, if any.
    #[must_use]
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Creates a new `ColoredItem` with the given color.
    ///
    /// # Examples
    /// ```
    /// use xcc::{ColoredItem, Color, ItemId};
    /// let item = ColoredItem::with_color(ItemId::new(42), Color::new(100));
    /// assert_eq!(item.item(), ItemId::new(42));
    /// assert_eq!(item.color(), Some(Color::new(100)));
    /// ```
    #[must_use]
    pub fn with_color(item_id: ItemId, color: Color) -> Self {
        ColoredItem {
            item_id,
            color: Some(color),
        }
    }
}
