use lazy_static;
use regex::Regex;
use std::{borrow::Cow, cmp, collections::HashSet};

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Represents the horizontal alignment of content within a cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

///A table cell containing some str data.
///
///A cell may span multiple columns by setting the value of `col_span`.
///
///`pad_content` will add a space to either side of the cell's content.AsRef
#[derive(Debug, Clone)]
pub struct TableCell<'data> {
    pub(crate) data: Cow<'data, str>,
    pub(crate) col_span: usize,
    pub(crate) alignment: Alignment,
    pub(crate) pad_content: bool,
}

impl<'data> Default for TableCell<'data> {
    fn default() -> Self {
        Self {
            data: Cow::Borrowed(""),
            col_span: 1,
            alignment: Alignment::Left,
            pad_content: true,
        }
    }
}

impl<'data> TableCell<'data> {
    fn owned(data: String) -> TableCell<'data> {
        Self {
            data: Cow::Owned(data),
            ..Default::default()
        }
    }

    /// Special builder that is slightly more efficient than using `From<String>`.
    fn borrowed(data: &'data str) -> Self {
        Self {
            data: Cow::Borrowed(data.as_ref()),
            ..Default::default()
        }
    }

    pub fn with_col_span(mut self, col_span: usize) -> Self {
        self.set_col_span(col_span);
        self
    }

    pub fn set_col_span(&mut self, col_span: usize) -> &mut Self {
        self.col_span = col_span;
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.set_alignment(alignment);
        self
    }

    pub fn set_alignment(&mut self, alignment: Alignment) -> &mut Self {
        self.alignment = alignment;
        self
    }

    pub fn with_padding(mut self, padding: bool) -> Self {
        self.set_padding(padding);
        self
    }

    pub fn set_padding(&mut self, padding: bool) -> &mut Self {
        self.pad_content = padding;
        self
    }

    /// Calculates the width of the cell.
    ///
    /// New line characters are taken into account during the calculation.
    pub fn width(&self) -> usize {
        let wrapped = self.wrapped_content(std::usize::MAX);
        let mut max = 0;
        for s in wrapped {
            let str_width = string_width(&s);
            max = cmp::max(max, str_width);
        }
        max
    }

    /// The width of the cell's content divided by its `col_span` value.
    pub fn split_width(&self) -> f32 {
        let res = self.width() as f32 / self.col_span as f32;
        res
    }

    /// The minium width required to display the cell properly
    pub fn min_width(&self) -> usize {
        let mut max_char_width: usize = 0;
        for c in self.data.chars() {
            max_char_width = cmp::max(max_char_width, c.width().unwrap_or(1) as usize);
        }

        if self.pad_content {
            max_char_width + ' '.width().unwrap_or(1) as usize * 2
        } else {
            max_char_width
        }
    }

    /// Wraps the cell's content to the provided width.
    ///
    /// New line characters are taken into account.
    pub fn wrapped_content(&self, width: usize) -> Vec<String> {
        let pad_char = if self.pad_content { ' ' } else { '\0' };
        let hidden: HashSet<usize> = STRIP_ANSI_RE
            .find_iter(&self.data)
            .flat_map(|m| m.start()..m.end())
            .collect();
        let mut res: Vec<String> = Vec::new();
        let mut buf = String::new();
        buf.push(pad_char);
        let mut byte_index = 0;
        for c in self.data.chars() {
            if !hidden.contains(&byte_index)
                && (string_width(&buf) >= width - pad_char.width().unwrap_or(1) || c == '\n')
            {
                buf.push(pad_char);
                res.push(buf);
                buf = String::new();
                buf.push(pad_char);
                if c == '\n' {
                    byte_index += 1;
                    continue;
                }
            }
            byte_index += c.len_utf8();
            buf.push(c);
        }
        buf.push(pad_char);
        res.push(buf);

        res
    }
}

impl<'data> From<String> for TableCell<'data> {
    fn from(other: String) -> Self {
        TableCell::owned(other)
    }
}

impl<'data> From<&'data String> for TableCell<'data> {
    fn from(other: &'data String) -> Self {
        TableCell::borrowed(other)
    }
}

impl<'data> From<&'data str> for TableCell<'data> {
    fn from(other: &'data str) -> Self {
        TableCell::borrowed(other)
    }
}

// Taken from https://github.com/mitsuhiko/console
lazy_static! {
    static ref STRIP_ANSI_RE: Regex =
        Regex::new(r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]")
            .unwrap();
}

// The width of a string. Strips ansi characters
pub fn string_width(string: &str) -> usize {
    let stripped = STRIP_ANSI_RE.replace_all(string, "");
    stripped.width()
}
