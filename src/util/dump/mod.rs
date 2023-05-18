//! This mod contains heap-dumping facilities for debugging.

use std::{fs::File, io::BufWriter , sync::Mutex};

use self::record::Record;

use super::ObjectReference;

pub mod record;
pub mod json_writer;

pub trait RecordWriter {
    fn write_record(&mut self, record: Record);
    fn flush(&mut self);
}

pub struct HeapDumper {
    sync: Mutex<HeapDumperSync>,
}

unsafe impl Sync for HeapDumper {}

struct HeapDumperSync {
    writer: Option<Box<dyn RecordWriter>>,
}

impl Default for HeapDumper {
    fn default() -> Self {
        Self {
            sync: Mutex::new(HeapDumperSync { writer: None }),
        }
    }
}

impl HeapDumper {
    pub fn start_recording(&self, gc_count: usize) {
        let mut sync = self.sync.lock().unwrap();
        assert!(sync.writer.is_none());

        let file_name = format!("mmtk-heap-dump-{}.json", gc_count);
        info!("Starting recording heap dump. File: {file_name}");

        let file = File::create(file_name).unwrap();
        let buf_writer = BufWriter::new(file);
        sync.writer = Some(Box::new(json_writer::JsonSeqWriter::new(Box::new(buf_writer))));
    }

    pub fn finish_recording(&self) {
        let mut sync = self.sync.lock().unwrap();
        assert!(sync.writer.is_some());

        sync.writer.as_mut().unwrap().flush();
        sync.writer = None;

        info!("Finished recording heap dump.");
    }

    pub fn write_many(&self, records: Vec<Record>) {
        let mut sync = self.sync.lock().unwrap();
        assert!(sync.writer.is_some());

        let writer = sync.writer.as_mut().unwrap();
        for record in records {
            writer.write_record(record);
        }
        writer.flush();
    }
}

pub struct HeapDumperLocal {
    buffer: Vec<Record>,
    cur_obj: Option<ObjectReference>,
}

impl Default for HeapDumperLocal {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            cur_obj: None,
        }
    }
}

impl HeapDumperLocal {
    pub fn add_record(&mut self, record: Record) {
        self.buffer.push(record);
    }

    pub fn set_cur_obj(&mut self, obj: ObjectReference) {
        self.cur_obj = Some(obj);
    }

    pub fn clear_cur_obj(&mut self) {
        self.cur_obj = None;
    }

    pub fn add_edge_from_cur_obj(&mut self, to: ObjectReference, valid: bool) {
        let from = self.cur_obj.unwrap();
        self.add_record(Record::Edge { from, to, valid })
    }

    pub fn flush(&mut self, heap_dumper: &HeapDumper) {
        heap_dumper.write_many(std::mem::take(&mut self.buffer));
    }
}
