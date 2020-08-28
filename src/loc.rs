#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
    pub offset: usize,
    pub len: usize,
}

impl SourceLocation {
    /// Return the section of `source` corresponding to this location.
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.offset..self.end_offset()]
    }

    /// Split the text `source` into three sections: the text that came before the location,
    /// the text corresponding to the location, and the text after it.
    pub fn split_source<'a>(&self, source: &'a str) -> (&'a str, &'a str, &'a str) {
        let before = &source[..self.offset];
        let after = &source[self.end_offset()..];
        (before, self.text(source), after)
    }

    /// Return the line or lines that contain the object's location,
    /// and the location relative to that context.
    fn context(self, source: &str) -> (&str, SourceLocation) {
        let offset = source[..=self.offset]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let end_offset = source[self.end_offset()..]
            .find('\n')
            .unwrap_or(source.len());
        let len = end_offset - offset;
        let loc = SourceLocation { offset, len };
        (&source[offset..end_offset], loc)
    }

    fn end_offset(&self) -> usize {
        self.offset.saturating_add(self.len)
    }
}

pub trait Locate {
    fn location(&self) -> SourceLocation;
}

pub trait TryLocate {
    fn maybe_location(&self) -> Option<SourceLocation>;
}

impl<T: Locate> TryLocate for T {
    fn maybe_location(&self) -> Option<SourceLocation> {
        Some(self.location())
    }
}
