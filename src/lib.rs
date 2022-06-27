//! The purpose of term-table is to make it easy for CLI apps to display data in a table format
//!# Example
//! Here is an example of how to create a simple table
//!```
//! use term_data_table::{ Table, TableCell, TableStyle, Alignment, Row };
//!
//! let table = Table::new()
//!     .with_max_column_width(40)
//!     .with_style(TableStyle::EXTENDED)
//!     .with_row(Row::new().with_cell(
//!         TableCell::from("This is some centered text")
//!             .with_alignment(Alignment::Center)
//!             .with_col_span(2)
//!     ))
//!     .with_row(Row::new().with_cell(
//!         TableCell::from("This is left aligned text")
//!     ).with_cell(
//!         TableCell::from("This is right aligned text")
//!             .with_alignment(Alignment::Right)
//!     ))
//!     .with_row(Row::new().with_cell(
//!         TableCell::from("This is left aligned text")
//!     ).with_cell(
//!         TableCell::from("This is right aligned text")
//!             .with_alignment(Alignment::Right)
//!     ))
//!     .with_row(Row::new().with_cell(
//!         TableCell::from("This is some really really really really really really really really really that is going to wrap to the next line")
//!             .with_col_span(2)
//!     ));
//!println!("{}", table.render());
//!```
//!
//!### This is the result
//!
//!<pre>
//! ╔═════════════════════════════════════════════════════════════════════════════════╗
//! ║                            This is some centered text                           ║
//! ╠════════════════════════════════════════╦════════════════════════════════════════╣
//! ║ This is left aligned text              ║             This is right aligned text ║
//! ╠════════════════════════════════════════╬════════════════════════════════════════╣
//! ║ This is left aligned text              ║             This is right aligned text ║
//! ╠════════════════════════════════════════╩════════════════════════════════════════╣
//! ║ This is some really really really really really really really really really tha ║
//! ║ t is going to wrap to the next line                                             ║
//! ╚═════════════════════════════════════════════════════════════════════════════════╝
//!</pre>

#[macro_use]
extern crate lazy_static;

mod row;
mod table_cell;

pub use crate::{
    row::{IntoRow, Row},
    table_cell::{Alignment, TableCell},
};
#[doc(inline)]
pub use term_data_table_derive::IntoRow;

use std::{
    cmp::{max, min},
    collections::HashMap,
    fmt,
};

/// Represents the vertical position of a row
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum RowPosition {
    First,
    Mid,
    Last,
}

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

    /// Returns the start character of a table style based on the
    /// vertical position of the row
    fn start_for_position(&self, pos: RowPosition) -> char {
        match pos {
            RowPosition::First => self.top_left_corner,
            RowPosition::Mid => self.outer_left_vertical,
            RowPosition::Last => self.bottom_left_corner,
        }
    }

    /// Returns the end character of a table style based on the
    /// vertical position of the row
    fn end_for_position(&self, pos: RowPosition) -> char {
        match pos {
            RowPosition::First => self.top_right_corner,
            RowPosition::Mid => self.outer_right_vertical,
            RowPosition::Last => self.bottom_right_corner,
        }
    }

    /// Returns the intersect character of a table style based on the
    /// vertical position of the row
    fn intersect_for_position(&self, pos: RowPosition) -> char {
        match pos {
            RowPosition::First => self.outer_top_horizontal,
            RowPosition::Mid => self.intersection,
            RowPosition::Last => self.outer_bottom_horizontal,
        }
    }

    /// Merges two intersecting characters based on the vertical position of a row.
    /// This is used to handle cases where one cell has a larger `col_span` value than the other
    fn merge_intersection_for_position(&self, top: char, bottom: char, pos: RowPosition) -> char {
        if (top == self.horizontal || top == self.outer_bottom_horizontal)
            && bottom == self.intersection
        {
            return self.outer_top_horizontal;
        } else if (top == self.intersection || top == self.outer_top_horizontal)
            && bottom == self.horizontal
        {
            return self.outer_bottom_horizontal;
        } else if top == self.outer_bottom_horizontal && bottom == self.horizontal {
            return self.horizontal;
        } else {
            return self.intersect_for_position(pos);
        }
    }
}

/// A set of rows containing data
#[derive(Clone, Debug)]
pub struct Table<'data> {
    rows: Vec<Row<'data>>,
    style: TableStyle,
    /// The maximum width of all columns. Overridden by values in `max_column_widths`. Defaults to
    /// `usize::MAX`.
    max_column_width: usize,
    /// The maximum widths of specific columns. Override `max_column_width`
    max_column_widths: HashMap<usize, usize>,
    /// Whether or not to vertically separate rows in the table
    pub has_separate_rows: bool,
    /// Whether the table should have a top border.
    /// Setting `has_separator` to false on the first row will have the same effect as setting this to false
    pub has_top_border: bool,
    /// Whether the table should have a bottom border
    pub has_bottom_border: bool,
}

impl<'data> Default for Table<'data> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            style: TableStyle::EXTENDED,
            max_column_width: std::usize::MAX,
            max_column_widths: HashMap::new(),
            has_separate_rows: true,
            has_top_border: true,
            has_bottom_border: true,
        }
    }
}

impl<'data> Table<'data> {
    pub fn new() -> Table<'data> {
        Default::default()
    }

    pub fn from_rows(rows: Vec<Row<'data>>) -> Table<'data> {
        Self {
            rows,
            ..Default::default()
        }
    }

    /// Add a row
    pub fn with_row(mut self, row: Row<'data>) -> Self {
        self.add_row(row);
        self
    }

    /// Add a row
    pub fn add_row(&mut self, row: Row<'data>) -> &mut Self {
        self.rows.push(row);
        self
    }

    pub fn with_style(mut self, style: TableStyle) -> Self {
        self.set_style(style);
        self
    }

    pub fn set_style(&mut self, style: TableStyle) -> &mut Self {
        self.style = style;
        self
    }

    pub fn max_column_width(&self) -> usize {
        self.max_column_width
    }

    pub fn set_max_column_width(&mut self, max_column_width: usize) -> &mut Self {
        self.max_column_width = max_column_width;
        self
    }

    pub fn with_max_column_width(mut self, max_column_width: usize) -> Self {
        self.set_max_column_width(max_column_width);
        self
    }

    /// Set the max width of a particular column
    ///
    /// Overrides any value set for `max_column_width`.
    pub fn set_max_width_for_column(&mut self, column_index: usize, max_width: usize) -> &mut Self {
        self.max_column_widths.insert(column_index, max_width);
        self
    }

    pub fn with_max_width_for_column(mut self, column_index: usize, max_width: usize) -> Self {
        self.set_max_width_for_column(column_index, max_width);
        self
    }

    pub fn has_separate_rows(&self) -> bool {
        self.has_separate_rows
    }

    pub fn with_separate_rows(mut self, has_separate_rows: bool) -> Self {
        self.set_separate_rows(has_separate_rows);
        self
    }

    pub fn set_separate_rows(&mut self, has_separate_rows: bool) -> &mut Self {
        self.has_separate_rows = has_separate_rows;
        self
    }

    /// Does all of the calculations to reformat the row based on it's current
    /// state and returns the result as a `String`
    pub fn render(&self) -> String {
        self.to_string()
    }

    /// Calculates the maximum width for each column.
    ///
    /// If a cell has a column span greater than 1, then the width of it's contents are divided by
    /// the column span, otherwise the cell would use more space than it needed.
    fn calculate_max_column_widths(&self) -> Vec<usize> {
        let mut num_columns = 0;

        for row in &self.rows {
            num_columns = max(row.num_columns(), num_columns);
        }
        let mut max_widths: Vec<usize> = vec![0; num_columns];
        let mut min_widths: Vec<usize> = vec![0; num_columns];
        for row in &self.rows {
            let column_widths = row.split_column_widths();
            for i in 0..column_widths.len() {
                min_widths[i] = max(min_widths[i], column_widths[i].1);
                let mut max_width = *self
                    .max_column_widths
                    .get(&i)
                    .unwrap_or(&self.max_column_width);
                max_width = max(min_widths[i] as usize, max_width);
                max_widths[i] = min(max_width, max(max_widths[i], column_widths[i].0 as usize));
            }
        }

        // Here we are dealing with the case where we have a cell that is center
        // aligned but the max_width doesn't allow for even padding on either side
        for row in &self.rows {
            let mut col_index = 0;
            for cell in row.cells.iter() {
                let mut total_col_width = 0;
                for i in col_index..col_index + cell.col_span {
                    total_col_width += max_widths[i];
                }
                if cell.width() != total_col_width
                    && cell.alignment == Alignment::Center
                    && total_col_width as f32 % 2.0 <= 0.001
                {
                    let mut max_col_width = self.max_column_width;
                    if let Some(specific_width) = self.max_column_widths.get(&col_index) {
                        max_col_width = *specific_width;
                    }

                    if max_widths[col_index] < max_col_width {
                        max_widths[col_index] += 1;
                    }
                }
                if cell.col_span > 1 {
                    col_index += cell.col_span - 1;
                } else {
                    col_index += 1;
                }
            }
        }

        return max_widths;
    }
}

impl<'data> fmt::Display for Table<'data> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let max_widths = self.calculate_max_column_widths();
        let mut previous_separator = None;
        if !self.rows.is_empty() {
            for i in 0..self.rows.len() {
                let row_pos = if i == 0 {
                    RowPosition::First
                } else {
                    RowPosition::Mid
                };

                let separator = self.rows[i].gen_separator(
                    &max_widths,
                    &self.style,
                    row_pos,
                    previous_separator.clone(),
                );

                previous_separator = Some(separator.clone());

                if self.rows[i].has_separator
                    && ((i == 0 && self.has_top_border) || i != 0 && self.has_separate_rows)
                {
                    writeln!(f, "{}", separator)?;
                }

                self.rows[i].format(&max_widths, &self.style, f)?;
            }
            if self.has_bottom_border {
                let separator = self.rows.last().unwrap().gen_separator(
                    &max_widths,
                    &self.style,
                    RowPosition::Last,
                    None,
                );
                writeln!(f, "{}", separator)?;
            }
        }
        Ok(())
    }
}

pub fn data_table<'a, R: 'a>(input: impl IntoIterator<Item = &'a R>)
where
    R: IntoRow,
{
    let mut table = Table::new();
    for row in input {
        table.add_row(row.into_row());
    }
    println!("{}", table);
}

#[cfg(test)]
mod test {

    use crate::row::Row;
    use crate::table_cell::{Alignment, TableCell};
    use crate::Table;
    use crate::TableStyle;
    use pretty_assertions::assert_eq;

    #[test]
    fn correct_default_padding() {
        let table = Table::new()
            .with_separate_rows(false)
            .with_style(TableStyle::SIMPLE)
            .with_row(
                Row::new()
                    .with_cell(TableCell::from("A").with_alignment(Alignment::Center))
                    .with_cell(TableCell::from("B").with_alignment(Alignment::Center)),
            )
            .with_row(
                Row::new()
                    .with_cell(TableCell::from(1.to_string()))
                    .with_cell(TableCell::from("1")),
            )
            .with_row(
                Row::new()
                    .with_cell(TableCell::from(2.to_string()))
                    .with_cell(TableCell::from("10")),
            )
            .with_row(
                Row::new()
                    .with_cell(TableCell::from(3.to_string()))
                    .with_cell(TableCell::from("100")),
            );
        let expected = r"+---+-----+
| A |  B  |
| 1 | 1   |
| 2 | 10  |
| 3 | 100 |
+---+-----+
";
        println!("{}", table.render());
        assert_eq!(expected, table.render());
    }

    #[test]
    fn uneven_center_alignment() {
        let table = Table::new()
            .with_separate_rows(false)
            .with_style(TableStyle::SIMPLE)
            .with_row(Row::new().with_cell(TableCell::from("A").with_alignment(Alignment::Center)))
            .with_row(Row::new().with_cell(TableCell::from(11.to_string())))
            .with_row(Row::new().with_cell(TableCell::from(2.to_string())))
            .with_row(Row::new().with_cell(TableCell::from(3.to_string())));
        let expected = r"+-----+
|  A  |
| 11  |
| 2   |
| 3   |
+-----+
";
        println!("{}", table.render());
        assert_eq!(expected, table.render());
    }

    #[test]
    fn uneven_center_alignment_2() {
        let table = Table::new()
            .with_separate_rows(false)
            .with_style(TableStyle::SIMPLE)
            .with_row(
                Row::new()
                    .with_cell(TableCell::from("A1").with_alignment(Alignment::Center))
                    .with_cell(TableCell::from("B").with_alignment(Alignment::Center)),
            );
        println!("{}", table.render());
        let expected = r"+----+---+
| A1 | B |
+----+---+
";
        println!("{}", table.render());
        assert_eq!(expected, table.render());
    }

    #[test]
    fn simple_table_style() {
        let mut table = Table::new().with_style(TableStyle::SIMPLE);

        add_data_to_test_table(&mut table);

        let expected = r"+---------------------------------------------------------------------------------+
|                            This is some centered text                           |
+----------------------------------------+----------------------------------------+
| This is left aligned text              |             This is right aligned text |
+----------------------------------------+----------------------------------------+
| This is left aligned text              |             This is right aligned text |
+----------------------------------------+----------------------------------------+
| This is some really really really really really really really really really tha |
| t is going to wrap to the next line                                             |
+---------------------------------------------------------------------------------+
";
        println!("{}", table.render());
        assert_eq!(expected, table.render());
    }

    #[test]
    #[ignore]
    fn uneven_with_varying_col_span() {
        let table = Table::new()
            .with_separate_rows(true)
            .with_style(TableStyle::SIMPLE)
            .with_row(
                Row::new()
                    .with_cell(TableCell::from("A1111111").with_alignment(Alignment::Center))
                    .with_cell(TableCell::from("B").with_alignment(Alignment::Center)),
            )
            .with_row(
                Row::new()
                    .with_cell(TableCell::from(1.to_string()))
                    .with_cell(TableCell::from("1")),
            )
            .with_row(
                Row::new()
                    .with_cell(TableCell::from(2.to_string()))
                    .with_cell(TableCell::from("10")),
            )
            .with_row(
                Row::new()
                    .with_cell(
                        TableCell::from(3.to_string())
                            .with_alignment(Alignment::Left)
                            .with_padding(false),
                    )
                    .with_cell(TableCell::from("100")),
            )
            .with_row(Row::new().with_cell(TableCell::from("S").with_alignment(Alignment::Center)));
        let expected = "+----------+-----+
| A1111111 |  B  |
+----------+-----+
| 1        | 1   |
+----------+-----+
| 2        | 10  |
+----------+-----+
|\03\0         | 100 |
+----------+-----+
|        S       |
+----------------+
";
        println!("{}", table.render());
        assert_eq!(expected.trim(), table.render().trim());
    }

    // TODO - The output of this test isn't ideal. There is probably a better way to calculate the
    // the column/row layout that would improve this
    #[test]
    fn uneven_with_varying_col_span_2() {
        let table = Table::new()
            .with_separate_rows(false)
            .with_style(TableStyle::SIMPLE)
            .with_row(
                Row::new()
                    .with_cell(TableCell::from("A").with_alignment(Alignment::Center))
                    .with_cell(TableCell::from("B").with_alignment(Alignment::Center)),
            )
            .with_row(
                Row::new()
                    .with_cell(TableCell::from(1.to_string()))
                    .with_cell(TableCell::from("1")),
            )
            .with_row(
                Row::new()
                    .with_cell(TableCell::from(2.to_string()))
                    .with_cell(TableCell::from("10")),
            )
            .with_row(
                Row::new()
                    .with_cell(TableCell::from(3.to_string()))
                    .with_cell(TableCell::from("100")),
            )
            .with_row(
                Row::new().with_cell(
                    TableCell::from("Spanner")
                        .with_col_span(2)
                        .with_alignment(Alignment::Center),
                ),
            );
        let expected = "+------+-----+
|   A  |  B  |
| 1    | 1   |
| 2    | 10  |
| 3    | 100 |
|   Spanner  |
+------------+
";
        println!("{}", table.render());
        assert_eq!(expected.trim(), table.render().trim());
    }

    /*
        #[test]
        fn extended_table_style_wrapped() {
            let table = Table::new()
                .with_max_column_width(40)
                .with_max_widths_for_columns([(0, 1), (1, 1)])

            .with_style ( TableStyle::EXTENDED)

            .with_row(Row::new(vec![TableCell::new_with_alignment(
                "This is some centered text",
                2,
                Alignment::Center,
            )]))

            .with_row(Row::new(vec![
                TableCell::new("This is left aligned text"),
                TableCell::new_with_alignment("This is right aligned text", 1, Alignment::Right),
            ]))

            .with_row(Row::new(vec![
                TableCell::new("This is left aligned text"),
                TableCell::new_with_alignment("This is right aligned text", 1, Alignment::Right),
            ]))

            .with_row(Row::new(vec![
                TableCell::new_with_col_span("This is some really really really really really really really really really that is going to wrap to the next line\n1\n2", 2),
            ]));

            let expected = r"╔═══════╗
    ║ This  ║
    ║ is so ║
    ║ me ce ║
    ║ ntere ║
    ║ d tex ║
    ║   t   ║
    ╠═══╦═══╣
    ║ T ║ T ║
    ║ h ║ h ║
    ║ i ║ i ║
    ║ s ║ s ║
    ║   ║   ║
    ║ i ║ i ║
    ║ s ║ s ║
    ║   ║   ║
    ║ l ║ r ║
    ║ e ║ i ║
    ║ f ║ g ║
    ║ t ║ h ║
    ║   ║ t ║
    ║ a ║   ║
    ║ l ║ a ║
    ║ i ║ l ║
    ║ g ║ i ║
    ║ n ║ g ║
    ║ e ║ n ║
    ║ d ║ e ║
    ║   ║ d ║
    ║ t ║   ║
    ║ e ║ t ║
    ║ x ║ e ║
    ║ t ║ x ║
    ║   ║ t ║
    ╠═══╬═══╣
    ║ T ║ T ║
    ║ h ║ h ║
    ║ i ║ i ║
    ║ s ║ s ║
    ║   ║   ║
    ║ i ║ i ║
    ║ s ║ s ║
    ║   ║   ║
    ║ l ║ r ║
    ║ e ║ i ║
    ║ f ║ g ║
    ║ t ║ h ║
    ║   ║ t ║
    ║ a ║   ║
    ║ l ║ a ║
    ║ i ║ l ║
    ║ g ║ i ║
    ║ n ║ g ║
    ║ e ║ n ║
    ║ d ║ e ║
    ║   ║ d ║
    ║ t ║   ║
    ║ e ║ t ║
    ║ x ║ e ║
    ║ t ║ x ║
    ║   ║ t ║
    ╠═══╩═══╣
    ║ This  ║
    ║ is so ║
    ║ me re ║
    ║ ally  ║
    ║ reall ║
    ║ y rea ║
    ║ lly r ║
    ║ eally ║
    ║  real ║
    ║ ly re ║
    ║ ally  ║
    ║ reall ║
    ║ y rea ║
    ║ lly r ║
    ║ eally ║
    ║  that ║
    ║  is g ║
    ║ oing  ║
    ║ to wr ║
    ║ ap to ║
    ║  the  ║
    ║ next  ║
    ║ line  ║
    ║ 1     ║
    ║ 2     ║
    ╚═══════╝
    ";
            println!("{}", table.render());
            assert_eq!(expected, table.render());
        }

            #[test]
            fn elegant_table_style() {
                let mut table = Table::new();
                table.style = TableStyle::elegant();

                add_data_to_test_table(&mut table);

                let expected = r"╔─────────────────────────────────────────────────────────────────────────────────╗
        │                            This is some centered text                           │
        ╠────────────────────────────────────────╦────────────────────────────────────────╣
        │ This is left aligned text              │             This is right aligned text │
        ╠────────────────────────────────────────┼────────────────────────────────────────╣
        │ This is left aligned text              │             This is right aligned text │
        ╠────────────────────────────────────────╩────────────────────────────────────────╣
        │ This is some really really really really really really really really really tha │
        │ t is going to wrap to the next line                                             │
        ╚─────────────────────────────────────────────────────────────────────────────────╝
        ";
                println!("{}", table.render());
                assert_eq!(expected, table.render());
            }

            #[test]
            fn thin_table_style() {
                let mut table = Table::new();
                table.style = TableStyle::thin();

                add_data_to_test_table(&mut table);

                let expected = r"┌─────────────────────────────────────────────────────────────────────────────────┐
        │                            This is some centered text                           │
        ├────────────────────────────────────────┬────────────────────────────────────────┤
        │ This is left aligned text              │             This is right aligned text │
        ├────────────────────────────────────────┼────────────────────────────────────────┤
        │ This is left aligned text              │             This is right aligned text │
        ├────────────────────────────────────────┴────────────────────────────────────────┤
        │ This is some really really really really really really really really really tha │
        │ t is going to wrap to the next line                                             │
        └─────────────────────────────────────────────────────────────────────────────────┘
        ";
                println!("{}", table.render());
                assert_eq!(expected, table.render());
            }

            #[test]
            fn rounded_table_style() {
                let mut table = Table::new();

                table.style = TableStyle::rounded();

                add_data_to_test_table(&mut table);

                let expected = r"╭─────────────────────────────────────────────────────────────────────────────────╮
        │                            This is some centered text                           │
        ├────────────────────────────────────────┬────────────────────────────────────────┤
        │ This is left aligned text              │             This is right aligned text │
        ├────────────────────────────────────────┼────────────────────────────────────────┤
        │ This is left aligned text              │             This is right aligned text │
        ├────────────────────────────────────────┴────────────────────────────────────────┤
        │ This is some really really really really really really really really really tha │
        │ t is going to wrap to the next line                                             │
        ╰─────────────────────────────────────────────────────────────────────────────────╯
        ";
                println!("{}", table.render());
                assert_eq!(expected, table.render());
            }

            #[test]
            fn complex_table() {
                let mut table = Table::new();

                table.add_row(Row::new(vec![
                    TableCell::new_with_col_span("Col*1*Span*2", 2),
                    TableCell::new("Col 2 Span 1"),
                    TableCell::new_with_col_span("Col 3 Span 2", 2),
                    TableCell::new("Col 4 Span 1"),
                ]));
                table.add_row(Row::new(vec![
                    TableCell::new("Col 1 Span 1"),
                    TableCell::new("Col 2 Span 1"),
                    TableCell::new("Col 3 Span 1"),
                    TableCell::new_with_col_span("Col 4 Span 1", 2),
                ]));
                table.add_row(Row::new(vec![
                    TableCell::new("fasdaff"),
                    TableCell::new("fff"),
                    TableCell::new("fff"),
                ]));
                table.add_row(Row::new(vec![
                    TableCell::new_with_alignment("fasdff", 3, Alignment::Right),
                    TableCell::new_with_col_span("fffdff", 4),
                ]));
                table.add_row(Row::new(vec![
                    TableCell::new("fasdsaff"),
                    TableCell::new("fff"),
                    TableCell::new("f\nf\nf\nfff\nrrr\n\n\n"),
                ]));
                table.add_row(Row::new(vec![TableCell::new("fasdsaff")]));

                let s = table.render().clone();

                table.add_row(Row::new(vec![TableCell::new_with_alignment(
                    s,
                    3,
                    Alignment::Left,
                )]));

                let expected = r"╔═════════════════════════════════════════════════════════╦════════════════════════════╦════════════════╦══════════════╦═══╗
        ║ Col*1*Span*2                                            ║ Col 2 Span 1               ║ Col 3 Span 2   ║ Col 4 Span 1 ║   ║
        ╠════════════════════════════╦════════════════════════════╬════════════════════════════╬════════════════╬══════════════╬═══╣
        ║ Col 1 Span 1               ║ Col 2 Span 1               ║ Col 3 Span 1               ║ Col 4 Span 1   ║              ║   ║
        ╠════════════════════════════╬════════════════════════════╬════════════════════════════╬═══════╦════════╬══════════════╬═══╣
        ║ fasdaff                    ║ fff                        ║ fff                        ║       ║        ║              ║   ║
        ╠════════════════════════════╩════════════════════════════╩════════════════════════════╬═══════╩════════╩══════════════╩═══╣
        ║                                                                               fasdff ║ fffdff                            ║
        ╠════════════════════════════╦════════════════════════════╦════════════════════════════╬═══════╦════════╦══════════════╦═══╣
        ║ fasdsaff                   ║ fff                        ║ f                          ║       ║        ║              ║   ║
        ║                            ║                            ║ f                          ║       ║        ║              ║   ║
        ║                            ║                            ║ f                          ║       ║        ║              ║   ║
        ║                            ║                            ║ fff                        ║       ║        ║              ║   ║
        ║                            ║                            ║ rrr                        ║       ║        ║              ║   ║
        ║                            ║                            ║                            ║       ║        ║              ║   ║
        ║                            ║                            ║                            ║       ║        ║              ║   ║
        ║                            ║                            ║                            ║       ║        ║              ║   ║
        ╠════════════════════════════╬════════════════════════════╬════════════════════════════╬═══════╬════════╬══════════════╬═══╣
        ║ fasdsaff                   ║                            ║                            ║       ║        ║              ║   ║
        ╠════════════════════════════╩════════════════════════════╩════════════════════════════╬═══════╬════════╬══════════════╬═══╣
        ║ ╔═════════════════════════════╦══════════════╦════════════════╦══════════════╦═══╗   ║       ║        ║              ║   ║
        ║ ║ Col*1*Span*2                ║ Col 2 Span 1 ║ Col 3 Span 2   ║ Col 4 Span 1 ║   ║   ║       ║        ║              ║   ║
        ║ ╠══════════════╦══════════════╬══════════════╬════════════════╬══════════════╬═══╣   ║       ║        ║              ║   ║
        ║ ║ Col 1 Span 1 ║ Col 2 Span 1 ║ Col 3 Span 1 ║ Col 4 Span 1   ║              ║   ║   ║       ║        ║              ║   ║
        ║ ╠══════════════╬══════════════╬══════════════╬═══════╦════════╬══════════════╬═══╣   ║       ║        ║              ║   ║
        ║ ║ fasdaff      ║ fff          ║ fff          ║       ║        ║              ║   ║   ║       ║        ║              ║   ║
        ║ ╠══════════════╩══════════════╩══════════════╬═══════╩════════╩══════════════╩═══╣   ║       ║        ║              ║   ║
        ║ ║                                     fasdff ║ fffdff                            ║   ║       ║        ║              ║   ║
        ║ ╠══════════════╦══════════════╦══════════════╬═══════╦════════╦══════════════╦═══╣   ║       ║        ║              ║   ║
        ║ ║ fasdsaff     ║ fff          ║ f            ║       ║        ║              ║   ║   ║       ║        ║              ║   ║
        ║ ║              ║              ║ f            ║       ║        ║              ║   ║   ║       ║        ║              ║   ║
        ║ ║              ║              ║ f            ║       ║        ║              ║   ║   ║       ║        ║              ║   ║
        ║ ║              ║              ║ fff          ║       ║        ║              ║   ║   ║       ║        ║              ║   ║
        ║ ║              ║              ║ rrr          ║       ║        ║              ║   ║   ║       ║        ║              ║   ║
        ║ ║              ║              ║              ║       ║        ║              ║   ║   ║       ║        ║              ║   ║
        ║ ║              ║              ║              ║       ║        ║              ║   ║   ║       ║        ║              ║   ║
        ║ ║              ║              ║              ║       ║        ║              ║   ║   ║       ║        ║              ║   ║
        ║ ╠══════════════╬══════════════╬══════════════╬═══════╬════════╬══════════════╬═══╣   ║       ║        ║              ║   ║
        ║ ║ fasdsaff     ║              ║              ║       ║        ║              ║   ║   ║       ║        ║              ║   ║
        ║ ╚══════════════╩══════════════╩══════════════╩═══════╩════════╩══════════════╩═══╝   ║       ║        ║              ║   ║
        ║                                                                                      ║       ║        ║              ║   ║
        ╚══════════════════════════════════════════════════════════════════════════════════════╩═══════╩════════╩══════════════╩═══╝
        ";
                println!("{}", table.render());
                assert_eq!(expected, table.render());
            }

            #[test]
            fn no_top_border() {
                let mut table = Table::new();
                table.style = TableStyle::simple();
                table.has_top_border = false;

                add_data_to_test_table(&mut table);

                let expected = r"|                            This is some centered text                           |
        +----------------------------------------+----------------------------------------+
        | This is left aligned text              |             This is right aligned text |
        +----------------------------------------+----------------------------------------+
        | This is left aligned text              |             This is right aligned text |
        +----------------------------------------+----------------------------------------+
        | This is some really really really really really really really really really tha |
        | t is going to wrap to the next line                                             |
        +---------------------------------------------------------------------------------+
        ";
                println!("{}", table.render());
                assert_eq!(expected, table.render());
            }

            #[test]
            fn no_bottom_border() {
                let mut table = Table::new();
                table.style = TableStyle::simple();
                table.has_bottom_border = false;

                add_data_to_test_table(&mut table);

                let expected = r"+---------------------------------------------------------------------------------+
        |                            This is some centered text                           |
        +----------------------------------------+----------------------------------------+
        | This is left aligned text              |             This is right aligned text |
        +----------------------------------------+----------------------------------------+
        | This is left aligned text              |             This is right aligned text |
        +----------------------------------------+----------------------------------------+
        | This is some really really really really really really really really really tha |
        | t is going to wrap to the next line                                             |
        ";
                println!("{}", table.render());
                assert_eq!(expected, table.render());
            }

            #[test]
            fn no_separators() {
                let mut table = Table::new();
                table.style = TableStyle::simple();
                table.separate_rows = false;

                add_data_to_test_table(&mut table);

                let expected = r"+---------------------------------------------------------------------------------+
        |                            This is some centered text                           |
        | This is left aligned text              |             This is right aligned text |
        | This is left aligned text              |             This is right aligned text |
        | This is some really really really really really really really really really tha |
        | t is going to wrap to the next line                                             |
        +---------------------------------------------------------------------------------+
        ";
                println!("{}", table.render());
                assert_eq!(expected, table.render());
            }

            #[test]
            fn some_rows_no_separators() {
                let mut table = Table::new();
                table.style = TableStyle::simple();

                add_data_to_test_table(&mut table);

                table.rows[2].has_separator = false;

                let expected = r"+---------------------------------------------------------------------------------+
        |                            This is some centered text                           |
        +----------------------------------------+----------------------------------------+
        | This is left aligned text              |             This is right aligned text |
        | This is left aligned text              |             This is right aligned text |
        +----------------------------------------+----------------------------------------+
        | This is some really really really really really really really really really tha |
        | t is going to wrap to the next line                                             |
        +---------------------------------------------------------------------------------+
        ";
                println!("{}", table.render());
                assert_eq!(expected, table.render());
            }

            #[test]
            fn colored_data_works() {
                let mut table = Table::new();
                table.add_row(Row::new(vec![TableCell::new("\u{1b}[31ma\u{1b}[0m")]));
                let expected = "╔═══╗
        ║ \u{1b}[31ma\u{1b}[0m ║
        ╚═══╝
        ";
                println!("{}", table.render());
                assert_eq!(expected, table.render());
            }
        */

    fn add_data_to_test_table(table: &mut Table) {
        table.set_max_column_width(40);
        table.add_row(
            Row::new().with_cell(
                TableCell::from("This is some centered text")
                    .with_col_span(2)
                    .with_alignment(Alignment::Center),
            ),
        );

        table.add_row(
            Row::new()
                .with_cell(TableCell::from("This is left aligned text"))
                .with_cell(
                    TableCell::from("This is right aligned text").with_alignment(Alignment::Right),
                ),
        );

        table.add_row(
            Row::new()
                .with_cell(TableCell::from("This is left aligned text"))
                .with_cell(
                    TableCell::from("This is right aligned text").with_alignment(Alignment::Right),
                ),
        );

        table.add_row(
            Row::new().with_cell(
                TableCell::from(
                    "This is some really really really really really \
                really really really really that is going to wrap to the next line",
                )
                .with_col_span(2),
            ),
        );
    }
}
