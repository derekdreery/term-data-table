use crate::{Cell, TableStyle};
use itertools::Itertools;
use std::fmt::{self, Write};

/// A set of table cells
#[derive(Debug, Clone)]
pub struct Row<'data> {
    pub(crate) cells: Vec<Cell<'data>>,
    /// Whether the row should have a top border or not
    pub(crate) has_separator: bool,
}

impl<'data> Default for Row<'data> {
    fn default() -> Self {
        Self {
            cells: vec![],
            has_separator: true,
        }
    }
}

impl<'data> Row<'data> {
    pub fn new() -> Self {
        Default::default()
    }

    /// Whether the row should have a top border or not
    pub fn with_separator(mut self, has_separator: bool) -> Self {
        self.set_has_separator(has_separator);
        self
    }

    /// Whether the row should have a top border or not
    pub fn set_has_separator(&mut self, has_separator: bool) -> &mut Self {
        self.has_separator = has_separator;
        self
    }

    pub fn add_cell(&mut self, cell: impl Into<Cell<'data>>) -> &mut Self {
        self.cells.push(cell.into());
        self
    }

    pub fn with_cell(mut self, cell: impl Into<Cell<'data>>) -> Self {
        self.add_cell(cell);
        self
    }

    /// Number of columns in this row, taking into account col_span > 1.
    pub(crate) fn columns(&self) -> usize {
        self.cells.iter().map(|cell| cell.col_span).sum()
    }

    /// Ask the row to calculate its layout.
    ///
    /// Returns the number of lines required to display this row (without the top border).
    pub(crate) fn layout(&self, column_widths: &[usize], border_width: usize) -> usize {
        let mut max_lines = 0;
        let mut idx = 0;
        dbg!(column_widths);
        for cell in self.cells.iter() {
            // start with the extra space for borders
            let mut width = (cell.col_span + 1) * border_width;

            // add in space for cell content.
            for w in column_widths[idx..idx + cell.col_span].iter().copied() {
                width += w;
            }
            let num_lines = cell.layout(Some(width));
            idx += cell.col_span;
            max_lines = max_lines.max(num_lines);
        }
        max_lines
    }

    pub fn render_top_separator(
        &self,
        cell_widths: &[usize],
        style: &TableStyle,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        if !self.has_separator {
            // don't draw anything
            return Ok(());
        }
        // special-case the first cell
        f.write_char(style.top_left_corner)?;
        let mut widths = cell_widths;
        let mut width;
        let mut cells = self.cells.iter();
        if let Some(first_cell) = cells.next() {
            (width, widths) = first_cell.width(style.border_width(), widths);
            for _ in 0..width {
                f.write_char(style.horizontal)?;
            }
        }
        for cell in cells {
            f.write_char(style.outer_top_horizontal)?;
            (width, widths) = cell.width(style.border_width(), widths);
            for _ in 0..width {
                f.write_char(style.horizontal)?;
            }
        }
        f.write_char(style.top_right_corner)?;
        writeln!(f)
    }

    pub fn render_bottom_separator(
        &self,
        cell_widths: &[usize],
        style: &TableStyle,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        if !self.has_separator {
            // don't draw anything
            return Ok(());
        }
        // special-case the first cell
        f.write_char(style.bottom_left_corner)?;
        let mut widths = cell_widths;
        let mut width;
        let mut cells = self.cells.iter();
        if let Some(first_cell) = cells.next() {
            (width, widths) = first_cell.width(style.border_width(), widths);
            for _ in 0..width {
                f.write_char(style.horizontal)?;
            }
        }
        for cell in cells {
            f.write_char(style.outer_bottom_horizontal)?;
            (width, widths) = cell.width(style.border_width(), widths);
            for _ in 0..width {
                f.write_char(style.horizontal)?;
            }
        }
        f.write_char(style.bottom_right_corner)?;
        writeln!(f)
    }

    pub fn render_separator(
        &self,
        prev: &Row,
        cell_widths: &[usize],
        style: &TableStyle,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        if !self.has_separator {
            // don't draw anything
            return Ok(());
        }
        f.write_char(style.outer_left_vertical)?;
        let mut iter = cell_widths
            .iter()
            .copied()
            .zip(self.iter_junctions(prev).skip(1))
            .peekable();
        while let Some((width, borders)) = iter.next() {
            for _ in 0..width {
                f.write_char(style.horizontal)?;
            }
            f.write_char(borders.joiner(style, iter.peek().is_none()))?;
        }
        writeln!(f)
    }

    /// Formats a row based on the provided table style
    pub(crate) fn render_content(
        &self,
        column_widths: &[usize],
        num_lines: usize,
        style: &TableStyle,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        for line_num in 0..num_lines {
            let mut width;
            let mut widths = column_widths;
            for cell in &self.cells {
                f.write_char(style.vertical)?;
                (width, widths) = cell.width(style.border_width(), widths);
                cell.render_line(line_num, width, f)?;
            }
            f.write_char(style.vertical)?;
            writeln!(f)?;
        }
        Ok(())
    }
    /// Number of columns in the row.
    ///
    /// This is the sum of all cell's col_span values
    pub fn num_columns(&self) -> usize {
        self.cells.iter().map(|x| x.col_span).sum()
    }

    /// What kind of join is at the beginning of each cell.
    fn iter_joins(&'data self) -> impl Iterator<Item = BorderTy> + 'data {
        struct IterJoins<'a> {
            inner: std::slice::Iter<'a, Cell<'a>>,
            // cols_remaining == 0 means we are past the end.
            cols_remaining: usize,
        }

        impl<'a> Iterator for IterJoins<'a> {
            type Item = BorderTy;
            fn next(&mut self) -> Option<Self::Item> {
                let out = Some(match &mut self.cols_remaining {
                    // we are past the end
                    0 => BorderTy::Empty,
                    // we are at the end of a cell
                    n @ 1 => {
                        *n = self.inner.next().map(|cell| cell.col_span).unwrap_or(0);
                        BorderTy::End
                    }
                    // we are in the middle of a cell
                    n => {
                        *n -= 1;
                        BorderTy::Middle
                    }
                });
                out
            }
        }
        IterJoins {
            inner: self.cells.iter(),
            // start as if we just finished a cell
            cols_remaining: 1,
        }
    }

    /// The correct border given the previous and next rows.
    fn iter_junctions(&'data self, prev: &'data Self) -> impl Iterator<Item = Borders> + 'data {
        prev.iter_joins()
            .zip(self.iter_joins())
            .map(|(above, below)| {
                let borders = Borders { above, below };
                if borders == Borders::EMPTY {
                    None
                } else {
                    Some(borders)
                }
            })
            .while_some()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Borders {
    above: BorderTy,
    below: BorderTy,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum BorderTy {
    Empty,
    Middle,
    End,
}

impl Borders {
    const EMPTY: Borders = Borders {
        above: BorderTy::Empty,
        below: BorderTy::Empty,
    };

    fn joiner(&self, style: &TableStyle, final_end: bool) -> char {
        use BorderTy::*;
        match (self.above, self.below) {
            (Empty, Empty) => unreachable!(),
            (Empty, Middle) | (Middle, Empty) | (Middle, Middle) => style.horizontal,
            (Empty, End) | (Middle, End) => {
                if final_end {
                    style.top_right_corner
                } else {
                    style.outer_top_horizontal
                }
            }
            (End, Empty) | (End, Middle) => {
                if final_end {
                    style.bottom_right_corner
                } else {
                    style.outer_bottom_horizontal
                }
            }
            (End, End) => {
                if final_end {
                    style.outer_right_vertical
                } else {
                    style.intersection
                }
            }
        }
    }
}

// ------------------

/// A trait for types that know how to turn themselves into a table row.
///
/// Note that the tuple implementations of these methods always copy strings.
pub trait IntoRow {
    /// Returns a set of cells that can be used as headers for the cells of data of this type.
    fn headers(&self) -> Row;
    /// Returns the row.
    fn into_row(&self) -> Row;
}

macro_rules! impl_row_for_tuple {
    () => {};

    (($first_label:expr, $first_ty:ident) $(,($rest_label:expr, $rest_ty:ident))*) => {
        impl<$first_ty, $($rest_ty,)*> IntoRow for ($first_ty, $($rest_ty),*)
            where $first_ty: ::std::fmt::Display,
                  $(
                      $rest_ty: ::std::fmt::Display,
                  )*
        {
            fn headers(&self) -> Row {
                let mut row = Row::default();
                row.add_cell(stringify!($first_ty));
                $(
                    row.add_cell(stringify!($rest_ty));
                )*
                row
            }

            fn into_row(&self) -> Row {
                #[allow(non_snake_case)]
                let (
                    ref $first_ty,
                    $(
                        ref $rest_ty
                    ),*
                ) = &self;
                let mut row = Row::default();
                row.add_cell($first_ty.to_string());
                $(
                    row.add_cell($rest_ty.to_string());
                )*
                row
            }
        }

        impl_row_for_tuple!($(($rest_label, $rest_ty)),*);
    };
}

impl_row_for_tuple!(
    ("_0", D0),
    ("_1", D1),
    ("_2", D2),
    ("_3", D3),
    ("_4", D4),
    ("_5", D5),
    ("_6", D6),
    ("_7", D7),
    ("_8", D8),
    ("_9", D9)
);
