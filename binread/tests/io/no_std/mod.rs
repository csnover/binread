mod cursor;

use binread::io::{Cursor, Error, ErrorKind, Read, Result};

#[derive(Debug)]
struct MalfunctioningEddie<'data> {
    error: Option<Error>,
    data: Cursor<&'data [u8]>,
}

impl <'data> MalfunctioningEddie<'data> {
    fn new(data: &'data [u8]) -> Self {
        Self {
            error: None,
            data: Cursor::new(data),
        }
    }

    // Pleased to meet you!
    // > Actually, we’ve met once before.
    fn trigger_fatal_error(&mut self) {
        // WHAT?!
        self.error = Some(Error::new(ErrorKind::BrokenPipe, ""));
    }

    // > You are being released.
    // Me? What a surprise!
    fn trigger_non_fatal_error(&mut self) {
        self.error = Some(Error::new(ErrorKind::Interrupted, ""));
        // Look! I barely exploded at all!
    }
}

impl Read for MalfunctioningEddie<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if let Some(error) = self.error.take() {
            Err(error)
        } else {
            self.data.read(buf)
        }
    }
}

#[test]
// Since binread re-exports std types when the std feature is turned on, make
// sure we are not accidentally testing the std implementation
fn sanity_check() {
    use core::any::TypeId;
    assert!(TypeId::of::<Cursor<[u8; 0]>>() != TypeId::of::<std::io::Cursor<[u8; 0]>>());
    assert!(TypeId::of::<Error>() != TypeId::of::<std::io::Error>());
    assert!(TypeId::of::<ErrorKind>() != TypeId::of::<std::io::ErrorKind>());
    assert!(TypeId::of::<dyn Read>() != TypeId::of::<dyn std::io::Read>());
    assert!(TypeId::of::<Result<()>>() != TypeId::of::<std::io::Result<()>>());
}

#[test]
fn bytes() {
    let mut cursor = MalfunctioningEddie::new(b"\0\x01\x02\x03\x04\x05");
    {
        let mut bytes = cursor.by_ref().bytes();
        assert!(matches!(bytes.next(), Some(Ok(0))));
        assert!(matches!(bytes.next(), Some(Ok(1))));
    }

    // Interrupted error should cause a retry
    cursor.trigger_non_fatal_error();
    {
        let mut bytes = cursor.by_ref().bytes();
        assert!(matches!(bytes.next(), Some(Ok(2))));
    }

    // Reads through Bytes should have advanced the underlying stream
    let mut raw_read_data = [0u8; 2];
    assert_eq!(cursor.read(&mut raw_read_data).unwrap(), 2);
    assert_eq!(raw_read_data, [3, 4]);

    // Errors other than Interrupted should be returned
    cursor.trigger_fatal_error();
    let mut bytes = cursor.bytes();
    assert_eq!(bytes.next().unwrap().unwrap_err().kind(), ErrorKind::BrokenPipe);
}

#[test]
fn read_exact() {
    let mut cursor = MalfunctioningEddie::new(b"\0\x01\x02\x03\x04\x05");

    let mut raw_read_data = [0u8; 2];
    cursor.read_exact(&mut raw_read_data).unwrap();
    assert_eq!(raw_read_data, [0, 1]);

    // Interrupted error should cause a retry
    cursor.trigger_non_fatal_error();
    cursor.read_exact(&mut raw_read_data).unwrap();
    assert_eq!(raw_read_data, [2, 3]);

    // Errors other than Interrupted should be returned
    cursor.trigger_fatal_error();
    assert_eq!(cursor.read_exact(&mut raw_read_data).unwrap_err().kind(), ErrorKind::BrokenPipe);

    // Read through a mutable reference should work as if it were directly on
    // the cursor
    cursor.by_ref().read_exact(&mut raw_read_data).unwrap();
    assert_eq!(raw_read_data, [4, 5]);

    // EOF reads should not succeed
    assert_eq!(cursor.read_exact(&mut raw_read_data).unwrap_err().kind(), ErrorKind::UnexpectedEof);
}

#[test]
fn interrupt_once() {
    struct InterruptReader(bool);

    impl Read for InterruptReader {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            if self.0 {
                self.0 = false;
                Err(Error::new(ErrorKind::Interrupted, ()))
            } else {
                buf.fill(0);
                Ok(buf.len())
            }
        }
    }

    let mut x = InterruptReader(true);
    let mut out = [1, 2, 3, 4];
    x.read_exact(&mut out).unwrap();

    assert_eq!(out, [0, 0, 0, 0]);

    let mut x = InterruptReader(true).bytes();
    assert_eq!(x.next().unwrap().unwrap(), 0);
    assert_eq!(x.next().unwrap().unwrap(), 0);
    assert_eq!(x.next().unwrap().unwrap(), 0);
    assert_eq!(x.next().unwrap().unwrap(), 0);
}

#[test]
fn return_error() {
    struct ReturnError(Option<Error>);

    impl Read for ReturnError {
        fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
            Err(self.0.take().unwrap())
        }
    }

    let mut x = ReturnError(Some(Error::new(ErrorKind::ConnectionRefused, ())));
    let mut out = [0, 1, 2, 3];

    assert_eq!(x.read_exact(&mut out).unwrap_err().kind(), ErrorKind::ConnectionRefused);

    let mut x = ReturnError(Some(Error::new(ErrorKind::ConnectionRefused, ()))).bytes();
    assert_eq!(x.next().unwrap().unwrap_err().kind(), ErrorKind::ConnectionRefused);
}
