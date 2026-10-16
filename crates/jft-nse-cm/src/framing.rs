//! Lossless framing for fixed-width, LF-delimited NSE records.

/// One fixed-width payload and its exact position in the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramedRecord<'a, const N: usize> {
    pub record_index: usize,
    pub byte_offset: usize,
    pub raw: &'a [u8; N],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FramingErrorReason {
    TrailingPartialRecord {
        expected_payload_bytes: usize,
        expected_delimiter_bytes: usize,
        actual_remaining_bytes: usize,
    },
    UnexpectedDelimiter {
        expected: u8,
        actual: u8,
        relative_offset: usize,
    },
}

/// A framing failure with all unconsumed source bytes preserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FramingError {
    pub record_index: usize,
    pub byte_offset: usize,
    pub unconsumed_raw: Vec<u8>,
    pub reason: FramingErrorReason,
}

/// A zero-copy iterator over exact-width payloads followed by one LF byte.
///
/// The iterator stops after the first error because resynchronizing after a
/// missing or displaced delimiter would require guessing about source bytes.
#[derive(Debug, Clone)]
pub struct FixedWidthLfRecords<'a, const N: usize> {
    source: &'a [u8],
    next_record_index: usize,
    next_byte_offset: usize,
    finished: bool,
}

impl<'a, const N: usize> FixedWidthLfRecords<'a, N> {
    #[must_use]
    pub fn new(source: &'a [u8]) -> Self {
        assert!(N > 0, "record payload width must be positive");
        Self {
            source,
            next_record_index: 0,
            next_byte_offset: 0,
            finished: false,
        }
    }

    fn fail(&mut self, reason: FramingErrorReason) -> FramingError {
        self.finished = true;
        FramingError {
            record_index: self.next_record_index,
            byte_offset: self.next_byte_offset,
            unconsumed_raw: self.source[self.next_byte_offset..].to_vec(),
            reason,
        }
    }
}

impl<'a, const N: usize> Iterator for FixedWidthLfRecords<'a, N> {
    type Item = Result<FramedRecord<'a, N>, FramingError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.next_byte_offset == self.source.len() {
            return None;
        }

        let remaining_len = self.source.len() - self.next_byte_offset;
        let framed_len = N + 1;
        if remaining_len < framed_len {
            return Some(Err(self.fail(FramingErrorReason::TrailingPartialRecord {
                expected_payload_bytes: N,
                expected_delimiter_bytes: 1,
                actual_remaining_bytes: remaining_len,
            })));
        }

        let delimiter_offset = self.next_byte_offset + N;
        let delimiter = self.source[delimiter_offset];
        if delimiter != b'\n' {
            return Some(Err(self.fail(FramingErrorReason::UnexpectedDelimiter {
                expected: b'\n',
                actual: delimiter,
                relative_offset: N,
            })));
        }

        let raw: &'a [u8; N] = self.source[self.next_byte_offset..delimiter_offset]
            .try_into()
            .expect("slice bounds establish the fixed record width");
        let framed = FramedRecord {
            record_index: self.next_record_index,
            byte_offset: self.next_byte_offset,
            raw,
        };

        self.next_record_index += 1;
        self.next_byte_offset += framed_len;
        Some(Ok(framed))
    }
}
