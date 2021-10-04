#[cfg(test)]
use crate::parser::*;
use serde_json::json;

fn parser() -> Parser {
    Parser::new("tags".into())
}

#[test]
fn test_parse_term() {
    // Unquoted terms
    assert_eq!(
        parser().parse_term("foo"),
        Ok((json!({"term": {"tags": "foo"}}), ""))
    );

    // Quoted terms
    assert_eq!(
        parser().parse_term("\"(foo)\""),
        Ok((json!({"term": {"tags": "(foo)"}}), ""))
    );

    // Errors
    assert!(parser().parse_term(")").is_err())
}

#[test]
fn test_parse_group() {
    // No additional nesting
    assert_eq!(
        parser().parse_group("foo"),
        Ok((json!({"term": {"tags": "foo"}}), ""))
    );
    assert_eq!(
        parser().parse_group("(foo)"),
        Ok((json!({"term": {"tags": "foo"}}), ""))
    );
    assert_eq!(
        parser().parse_group("((foo))"),
        Ok((json!({"term": {"tags": "foo"}}), ""))
    );

    // Terms with subexpressions
    assert_eq!(
        parser().parse_group("(foo (bar))"),
        Ok((json!({"term": {"tags": "foo (bar)"}}), ""))
    );
}
