//! A writer that doesn't write to anywhere.

use super::RecordWriter;

pub struct NoopWriter;

impl RecordWriter for NoopWriter {
    fn write_record(&mut self, _record: super::record::Record) {
        // Do nothing.
    }

    fn flush(&mut self) {
        // Do nothing.
    }
}
