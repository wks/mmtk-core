pub(crate) struct Batcher<T, F: FnMut(Vec<T>)> {
    batch_size: usize,
    callback: F,
    buffer: Vec<T>,
}

impl<T, F: FnMut(Vec<T>)> Batcher<T, F> {
    pub fn new(batch_size: usize, callback: F) -> Self {
        Self {
            batch_size,
            callback,
            buffer: Vec::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        if self.buffer.is_empty() {
            self.buffer.reserve_exact(self.batch_size);
        }

        self.buffer.push(item);

        if self.buffer.len() == self.batch_size {
            self.flush();
        }

        debug_assert!(self.buffer.len() < self.batch_size);
    }

    pub fn finish(mut self) {
        if !self.buffer.is_empty() {
            self.flush();
        }

        debug_assert!(self.buffer.is_empty());
    }

    fn flush(&mut self) {
        let current_batch = std::mem::take(&mut self.buffer);
        (self.callback)(current_batch);
    }
}

#[cfg(test)]
fn make_typical_batches(batch_size: usize, items: usize, finish: bool) -> Vec<Vec<usize>> {
    let mut batches = vec![];
    let mut batcher = Batcher::new(batch_size, |batch| {
        batches.push(batch);
    });

    for i in 0..items {
        batcher.push(i);
    }

    if finish {
        batcher.finish();
    }

    batches
}

#[test]
fn test_not_full() {
    let batches = make_typical_batches(5, 3, false);
    assert!(batches.is_empty());
}

#[test]
fn test_full() {
    let batches = make_typical_batches(5, 5, false);
    assert_eq!(batches.as_slice(), vec![vec![0, 1, 2, 3, 4]].as_slice());
}

#[test]
fn test_multiple_batches() {
    let batches = make_typical_batches(3, 9, false);

    assert_eq!(
        batches.as_slice(),
        vec![vec![0, 1, 2], vec![3, 4, 5], vec![6, 7, 8]].as_slice()
    );
}

#[test]
fn test_remainder() {
    let batches = make_typical_batches(3, 8, false);

    assert_eq!(
        batches.as_slice(),
        vec![vec![0, 1, 2], vec![3, 4, 5]].as_slice()
    );
}

#[test]
fn test_remainder_finish() {
    let batches = make_typical_batches(3, 8, true);

    assert_eq!(
        batches.as_slice(),
        vec![vec![0, 1, 2], vec![3, 4, 5], vec![6, 7]].as_slice()
    );
}

#[test]
fn test_one_batch_finish() {
    let batches = make_typical_batches(5, 4, true);

    assert_eq!(
        batches.as_slice(),
        vec![vec![0, 1, 2, 3]].as_slice()
    );
}

#[test]
fn test_empty_batch_finish() {
    let batches = make_typical_batches(5, 0, true);

    assert!(batches.is_empty());
}
