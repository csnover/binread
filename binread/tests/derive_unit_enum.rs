use binread::{BinRead, io::Cursor};

#[test]
fn unit_enum_magic() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        #[br(magic(0u16))]
        Zero,
        #[allow(dead_code)]
        One,
        #[br(magic(2u16))]
        Two,
    }

    assert_eq!(Test::read(&mut Cursor::new(b"\0\0")).unwrap(), Test::Zero);
    let error = Test::read(&mut Cursor::new(b"\0\x01")).expect_err("accepted bad data");
    assert!(matches!(error, binread::Error::NoVariantMatch { .. }));
    assert_eq!(Test::read(&mut Cursor::new(b"\0\x02")).unwrap(), Test::Two);
}

#[test]
fn unit_enum_repr() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(big, repr(i16))]
    enum Test {
        Neg1 = -1,
        Zero,
        Two = 2,
    }

    assert_eq!(Test::read(&mut Cursor::new(b"\xff\xff")).unwrap(), Test::Neg1);
    let error = Test::read(&mut Cursor::new(b"\0\x01")).expect_err("accepted bad data");
    assert!(matches!(error, binread::Error::NoVariantMatch { .. }));
    assert_eq!(Test::read(&mut Cursor::new(b"\0\x02")).unwrap(), Test::Two);
}