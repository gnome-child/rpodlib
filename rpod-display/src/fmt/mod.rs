use std::fmt::{Arguments, Debug, Display, Formatter, Result, Write};

pub mod album_item;
pub mod data_object;
pub mod list;
pub mod playlist_item;
pub mod root;
pub mod track_item;

#[derive(Clone)]
pub enum LineType {
    Title,
    Body,
    Empty,
}

#[derive(Clone)]
pub enum Row {
    Line { label: String, value: String },
    Raw(String),
}

pub trait TreeDisplay {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result;
}

pub struct TreeView<'a, T: ?Sized> {
    inner: &'a T,
    ctx: TreeContext,
}

impl<'a, T: TreeDisplay + ?Sized> Display for TreeView<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.inner.tree_fmt(f, &self.ctx)
    }
}

pub trait TreeDisplayExt: TreeDisplay {
    fn view<'a>(&'a self, ctx: TreeContext) -> TreeView<'a, Self>
    where
        Self: Sized,
    {
        TreeView { inner: self, ctx }
    }

    fn to_tree_string(&self, ctx: TreeContext) -> String
    where
        Self: Sized,
    {
        format!("{}", self.view(ctx))
    }

    fn to_tree_string_pretty(&self, ctx: TreeContext) -> String
    where
        Self: Sized,
    {
        // `{:#}` sets `f.alternate()` which you already use to print children
        format!("{:#}", self.view(ctx))
    }
}
impl<T: TreeDisplay + ?Sized> TreeDisplayExt for T {}

#[derive(Clone)]
pub struct Glyphs {
    empty: &'static str,
    pipe: &'static str,
    branch_top: &'static str,
    branch_mid: &'static str,
    branch_end: &'static str,
    branch_indent: &'static str,
}

impl Glyphs {
    const UNICODE: Self = Self {
        empty: "    ",
        pipe: "│   ",
        branch_top: "┌── ",
        branch_mid: "├───",
        branch_end: "└───",
        branch_indent: "┬── ",
    };

    const ASCII: Self = Self {
        empty: "    ",
        pipe: "|   ",
        branch_top: ".-- ",
        branch_mid: "|---",
        branch_end: "'---",
        branch_indent: "+── ",
    };
}

#[derive(Clone)]
pub struct Body {
    rows: Vec<Row>,
    cached_width: Option<usize>,
}

impl Body {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            cached_width: None,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            rows: Vec::with_capacity(cap),
            cached_width: None,
        }
    }

    pub fn reset_cached_width(&mut self) {
        self.cached_width = None;
    }

    pub fn width(&mut self) -> usize {
        if let Some(width) = self.cached_width {
            return width;
        }

        let width = self
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Line { label, .. } => Some(label.len()),
                Row::Raw(_) => None,
            })
            .max()
            .unwrap_or(0);
        self.cached_width = Some(width);
        self.width()
    }

    pub fn push(&mut self, label: impl AsRef<str>, value: impl Display) -> &mut Self {
        self.rows.push(Row::Line {
            label: label.as_ref().to_string(),
            value: value.to_string(),
        });
        self.reset_cached_width();
        self
    }

    pub fn push_debug(&mut self, label: impl AsRef<str>, value: impl Debug) -> &mut Self {
        self.rows.push(Row::Line {
            label: label.as_ref().to_string(),
            value: format!("{value:?}"),
        });
        self.reset_cached_width();
        self
    }

    pub fn push_fmt(&mut self, label: impl AsRef<str>, value: Arguments<'_>) -> &mut Self {
        let mut str = String::new();
        str.write_fmt(value).unwrap();

        self.rows.push(Row::Line {
            label: label.as_ref().to_string(),
            value: str,
        });
        self.reset_cached_width();
        self
    }

    pub fn push_raw(&mut self, s: impl Into<String>) -> &mut Self {
        self.rows.push(Row::Raw(s.into()));
        self.reset_cached_width();
        self
    }
}

#[derive(Clone)]
pub struct TreeContext {
    glyphs: Glyphs,
    last_stack: Vec<bool>,
}

impl TreeContext {
    pub fn begin_unicode() -> Self {
        Self {
            glyphs: Glyphs::UNICODE,
            last_stack: Vec::with_capacity(16),
        }
    }

    pub fn begin_ascii() -> Self {
        Self {
            glyphs: Glyphs::ASCII,
            last_stack: Vec::with_capacity(16),
        }
    }

    pub fn descend(&self, is_last: bool) -> Self {
        let mut next = self.clone();

        next.last_stack.push(is_last);
        next
    }

    pub fn depth(&self) -> usize {
        self.last_stack.len()
    }

    pub fn writeln_title(&self, f: &mut Formatter<'_>, s: impl Display) -> Result {
        self.write_prefix(f, LineType::Title)?;

        writeln!(f, "{s}")
    }

    pub fn writeln_body(&self, f: &mut Formatter<'_>, s: impl Display) -> Result {
        self.write_prefix(f, LineType::Body)?;

        writeln!(f, "{s}")
    }

    pub fn writeln_empty(&self, f: &mut Formatter<'_>, s: impl Display) -> Result {
        self.write_prefix(f, LineType::Empty)?;

        writeln!(f, "{s}")
    }

    pub fn write_body(&self, f: &mut Formatter<'_>, body: &mut Body) -> Result {
        if body.rows.is_empty() {
            return Ok(());
        }

        let width = body.width();

        for row in &body.rows {
            match row {
                Row::Line { label, value } => {
                    let mut lines = value.lines();

                    if let Some(first) = lines.next() {
                        self.write_prefix(f, LineType::Body)?;

                        writeln!(f, "{label:<width$} {first}", width = width)?;
                    }

                    for line in lines {
                        self.write_prefix(f, LineType::Body)?;

                        writeln!(f, "{:width$} {line}", "", width = width)?;
                    }
                }

                Row::Raw(value) => {
                    for line in value.lines() {
                        self.write_prefix(f, LineType::Body)?;

                        writeln!(f, "{line}")?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn write_prefix(&self, f: &mut Formatter<'_>, line_type: LineType) -> Result {
        let depth = self.depth();

        if depth == 0 {
            if let LineType::Title = line_type {
                f.write_str(self.glyphs.branch_top)?;
            }
            return Ok(());
        }

        for &is_last in &self.last_stack[..depth - 1] {
            if is_last {
                f.write_str(self.glyphs.empty)?;
            } else {
                f.write_str(self.glyphs.pipe)?;
            }
        }

        match line_type {
            LineType::Title => {
                let is_last = *self.last_stack.last().unwrap();

                f.write_str(if is_last {
                    self.glyphs.branch_end
                } else {
                    self.glyphs.branch_mid
                })?;
                f.write_str(self.glyphs.branch_indent)
            }

            LineType::Body => {
                let is_last = *self.last_stack.last().unwrap();

                f.write_str(if is_last {
                    self.glyphs.empty
                } else {
                    self.glyphs.pipe
                })
            }

            LineType::Empty => f.write_str(self.glyphs.pipe),
        }
    }
}
