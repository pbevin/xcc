/// Represents an item in the Dancing Links data structure that may or may not have
/// a color assigned to it.
///
/// A color here is just an integer.
///
/// # Examples
///
/// ```
/// use xcc::ColoredItem;
///
/// let item = ColoredItem::new(1);
/// assert_eq!(item.color(), None);
///
/// let item = ColoredItem::with_color(1, 100);
/// assert_eq!(item.color(), Some(100));
/// ```
#[derive(Clone, Copy)]
pub struct ColoredItem {
    /// The index of the header cell for this item.
    header_pos: usize,
    /// The color of this item, if any.
    color: Option<usize>,
}

impl ColoredItem {
    /// Creates a new ColoredItem with no color.
    ///
    /// # Examples
    /// ```
    /// use xcc::ColoredItem;
    ///
    /// let item = ColoredItem::new(1);
    /// assert_eq!(item.color(), None);
    /// ```
    pub fn new(name: usize) -> Self {
        ColoredItem {
            header_pos: name,
            color: None,
        }
    }

    pub fn item(&self) -> usize {
        self.header_pos
    }

    pub fn color(&self) -> Option<usize> {
        self.color
    }

    /// Creates a new ColoredItem with the given color.
    ///
    /// # Examples
    /// ```
    /// use xcc::ColoredItem;
    /// let item = ColoredItem::with_color(1, 100);
    /// assert_eq!(item.color(), Some(100));
    /// ```
    pub fn with_color(name: usize, color: usize) -> Self {
        ColoredItem {
            header_pos: name,
            color: Some(color),
        }
    }
}
