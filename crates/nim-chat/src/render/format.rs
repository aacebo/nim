#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Indent {
    Space(usize),
    Tab(usize),
}

pub struct Formatter<'a> {
    indent: Indent,
    at_line_start: bool,
    stream: &'a mut dyn std::io::Write,
}

impl<'a> Formatter<'a> {
    pub fn new(stream: &'a mut dyn std::io::Write) -> Self {
        Self {
            indent: Indent::Space(0),
            at_line_start: false,
            stream,
        }
    }

    pub fn indented(mut self, indent: Indent) -> Self {
        self.indent = indent;
        self
    }

    pub fn write(&mut self, data: impl std::fmt::Display) -> Result<(), std::io::Error> {
        self.write_indented(&data.to_string())
    }

    pub fn writeln(&mut self, data: impl std::fmt::Display) -> Result<(), std::io::Error> {
        self.write(data)?;
        self.write("\n")
    }

    pub fn write_debug(&mut self, data: impl std::fmt::Debug) -> Result<(), std::io::Error> {
        self.write_indented(&format!("{data:#?}"))
    }

    pub fn writeln_debug(&mut self, data: impl std::fmt::Debug) -> Result<(), std::io::Error> {
        self.write_debug(data)?;
        self.write("\n")
    }

    fn write_indented(&mut self, data: &str) -> Result<(), std::io::Error> {
        for line in data.split_inclusive('\n') {
            if self.at_line_start && line != "\n" {
                self.write_indent()?;
            }

            std::io::Write::write_all(self.stream, line.as_bytes())?;
            self.at_line_start = line.ends_with('\n');
        }

        Ok(())
    }

    fn write_indent(&mut self) -> Result<(), std::io::Error> {
        match self.indent {
            Indent::Space(count) => std::io::Write::write_all(self.stream, &vec![b' '; count]),
            Indent::Tab(count) => std::io::Write::write_all(self.stream, &vec![b'\t'; count]),
        }
    }
}
