//! The purpose of term-table is to make it easy for CLI apps to display data in a table format
//!# Example
//! Here is an example of how to create a simple table
//!```
//! use term_data_table::{ Table, Cell, TableStyle, Alignment, Row };
//!
//! let table = Table::new()
//!     .with_style(TableStyle::EXTENDED)
//!     .with_row(Row::new().with_cell(
//!         Cell::from("This is some centered text")
//!             .with_alignment(Alignment::Center)
//!             .with_col_span(2)
//!     ))
//!     .with_row(Row::new().with_cell(
//!         Cell::from("This is left aligned text")
//!     ).with_cell(
//!         Cell::from("This is right aligned text")
//!             .with_alignment(Alignment::Right)
//!     ))
//!     .with_row(Row::new().with_cell(
//!         Cell::from("This is left aligned text")
//!     ).with_cell(
//!         Cell::from("This is right aligned text")
//!             .with_alignment(Alignment::Right)
//!     ))
//!     .with_row(Row::new().with_cell(
//!         Cell::from("This is some really really really really really really really really really that is going to wrap to the next line")
//!             .with_col_span(2)
//!     ));
//!println!("{}", table.fixed_width(80));
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

mod cell;
mod row;
mod style;

pub use crate::{
    cell::{Alignment, Cell},
    row::{IntoRow, Row},
    style::TableStyle,
};
// TODO just use a serde deserializer.
#[doc(inline)]
pub use term_data_table_derive::IntoRow;

use itertools::Itertools;
use std::{cell::RefCell, fmt};
use terminal_size::terminal_size;

/// Represents the vertical position of a row
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum RowPosition {
    First,
    Mid,
    Last,
}

/// A set of rows containing data
#[derive(Clone, Debug)]
pub struct Table<'data> {
    rows: Vec<Row<'data>>,
    style: TableStyle,
    /// Whether or not to vertically separate rows in the table.
    ///
    /// Defaults to `true`.
    pub has_separate_rows: bool,
    /// Whether the table should have a top border.
    ///
    /// Setting `has_separator` to false on the first row will have the same effect as setting this
    /// to false
    ///
    /// Defaults to `true`.
    pub has_top_border: bool,
    /// Whether the table should have a bottom border
    ///
    /// Defaults to `true`.
    pub has_bottom_border: bool,

    /// Calculated column widths.
    column_widths: RefCell<ColumnWidths>,
    /// Calculated row lines
    row_lines: RefCell<Vec<usize>>,
}

impl<'data> Default for Table<'data> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            style: TableStyle::EXTENDED,
            has_separate_rows: true,
            has_top_border: true,
            has_bottom_border: true,

            column_widths: RefCell::new(ColumnWidths::new()),
            row_lines: RefCell::new(vec![]),
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

    /*
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
    */

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

    /// Decide how much space to give each cell and layout the rows.
    ///
    /// If no width is given, all cells will be the largest of their contents.
    ///
    fn layout(&self, width: Option<usize>) {
        // We need to know the maxiumum number of columns in a row.
        let cols = self.rows.iter().map(|row| row.columns()).max().unwrap_or(0);
        let border_width = self.style.border_width();
        let mut col_widths = self.column_widths.borrow_mut();
        col_widths.reset(cols);

        // short-circuit when there are no columns
        if cols == 0 {
            return;
        }

        if let Some(width) = width {
            // For now, just give each cell the same amount of space. In the future, it might be
            // possible to give more space to cells that need it (based on their min_width).
            col_widths.fit_even_rows(width, cols, border_width);
        } else {
            // Give all cells all the space they need.
            for row in self.rows.iter() {
                col_widths.fit_row_singleline(row, border_width);
            }
        }
        for row in self.rows.iter() {
            self.row_lines
                .borrow_mut()
                .push(row.layout(&*col_widths, border_width));
        }
    }

    /// Write the table out to a formatter.
    ///
    /// This method calculates stale state that it needs.
    ///
    /// # Params
    ///  - `view_width` - the width of the viewport we are rendering to, if any. If unspecified,
    ///    we will assume infinite width.
    fn render(&self, view_width: Option<usize>, f: &mut fmt::Formatter) -> fmt::Result {
        self.layout(view_width);
        if self.rows.is_empty() {
            return writeln!(f, "<empty table>");
        }
        let row_lines = self.row_lines.borrow();

        if self.has_top_border {
            self.rows[0].render_top_separator(&*self.column_widths.borrow(), &self.style, f)?;
        }
        self.rows[0].render_content(&*self.column_widths.borrow(), row_lines[0], &self.style, f)?;

        for (idx, (prev_row, row)) in self.rows.iter().tuple_windows().enumerate() {
            if self.has_separate_rows {
                row.render_separator(prev_row, &*self.column_widths.borrow(), &self.style, f)?;
            }

            let row_lines = self.row_lines.borrow();
            row.render_content(
                &*self.column_widths.borrow(),
                row_lines[idx + 1],
                &self.style,
                f,
            )?;
        }
        if self.has_bottom_border {
            self.rows[self.rows.len() - 1].render_bottom_separator(
                &*self.column_widths.borrow(),
                &self.style,
                f,
            )?;
        }
        Ok(())
    }

    /// Get the terminal width and use this for the table width.
    ///
    /// # Panics
    ///
    /// Will panic if it cannot get the terminal width (e.g. because we aren't in a terminal).
    pub fn for_terminal(&self) -> impl fmt::Display + '_ {
        FixedWidth {
            table: self,
            width: (terminal_size().unwrap().0).0.try_into().unwrap(),
        }
    }

    /// Use a custom value for the table width
    pub fn fixed_width(&self, width: usize) -> impl fmt::Display + '_ {
        FixedWidth { table: self, width }
    }
}

struct FixedWidth<'a> {
    table: &'a Table<'a>,
    width: usize,
}

impl fmt::Display for FixedWidth<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.table.render(Some(self.width), f)
    }
}

impl<'data> fmt::Display for Table<'data> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.render(None, f)
    }
}

pub fn data_table<'a, R: 'a>(input: impl IntoIterator<Item = &'a R>) -> Table<'a>
where
    R: IntoRow,
{
    let mut table = Table::new();
    for row in input {
        table.add_row(row.into_row());
    }
    table
}

#[derive(Debug, Clone)]
struct ColumnWidths(Vec<usize>);

impl ColumnWidths {
    fn new() -> Self {
        ColumnWidths(vec![])
    }

    /// Reset all columns to 0 and make sure there are the correct number of columns.
    fn reset(&mut self, num_cols: usize) {
        self.0.clear();
        self.0.resize(num_cols, 0);
    }

    fn fit_even_rows(&mut self, total_width: usize, cols: usize, border_width: usize) {
        // maybe turn this into something other than a panic.
        assert!(total_width >= border_width * (cols + 1));
        // there is 1 more border than number of cells.
        // Take off the border width to get the width of the cell interior.
        let cell_width = (total_width - border_width) / cols - border_width;
        let mut used_width = 0;
        let len = self.0.len();
        for col in &mut self.0[..len - 1] {
            *col = cell_width;
            used_width += cell_width + border_width;
        }
        // use remaining space for last cell.
        self.0[len - 1] = total_width - used_width - 2 * border_width;
    }

    /// Make our widths fit the given row with all text on a single line.
    ///
    /// This is for when we are allowed to use as much space as we want.
    fn fit_row_singleline(&mut self, row: &Row, border_width: usize) {
        let mut idx = 0;
        for cell in row.cells.iter() {
            if cell.col_span == 1 {
                self.0[idx] = self.0[idx].max(cell.min_width(true));
            } else {
                // space required to fit this cell (taking into account we have some borders to
                // use).
                let required_width = cell.min_width(true) - border_width * (cell.col_span - 1);
                let floor_per_cell = required_width / cell.col_span;
                // space we need to put somewhere
                let mut to_fit = required_width % cell.col_span;
                // split space evenly, with remainder in last space.
                for i in 0..cell.col_span {
                    let extra = if to_fit > 0 { 1 } else { 0 };
                    to_fit = to_fit.saturating_sub(1);
                    self.0[idx + i] = self.0[idx + 1].max(floor_per_cell + extra);
                }
            }
            idx += cell.col_span;
        }
    }
}

impl std::ops::Deref for ColumnWidths {
    type Target = [usize];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod test {

    use crate::cell::{Alignment, Cell};
    use crate::row::Row;
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
                    .with_cell(Cell::from("A").with_alignment(Alignment::Center))
                    .with_cell(Cell::from("B").with_alignment(Alignment::Center)),
            )
            .with_row(
                Row::new()
                    .with_cell(Cell::from(1.to_string()))
                    .with_cell(Cell::from("1")),
            )
            .with_row(
                Row::new()
                    .with_cell(Cell::from(2.to_string()))
                    .with_cell(Cell::from("10")),
            )
            .with_row(
                Row::new()
                    .with_cell(Cell::from(3.to_string()))
                    .with_cell(Cell::from("100")),
            );
        let expected = r"+---+-----+
| A |  B  |
| 1 | 1   |
| 2 | 10  |
| 3 | 100 |
+---+-----+
";
        println!("{}", table);
        assert_eq!(expected, table.to_string());
    }

    #[test]
    fn uneven_center_alignment() {
        let table = Table::new()
            .with_separate_rows(false)
            .with_style(TableStyle::SIMPLE)
            .with_row(Row::new().with_cell(Cell::from("A").with_alignment(Alignment::Center)))
            .with_row(Row::new().with_cell(Cell::from(11.to_string())))
            .with_row(Row::new().with_cell(Cell::from(2.to_string())))
            .with_row(Row::new().with_cell(Cell::from(3.to_string())));
        let expected = r"+----+
| A  |
| 11 |
| 2  |
| 3  |
+----+
";
        println!("{}", table);
        assert_eq!(expected, table.to_string());
    }

    #[test]
    fn uneven_center_alignment_2() {
        let table = Table::new()
            .with_separate_rows(false)
            .with_style(TableStyle::SIMPLE)
            .with_row(
                Row::new()
                    .with_cell(Cell::from("A1").with_alignment(Alignment::Center))
                    .with_cell(Cell::from("B").with_alignment(Alignment::Center)),
            );
        println!("{}", table);
        let expected = r"+----+---+
| A1 | B |
+----+---+
";
        assert_eq!(expected, table.to_string());
    }

    #[test]
    fn simple_table_style() {
        let mut table = Table::new().with_style(TableStyle::SIMPLE);

        add_data_to_test_table(&mut table);

        let expected = r"+------------------------------------------------------------------------------+
|                          This is some centered text                          |
+--------------------------------------+---------------------------------------+
| This is left aligned text            |            This is right aligned text |
+--------------------------------------+---------------------------------------+
| This is left aligned text            |            This is right aligned text |
+--------------------------------------+---------------------------------------+
| This is some really really really really really really really really really  |
| that is going to wrap to the next line                                       |
+------------------------------------------------------------------------------+
";
        let table = table.fixed_width(80);
        println!("{}", table);
        assert_eq!(expected, table.to_string());
    }

    #[test]
    #[ignore]
    fn uneven_with_varying_col_span() {
        let table = Table::new()
            .with_separate_rows(true)
            .with_style(TableStyle::SIMPLE)
            .with_row(
                Row::new()
                    .with_cell(Cell::from("A1111111").with_alignment(Alignment::Center))
                    .with_cell(Cell::from("B").with_alignment(Alignment::Center)),
            )
            .with_row(
                Row::new()
                    .with_cell(Cell::from(1.to_string()))
                    .with_cell(Cell::from("1")),
            )
            .with_row(
                Row::new()
                    .with_cell(Cell::from(2.to_string()))
                    .with_cell(Cell::from("10")),
            )
            .with_row(
                Row::new()
                    .with_cell(
                        Cell::from(3.to_string())
                            .with_alignment(Alignment::Left)
                            .with_padding(false),
                    )
                    .with_cell(Cell::from("100")),
            )
            .with_row(Row::new().with_cell(Cell::from("S").with_alignment(Alignment::Center)));
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
        println!("{}", table);
        assert_eq!(expected.trim(), table.to_string().trim());
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
                    .with_cell(Cell::from("A").with_alignment(Alignment::Center))
                    .with_cell(Cell::from("B").with_alignment(Alignment::Center)),
            )
            .with_row(
                Row::new()
                    .with_cell(Cell::from(1.to_string()))
                    .with_cell(Cell::from("1")),
            )
            .with_row(
                Row::new()
                    .with_cell(Cell::from(2.to_string()))
                    .with_cell(Cell::from("10")),
            )
            .with_row(
                Row::new()
                    .with_cell(Cell::from(3.to_string()))
                    .with_cell(Cell::from("100")),
            )
            .with_row(
                Row::new().with_cell(
                    Cell::from("Spanner")
                        .with_col_span(2)
                        .with_alignment(Alignment::Center),
                ),
            );
        let expected = "+-----+-----+
|  A  |  B  |
| 1   | 1   |
| 2   | 10  |
| 3   | 100 |
|  Spanner  |
+-----------+
";
        println!("{}", table);
        assert_eq!(expected.trim(), table.to_string().trim());
    }

    /*
        #[test]
        fn extended_table_style_wrapped() {
            let table = Table::new()
                .with_max_column_width(40)
                .with_max_widths_for_columns([(0, 1), (1, 1)])

            .with_style ( TableStyle::EXTENDED)

            .with_row(Row::new(vec![Cell::new_with_alignment(
                "This is some centered text",
                2,
                Alignment::Center,
            )]))

            .with_row(Row::new(vec![
                Cell::new("This is left aligned text"),
                Cell::new_with_alignment("This is right aligned text", 1, Alignment::Right),
            ]))

            .with_row(Row::new(vec![
                Cell::new("This is left aligned text"),
                Cell::new_with_alignment("This is right aligned text", 1, Alignment::Right),
            ]))

            .with_row(Row::new(vec![
                Cell::new_with_col_span("This is some really really really really really really really really really that is going to wrap to the next line\n1\n2", 2),
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
                    Cell::new_with_col_span("Col*1*Span*2", 2),
                    Cell::new("Col 2 Span 1"),
                    Cell::new_with_col_span("Col 3 Span 2", 2),
                    Cell::new("Col 4 Span 1"),
                ]));
                table.add_row(Row::new(vec![
                    Cell::new("Col 1 Span 1"),
                    Cell::new("Col 2 Span 1"),
                    Cell::new("Col 3 Span 1"),
                    Cell::new_with_col_span("Col 4 Span 1", 2),
                ]));
                table.add_row(Row::new(vec![
                    Cell::new("fasdaff"),
                    Cell::new("fff"),
                    Cell::new("fff"),
                ]));
                table.add_row(Row::new(vec![
                    Cell::new_with_alignment("fasdff", 3, Alignment::Right),
                    Cell::new_with_col_span("fffdff", 4),
                ]));
                table.add_row(Row::new(vec![
                    Cell::new("fasdsaff"),
                    Cell::new("fff"),
                    Cell::new("f\nf\nf\nfff\nrrr\n\n\n"),
                ]));
                table.add_row(Row::new(vec![Cell::new("fasdsaff")]));

                let s = table.render().clone();

                table.add_row(Row::new(vec![Cell::new_with_alignment(
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
                table.add_row(Row::new(vec![Cell::new("\u{1b}[31ma\u{1b}[0m")]));
                let expected = "╔═══╗
        ║ \u{1b}[31ma\u{1b}[0m ║
        ╚═══╝
        ";
                println!("{}", table.render());
                assert_eq!(expected, table.render());
            }
        */

    fn add_data_to_test_table(table: &mut Table) {
        table.add_row(
            Row::new().with_cell(
                Cell::from("This is some centered text")
                    .with_col_span(2)
                    .with_alignment(Alignment::Center),
            ),
        );

        table.add_row(
            Row::new()
                .with_cell(Cell::from("This is left aligned text"))
                .with_cell(
                    Cell::from("This is right aligned text").with_alignment(Alignment::Right),
                ),
        );

        table.add_row(
            Row::new()
                .with_cell(Cell::from("This is left aligned text"))
                .with_cell(
                    Cell::from("This is right aligned text").with_alignment(Alignment::Right),
                ),
        );

        table.add_row(
            Row::new().with_cell(
                Cell::from(
                    "This is some really really really really really \
                really really really really that is going to wrap to the next line",
                )
                .with_col_span(2),
            ),
        );
    }
}
