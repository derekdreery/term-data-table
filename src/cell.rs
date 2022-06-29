use itertools::Itertools;
use lazy_static;
use regex::Regex;
use std::{borrow::Cow, cell::RefCell, fmt, iter};
use unicode_width::UnicodeWidthChar;

use unicode_linebreak::{linebreaks, BreakOpportunity};
use unicode_width::UnicodeWidthStr;

/// Represents the horizontal alignment of content within a cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

///A table cell containing some str content.
///
///A cell may span multiple columns by setting the value of `col_span`.
///
///`pad_content` will add a space to either side of the cell's content.
#[derive(Debug, Clone)]
pub struct Cell<'txt> {
    pub(crate) content: Cow<'txt, str>,
    pub(crate) col_span: usize,
    pub(crate) alignment: Alignment,
    pub(crate) pad_content: bool,

    /// Positions we should split the text into multiple lines, if any.
    ///
    /// Is rebuild as needed.
    layout_newlines: RefCell<Option<Vec<usize>>>,

    content_without_ansi_esc: Option<String>,
}

impl<'txt> Default for Cell<'txt> {
    fn default() -> Self {
        Self {
            content: Cow::Borrowed(""),
            col_span: 1,
            alignment: Alignment::Left,
            pad_content: true,

            layout_newlines: RefCell::new(None),
            content_without_ansi_esc: None,
        }
    }
}

impl<'txt> Cell<'txt> {
    fn owned(content: String) -> Cell<'txt> {
        let mut this = Self {
            content: Cow::Owned(content),
            ..Default::default()
        };
        this.update_without_ansi_esc();
        this
    }

    /// Special builder that is slightly more efficient than using `From<String>`.
    fn borrowed(content: &'txt str) -> Self {
        let mut this = Self {
            content: Cow::Borrowed(content.as_ref()),
            ..Default::default()
        };
        this.update_without_ansi_esc();
        this
    }

    pub fn with_content(mut self, content: impl Into<Cow<'txt, str>>) -> Self {
        self.set_content(content);
        self
    }

    pub fn set_content(&mut self, content: impl Into<Cow<'txt, str>>) -> &mut Self {
        self.content = content.into();
        self.update_without_ansi_esc();
        self
    }

    fn content_for_layout(&self) -> &str {
        self.content_without_ansi_esc
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(&self.content)
    }

    fn update_without_ansi_esc(&mut self) {
        self.content_without_ansi_esc = if ANSI_ESC_RE.is_match(&self.content) {
            Some(ANSI_ESC_RE.split(&self.content).collect())
        } else {
            None
        };
    }

    /// Set the number of columns this cell spans.
    ///
    /// # Panics
    ///
    /// Will panic if `col_span == 0`.
    pub fn with_col_span(mut self, col_span: usize) -> Self {
        self.set_col_span(col_span);
        self
    }

    /// Set the number of columns this cell spans.
    ///
    /// # Panics
    ///
    /// Will panic if `col_span == 0`.
    pub fn set_col_span(&mut self, col_span: usize) -> &mut Self {
        assert!(col_span > 0, "cannot have a col_span of 0");
        self.col_span = col_span;
        *self.layout_newlines.borrow_mut() = None;
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.set_alignment(alignment);
        self
    }

    pub fn set_alignment(&mut self, alignment: Alignment) -> &mut Self {
        self.alignment = alignment;
        *self.layout_newlines.borrow_mut() = None;
        self
    }

    pub fn with_padding(mut self, padding: bool) -> Self {
        self.set_padding(padding);
        self
    }

    pub fn set_padding(&mut self, padding: bool) -> &mut Self {
        self.pad_content = padding;
        *self.layout_newlines.borrow_mut() = None;
        self
    }

    /// Calculate positions of newlines.
    ///
    /// Passed width includes padding spaces (if Some).
    ///
    /// Returns the total number of lines to be drawn.
    // The meaining of the parameter option None (means unbounded) is different from layout_width =
    // None (means cache is stale)
    pub(crate) fn layout(&self, width: Option<usize>) -> usize {
        // We can just pretend we have loads of space - we only calculate linebreaks here.
        let width = width.unwrap_or(usize::MAX);
        if width < 1 || (self.pad_content && width < 3) {
            panic!("cell too small to show anything");
        }
        let content_width = if self.pad_content {
            width.saturating_sub(2)
        } else {
            width
        };
        let mut ln = self.layout_newlines.borrow_mut();
        let ln = ln.get_or_insert(vec![]);
        ln.clear();
        ln.push(0);

        let mut s = self.content_for_layout();
        // Go through potential linebreak locations to find where we should break.
        let mut acc = 0;
        while let Some(idx) = next_linebreak(s, content_width) {
            s = &s[idx..];
            ln.push(idx + acc);
            acc += idx;
        }
        // the above method always ends the text with a newline, so pop it.
        ln.pop();
        // return number of lines
        ln.len()
    }

    /// The minium width required to display the cell correctly.
    ///
    /// If `only_mandatory` is passed, then only mandatory newlines will be considered, meaning the
    /// width will be larger.
    pub(crate) fn min_width(&self, only_mandatory: bool) -> usize {
        let content = self.content_for_layout();
        let max_newline_gap = linebreaks(content).filter_map(|(idx, ty)| {
            if only_mandatory && !matches!(ty, BreakOpportunity::Mandatory) {
                None
            } else {
                Some(idx)
            }
        });
        let max_newline_gap = iter::once(0)
            .chain(max_newline_gap)
            .chain(iter::once(content.len()))
            .tuple_windows()
            .map(|(start, end)| content[start..end].width())
            .max()
            .unwrap_or(0);

        // We need space for the padding if the user specified to use it.
        max_newline_gap + if self.pad_content { 2 } else { 0 }
    }

    /// Get the width of this cell, given the cell widths.
    ///
    /// Assumes slice starts at current cell, and returns slice starting at next cell.
    pub(crate) fn width<'s>(
        &self,
        border_width: usize,
        cell_widths: &'s [usize],
    ) -> (usize, &'s [usize]) {
        (
            cell_widths[..self.col_span].iter().copied().sum::<usize>()
                + border_width * self.col_span.saturating_sub(1),
            &cell_widths[self.col_span..],
        )
    }

    /// Write out the given line to the formatter.
    ///
    /// You must call `layout` (which lays out the text)  before calling this method, otherwise
    /// you may get panics or garbage.
    pub(crate) fn render_line(
        &self,
        line_idx: usize,
        width: usize,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        let newlines = self.layout_newlines.borrow();
        let newlines = newlines.as_ref().expect("missed call to `layout`");
        let line = match newlines.get(line_idx) {
            Some(&start_idx) => match newlines.get(line_idx + 1) {
                Some(&end_idx) => &self.content[start_idx..end_idx],
                None => &self.content[start_idx..],
            },
            // This will be the case if we already drew all the lines.
            None => "",
        };

        let (front_pad, back_pad) = self.get_padding(width, line.width());
        let edge = self.edge_char();
        f.write_str(edge)?;
        for _ in 0..front_pad {
            f.write_str(" ")?;
        }
        f.write_str(line)?;
        for _ in 0..back_pad {
            f.write_str(" ")?;
        }
        f.write_str(edge)
    }

    /// Returns the number of spaces that should be placed before and after the text (excluding the
    /// single padding char)
    ///
    /// line_width includes padding spaces
    fn get_padding(&self, width: usize, line_width: usize) -> (usize, usize) {
        let padding = if self.pad_content { 2 } else { 0 };
        let gap = (width - line_width).saturating_sub(padding);
        match self.alignment {
            Alignment::Left => (0, gap),
            Alignment::Center => (gap / 2, gap - gap / 2),
            Alignment::Right => (gap, 0),
        }
    }

    fn edge_char(&self) -> &'static str {
        if self.pad_content {
            " "
        } else {
            "\0"
        }
    }
}

impl<'txt> From<String> for Cell<'txt> {
    fn from(other: String) -> Self {
        Cell::owned(other)
    }
}

impl<'txt> From<&'txt String> for Cell<'txt> {
    fn from(other: &'txt String) -> Self {
        Cell::borrowed(other)
    }
}

impl<'txt> From<&'txt str> for Cell<'txt> {
    fn from(other: &'txt str) -> Self {
        Cell::borrowed(other)
    }
}

// Will match any ansi escape sequence.
// Taken from https://github.com/mitsuhiko/console
lazy_static! {
    static ref ANSI_ESC_RE: Regex =
        Regex::new(r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><]")
            .unwrap();
}

/// Find where to put the next linebreak, or return None if we don't need any more.
fn next_linebreak(text: &str, max_width: usize) -> Option<usize> {
    let mut prev = None;
    for (idx, ty) in linebreaks(text) {
        if text[..idx].width() > max_width {
            // use the previous linebreak or 1 char as a backup.
            return if let Some(prev) = prev {
                Some(prev)
            } else {
                text.chars().next().map(|ch| ch.width()).flatten()
            };
        } else if matches!(ty, BreakOpportunity::Mandatory) {
            // we must insert a linebreak here
            return Some(idx);
        } else {
            prev = Some(idx);
        }
    }
    None
}
