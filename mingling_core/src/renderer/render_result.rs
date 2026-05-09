use std::{
    fmt::{Display, Formatter},
    io::Write,
    ops::Deref,
};

/// Render result, containing the rendered text content.
#[derive(Default, Debug, PartialEq)]
pub struct RenderResult {
    render_text: String,
    pub exit_code: i32,
}

impl Write for RenderResult {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = std::str::from_utf8(buf).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "not valid UTF-8")
        })?;
        self.render_text.push_str(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Display for RenderResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.render_text.trim())
    }
}

impl Deref for RenderResult {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.render_text
    }
}

impl RenderResult {
    /// Appends the given text to the rendered content.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    /// use std::ops::Deref;
    ///
    /// let mut result = RenderResult::default();
    /// result.print("Hello");
    /// result.print(", world!");
    /// assert_eq!(result.deref(), "Hello, world!");
    /// ```
    pub fn print(&mut self, text: &str) {
        self.render_text.push_str(text);
    }

    /// Appends the given text followed by a newline to the rendered content.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    /// use std::ops::Deref;
    ///
    /// let mut result = RenderResult::default();
    /// result.println("First line");
    /// result.println("Second line");
    /// assert_eq!(result.deref(), "First line\nSecond line\n");
    /// ```
    pub fn println(&mut self, text: &str) {
        self.render_text.push_str(text);
        self.render_text.push('\n');
    }

    /// Clears all rendered content.
    ///
    /// # Examples
    ///
    /// ```
    /// use mingling_core::RenderResult;
    /// use std::ops::Deref;
    ///
    /// let mut result = RenderResult::default();
    /// result.print("Some content");
    /// assert!(!result.is_empty());
    /// result.clear();
    /// assert!(result.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.render_text.clear();
    }
}
