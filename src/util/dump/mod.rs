//! This mod contains heap-dumping facilities for debugging.

use std::{sync::Mutex, fs::File, path::Path};

use self::record::Record;

pub mod record;
pub mod yaml_writer;

pub trait RecordWriter {
    fn write_record(&mut self, record: Record);
    fn flush(&mut self);
}

pub struct HeapDumper {
    sync: Mutex<HeapDumperSync>,
}

struct HeapDumperSync {
    writer: Option<Box<dyn RecordWriter>>,
}

impl Default for HeapDumper {
    fn default() -> Self {
        Self {
            sync: Mutex::new(HeapDumperSync {
                writer: None,
            }),
        }
    }
}

impl HeapDumper {
    pub fn start_recording(&self, gc_count: usize) {
        let mut sync = self.sync.lock().unwrap();
        assert!(sync.writer.is_none());

        let file = File::open(format!("mmtk-heap-dump-{}.yml", gc_count)).unwrap();
        sync.writer = Some(Box::new(yaml_writer::YamlWriter::new(file)));
    }

    pub fn finish_recording(&self) {
        let mut sync = self.sync.lock().unwrap();
        assert!(sync.writer.is_some());

        sync.writer.as_mut().unwrap().flush();
        sync.writer = None;
    }
}
