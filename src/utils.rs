#[derive(Default, Debug, PartialEq, Clone)]
pub struct Span {
    file: String,
    start: usize,
    end: usize,
}

impl ariadne::Span for Span {
    type SourceId = String;

    fn source(&self) -> &Self::SourceId {
        &self.file
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

// The span is inclusive for start but exclusive for end [start, end)
// Used to hold location information in case of lexing errors
impl Span {
    pub fn new(file: String, start: usize, end: usize) -> Self {
        Self { file, start, end }
    }
}
