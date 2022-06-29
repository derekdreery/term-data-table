use unicode_width::UnicodeWidthChar;

/// A set of characters which make up a table style
///
///# Example
///
///```
/// term_data_table::TableStyle {
///     top_left_corner: '╔',
///     top_right_corner: '╗',
///     bottom_left_corner: '╚',
///     bottom_right_corner: '╝',
///     outer_left_vertical: '╠',
///     outer_right_vertical: '╣',
///     outer_bottom_horizontal: '╩',
///     outer_top_horizontal: '╦',
///     intersection: '╬',
///     vertical: '║',
///     horizontal: '═',
/// };
///```
#[derive(Debug, Clone, Copy)]
pub struct TableStyle {
    pub top_left_corner: char,
    pub top_right_corner: char,
    pub bottom_left_corner: char,
    pub bottom_right_corner: char,
    pub outer_left_vertical: char,
    pub outer_right_vertical: char,
    pub outer_bottom_horizontal: char,
    pub outer_top_horizontal: char,
    pub intersection: char,
    pub vertical: char,
    pub horizontal: char,
}

impl TableStyle {
    /// Basic terminal table style
    ///
    ///# Example
    ///
    ///<pre>
    ///   +---------------------------------------------------------------------------------+
    ///   |                            This is some centered text                           |
    ///   +----------------------------------------+----------------------------------------+
    ///   | This is left aligned text              |             This is right aligned text |
    ///   +----------------------------------------+----------------------------------------+
    ///   | This is left aligned text              |             This is right aligned text |
    ///   +----------------------------------------+----------------------------------------+
    ///   | This is some really really really really really really really really really tha |
    ///   | t is going to wrap to the next line                                             |
    ///   +---------------------------------------------------------------------------------+
    ///</pre>
    pub const SIMPLE: TableStyle = TableStyle {
        top_left_corner: '+',
        top_right_corner: '+',
        bottom_left_corner: '+',
        bottom_right_corner: '+',
        outer_left_vertical: '+',
        outer_right_vertical: '+',
        outer_bottom_horizontal: '+',
        outer_top_horizontal: '+',
        intersection: '+',
        vertical: '|',
        horizontal: '-',
    };

    /// Table style using extended character set
    ///
    ///# Example
    ///
    ///<pre>
    /// ╔═════════════════════════════════════════════════════════════════════════════════╗
    /// ║                            This is some centered text                           ║
    /// ╠════════════════════════════════════════╦════════════════════════════════════════╣
    /// ║ This is left aligned text              ║             This is right aligned text ║
    /// ╠════════════════════════════════════════╬════════════════════════════════════════╣
    /// ║ This is left aligned text              ║             This is right aligned text ║
    /// ╠════════════════════════════════════════╩════════════════════════════════════════╣
    /// ║ This is some really really really really really really really really really tha ║
    /// ║ t is going to wrap to the next line                                             ║
    /// ╚═════════════════════════════════════════════════════════════════════════════════╝
    ///</pre>
    pub const EXTENDED: TableStyle = TableStyle {
        top_left_corner: '╔',
        top_right_corner: '╗',
        bottom_left_corner: '╚',
        bottom_right_corner: '╝',
        outer_left_vertical: '╠',
        outer_right_vertical: '╣',
        outer_bottom_horizontal: '╩',
        outer_top_horizontal: '╦',
        intersection: '╬',
        vertical: '║',
        horizontal: '═',
    };

    /// <pre>
    /// ┌─────────────────────────────────────────────────────────────────────────────────┐
    /// │                            This is some centered text                           │
    /// ├────────────────────────────────────────┬────────────────────────────────────────┤
    /// │ This is left aligned text              │             This is right aligned text │
    /// ├────────────────────────────────────────┼────────────────────────────────────────┤
    /// │ This is left aligned text              │             This is right aligned text │
    /// ├────────────────────────────────────────┴────────────────────────────────────────┤
    /// │ This is some really really really really really really really really really tha │
    /// │ t is going to wrap to the next line                                             │
    /// └─────────────────────────────────────────────────────────────────────────────────┘
    /// </pre>
    pub const THIN: TableStyle = TableStyle {
        top_left_corner: '┌',
        top_right_corner: '┐',
        bottom_left_corner: '└',
        bottom_right_corner: '┘',
        outer_left_vertical: '├',
        outer_right_vertical: '┤',
        outer_bottom_horizontal: '┴',
        outer_top_horizontal: '┬',
        intersection: '┼',
        vertical: '│',
        horizontal: '─',
    };

    ///  <pre>
    /// ╭─────────────────────────────────────────────────────────────────────────────────╮
    /// │                            This is some centered text                           │
    /// ├────────────────────────────────────────┬────────────────────────────────────────┤
    /// │ This is left aligned text              │             This is right aligned text │
    /// ├────────────────────────────────────────┼────────────────────────────────────────┤
    /// │ This is left aligned text              │             This is right aligned text │
    /// ├────────────────────────────────────────┴────────────────────────────────────────┤
    /// │ This is some really really really really really really really really really tha │
    /// │ t is going to wrap to the next line                                             │
    /// ╰─────────────────────────────────────────────────────────────────────────────────╯
    /// </pre>
    pub const ROUNDED: TableStyle = TableStyle {
        top_left_corner: '╭',
        top_right_corner: '╮',
        bottom_left_corner: '╰',
        bottom_right_corner: '╯',
        outer_left_vertical: '├',
        outer_right_vertical: '┤',
        outer_bottom_horizontal: '┴',
        outer_top_horizontal: '┬',
        intersection: '┼',
        vertical: '│',
        horizontal: '─',
    };

    /// <pre>
    /// ╔─────────────────────────────────────────────────────────────────────────────────╗
    /// │                            This is some centered text                           │
    /// ╠────────────────────────────────────────╦────────────────────────────────────────╣
    /// │ This is left aligned text              │             This is right aligned text │
    /// ╠────────────────────────────────────────┼────────────────────────────────────────╣
    /// │ This is left aligned text              │             This is right aligned text │
    /// ╠────────────────────────────────────────╩────────────────────────────────────────╣
    /// │ This is some really really really really really really really really really tha │
    /// │ t is going to wrap to the next line                                             │
    /// ╚─────────────────────────────────────────────────────────────────────────────────╝
    /// </pre>

    pub const ELEGANT: TableStyle = TableStyle {
        top_left_corner: '╔',
        top_right_corner: '╗',
        bottom_left_corner: '╚',
        bottom_right_corner: '╝',
        outer_left_vertical: '╠',
        outer_right_vertical: '╣',
        outer_bottom_horizontal: '╩',
        outer_top_horizontal: '╦',
        intersection: '┼',
        vertical: '│',
        horizontal: '─',
    };

    /// Table style comprised of null characters
    ///
    ///# Example
    ///
    ///<pre>
    ///                           This is some centered text
    ///
    /// This is left aligned text                           This is right aligned text
    ///
    /// This is left aligned text                           This is right aligned text
    ///
    /// This is some really really really really really really really really really tha
    /// t is going to wrap to the next line
    ///</pre>
    pub const BLANK: TableStyle = TableStyle {
        top_left_corner: '\0',
        top_right_corner: '\0',
        bottom_left_corner: '\0',
        bottom_right_corner: '\0',
        outer_left_vertical: '\0',
        outer_right_vertical: '\0',
        outer_bottom_horizontal: '\0',
        outer_top_horizontal: '\0',
        intersection: '\0',
        vertical: '\0',
        horizontal: '\0',
    };

    /// Table style comprised of empty characters for compatibility with terminals
    /// that don't handle null characters appropriately
    ///
    ///# Example
    ///
    ///<pre>
    ///                           This is some centered text
    ///
    /// This is left aligned text                           This is right aligned text
    ///
    /// This is left aligned text                           This is right aligned text
    ///
    /// This is some really really really really really really really really really tha
    /// t is going to wrap to the next line
    ///</pre>
    pub const EMPTY: TableStyle = TableStyle {
        top_left_corner: ' ',
        top_right_corner: ' ',
        bottom_left_corner: ' ',
        bottom_right_corner: ' ',
        outer_left_vertical: ' ',
        outer_right_vertical: ' ',
        outer_bottom_horizontal: ' ',
        outer_top_horizontal: ' ',
        intersection: ' ',
        vertical: ' ',
        horizontal: ' ',
    };

    pub(crate) fn border_width(&self) -> usize {
        self.vertical.width().unwrap_or(0)
    }
}
