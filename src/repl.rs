use std::io;

pub struct ReplStream {
    line: String,
}

impl ReplStream {
    pub fn new() -> Self {
        let mut ret = ReplStream {
            line: String::new(),
        };
        ret.next_line();
        ret
    }

    pub fn next_line(&mut self) {
        io::stdin().read_line(&mut self.line);
    }

    pub fn iter(&mut self) -> ReplScanner {
        ReplScanner {
            stream: self,
            line: &self.line,
        }
    }
}

pub struct ReplScanner<'a> {
    stream: &'a mut ReplStream,
    line: &'a str,
}

impl<'a> Iterator for ReplScanner<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        Some(&self.stream.line)
    }
}
