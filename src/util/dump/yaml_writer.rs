//! Writing records into a YAML file.

use std::{fs::File, io::Write};

use super::RecordWriter;

pub struct YamlWriter {
    file: File,
}

impl YamlWriter {
    pub fn new(file: File) -> Self {
        Self { file }
    }
}

impl RecordWriter for YamlWriter {
    fn write_record(&mut self, record: super::record::Record) {
        match record {
            super::record::Record::Node {
                objref,
                pinned,
                root,
            } => {
                write!(
                    self.file,
                    "
- event: Node
  objref: {objref}
  pinned: {pinned}
  root: {root}
"
                )
                .unwrap();
            }
            super::record::Record::Edge { from, to, valid } => {
                write!(
                    self.file,
                    "
- event: Edge
  from: {from}
  to: {to}
  valid: {valid}
"
                )
                .unwrap();
            }
            super::record::Record::Forward { from, to } => {
                write!(
                    self.file,
                    "
- event: Forward
  from: {from}
  to: {to}
"
                )
                .unwrap();
            }
            super::record::Record::Resurrect { objref } => {
                write!(
                    self.file,
                    "
- event: Resurrect
  objref: {objref}
"
                )
                .unwrap();
            }
        }
    }

    fn flush(&mut self) {
        self.file.flush().unwrap();
    }
}
