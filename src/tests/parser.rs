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

#[test]
fn test_parse_or() {
    assert_eq!(
        parser().parse_top("foo || bar OR baz\nfaz"),
        Ok((
            json!({"bool": {
                "should": [
                    {"term": {"tags": "foo"}},
                    {"bool": {
                        "should": [
                            {"term": {"tags": "bar"}},
                            {"bool": {
                                "should": [
                                    {"term": {"tags": "baz"}},
                                    {"term": {"tags": "faz"}}
                                ]
                            }}
                        ]
                    }}
                ]
            }}),
            ""
        ))
    );
}

#[test]
fn test_comments() {
    assert_eq!(
        parser().parse_top("# some comment\nfoo, bar"),
        Ok((
            json!({"bool": {
                "must": [
                    {"term": {"tags": "foo"}},
                    {"term": {"tags": "bar"}},
                ]
            }}),
            ""
        ))
    );

    assert_eq!(
        parser().parse_top("foo, bar # some comment"),
        Ok((
            json!({"bool": {
                "must": [
                    {"term": {"tags": "foo"}},
                    {"term": {"tags": "bar"}},
                ]
            }}),
            ""
        ))
    );
}
