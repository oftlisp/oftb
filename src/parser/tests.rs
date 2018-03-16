use pest::{Error as PestError, Position};

use parser::{parse_program, Rule};

#[test]
fn invalid_bytes_escape() {
    let src = r#"b"\x01\x23\u4567""#;
    let r = parse_program(src);
    assert_eq!(
        r,
        Err(PestError::ParsingError {
            positives: vec![Rule::hex_esc, Rule::predef_esc],
            negatives: vec![],
            pos: Position::from_start(src).skip(11).unwrap(),
        })
    );
}

#[test]
fn invalid_unicode_escape() {
    let src = r#" "ab\U00110000cd" "#;
    let r = parse_program(src);
    assert_eq!(
        r,
        Err(PestError::CustomErrorSpan {
            message: r#"Invalid Unicode Escape: \U00110000"#.to_string(),
            span: Position::from_start(src)
                .skip(4)
                .unwrap()
                .span(&Position::from_start(src).skip(14).unwrap()),
        })
    );
}
