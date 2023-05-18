//! Writing records into a JSON file.
use json::JsonValue;

use crate::util::ObjectReference;

use super::{record::Record, RecordWriter};

/// This writes the heap-dump records into a custom application/json-seq format.
pub struct JsonSeqWriter {
    file: Box<dyn std::io::Write>,
}

impl JsonSeqWriter {
    const START_OF_RECORD: char = '\x1e';
    const END_OF_RECORD: char = '\x0a';

    pub fn new(file: Box<dyn std::io::Write>) -> Self {
        Self { file }
    }

    // fn maybe_to_yaml_string(maybe_string: Option<String>) -> String {
    //     maybe_string.map(Self::to_yaml_string).unwrap_or_else(|| "null".to_string())
    // }

    // fn to_yaml_string(string: String) -> String {
    //     let mut out = String::new();
    //     {
    //         let mut yaml_emitter = YamlEmitter::new(&mut out);
    //         yaml_emitter.compact(true);
    //         yaml_emitter.dump(&Yaml::String(string)).unwrap();
    //     }
    //     out
    // }
}

impl RecordWriter for JsonSeqWriter {
    fn flush(&mut self) {
        self.file.flush().unwrap();
    }

    fn write_record(&mut self, record: Record) {
        write!(self.file, "{}", Self::START_OF_RECORD).unwrap();
        let json_value = match record {
            Record::Root { objref, pinning } => {
                json::object! {
                    event: "Root",
                    objref: objref,
                    pinning: pinning,
                }
            }
            Record::Node {
                objref,
                pinned,
                type_string,
            } => {
                json::object! {
                    event: "Node",
                    objref: objref,
                    pinned: pinned,
                    type_string: type_string
                }
            }
            Record::Edge { from, to, valid } => {
                json::object! {
                    event: "Edge",
                    from: from,
                    to: to,
                    valid: valid,
                }
            }
            Record::Forward { from, to } => {
                json::object! {
                    event: "Forward",
                    from: from,
                    to: to,
                }
            }
            Record::Resurrect { objref } => {
                json::object! {
                    event: "Resurrect",
                    objref: objref,
                }
            }
        };
        write!(self.file, "{}", json_value).unwrap();
        write!(self.file, "{}", Self::END_OF_RECORD).unwrap();
    }
}

impl From<ObjectReference> for JsonValue {
    fn from(value: ObjectReference) -> Self {
        JsonValue::Number(value.to_raw_address().as_usize().into())
    }
}
