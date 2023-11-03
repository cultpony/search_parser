#![allow(dead_code)]
#![allow(unused_imports)]

use crate::tokenizer::*;
use chrono::{prelude::*, Duration};
use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Map, Value};

/// Represents a parsing error.
///
/// Different types of parsing errors may occur during the process
/// of execution. An invalid input error is fatal and entirely stops
/// parsing, while no match may simply allow a rule to fall through.
#[derive(Debug, Eq, PartialEq)]
pub enum ParseError {
    NoMatch,
    InvalidInput(String),
}

/// Represents the result of execution of a parser.
///
/// A parser should either return an error or return a value and
/// slice its input string to pass along to the next parser.
type ParseResult<'a> = Result<(Value, &'a str), ParseError>;

/// Represents a set of owned strings.
type FieldSet = Vec<String>;

/// Contains the resumable state of the parser.
///
/// The parser may occasionally need to call back into Erlang to update
/// the scheduler timeslice, or to retrieve the value of a custom field.
/// This parser design is intended to be serialized and passed back to
/// the runtime to allow this.
#[derive(Default)]
pub struct Parser {
    bool_fields: FieldSet,
    date_fields: FieldSet,
    float_fields: FieldSet,
    int_fields: FieldSet,
    ip_fields: FieldSet,
    literal_fields: FieldSet,
    ngram_fields: FieldSet,
    custom_fields: FieldSet,
    default_field: String,
}

/// The five types of ranges usable for fields which support a
/// range expression: `.lt:`, `.lte:`, `.gt:, `.gte:`, `:`
const RANGES: [SearchToken; 5] = [
    SearchToken::RangeLt,
    SearchToken::RangeLte,
    SearchToken::RangeGt,
    SearchToken::RangeGte,
    SearchToken::RangeEq,
];

/// The equality range field: `:`
const RANGE_EQ: [SearchToken; 1] = [SearchToken::RangeEq];

/// The components of an RFC3339 absolute date fragment
/// (naive, not including timezone):
/// `YYYY-MM-DDTHH:MM:SS` (or `YYYY-MM-DD HH:MM:SS`)
const ABS_DATE_FRAG: [SearchToken; 11] = [
    SearchToken::AbsoluteDate4Digit,
    SearchToken::AbsoluteDateHyphen,
    SearchToken::AbsoluteDate2Digit,
    SearchToken::AbsoluteDateHyphen,
    SearchToken::AbsoluteDate2Digit,
    SearchToken::AbsoluteDateTimeSep,
    SearchToken::AbsoluteDate2Digit,
    SearchToken::AbsoluteDateColon,
    SearchToken::AbsoluteDate2Digit,
    SearchToken::AbsoluteDateColon,
    SearchToken::AbsoluteDate2Digit,
];

/// The components of an RFC3339 offset component:
/// `Z` (zulu) or `+HH:MM` or `-HH:MM`
const ABS_DATE_OFFSET: [&[SearchToken]; 2] = [
    &[SearchToken::AbsoluteDateZulu],
    &[
        SearchToken::AbsoluteDateOffsetDirection,
        SearchToken::AbsoluteDate2Digit,
        SearchToken::AbsoluteDateColon,
        SearchToken::AbsoluteDate2Digit,
    ],
];

/// The components of a relative date:
/// `amount multiplier direction`
const REL_DATE: [SearchToken; 3] = [
    SearchToken::Integer,
    SearchToken::RelativeDateMultiplier,
    SearchToken::RelativeDateDirection,
];

impl Parser {
    pub fn new(default_field: String) -> Parser {
        Parser {
            default_field,
            ..Default::default()
        }
    }

    /// Parse the given search query into a JSON value, suitable for delivery
    /// to Elasticsearch.
    ///
    /// Returns either the encoded JSON value or a string error with an
    /// explanation for why parsing failed.
    pub fn parse(&mut self, input: &str) -> Result<Value, String> {
        match self.parse_lines(input) {
            Ok((v, _)) => Ok(v),
            Err(_) => Err("Parse error".to_string()),
        }
    }

    /// Parse any number of lines (treated as a disjunction between them).
    ///
    /// `lines = (top t_nl)*`
    pub fn parse_lines<'a>(&mut self, mut input: &'a str) -> ParseResult<'a> {
        let mut clauses: Vec<Value> = vec![];

        loop {
            if let Ok((left, rest)) = self.parse_top(input) {
                clauses.push(left);
                input = rest;
            } else if let Ok((rest, _)) = match_token(input, &SearchToken::Newline) {
                input = rest;
            } else {
                break;
            }
        }

        match_token(input, &SearchToken::Eof).map_err(|_| {
            ParseError::InvalidInput("Junk at end of expression".to_string())
        })?;

        match clauses.len() {
            0 => Ok((json!({"match_none": {}}), input)),
            1 => Ok((clauses.remove(0), input)),
            _ => Ok((json!({"bool": {"should": clauses}}), input))
        }
    }

    /// Top parse function. This is a convenience wrapper for the lowest priority
    /// rule in case it needs to be changed (in this case it is [Parser::parse_or]).
    ///
    /// `top = or`
    pub fn parse_top<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        self.parse_or(input)
    }

    /// Parse an OR expression.
    ///
    /// `or = and t_or top | and`
    pub fn parse_or<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        let (left, input) = self.parse_and(input)?;

        if let Ok((input, _)) = match_token(input, &SearchToken::Or) {
            let (right, input) = self.parse_top(input)?;

            Ok((json!({"bool": {"should": [left, right]}}), input))
        } else {
            Ok((left, input))
        }
    }

    /// Parse an AND expression.
    ///
    /// `and = boost t_and top | boost`
    fn parse_and<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        let (left, input) = self.parse_boost(input)?;

        if let Ok((input, _)) = match_token(input, &SearchToken::And) {
            let (right, input) = self.parse_top(input)?;

            Ok((json!({"bool": {"must": [left, right]}}), input))
        } else {
            Ok((left, input))
        }
    }

    /// Parse a query boosting expression.
    ///
    /// `boost = not t_boost t_float | not`
    fn parse_boost<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        let (child, input) = self.parse_not(input)?;

        if let Ok((input, boost)) = match_tokens(input, &[SearchToken::Boost, SearchToken::Float]) {
            let boost_val = boost[1].parse::<f32>().unwrap_or(0.0);

            match boost_val >= 0.0 {
                true => Ok((
                    json!({"function_score": {"query": child, "boost": boost[1]}}),
                    input,
                )),
                false => Err(ParseError::InvalidInput(
                    "Boost values must be non-negative".to_string(),
                )),
            }
        } else {
            Ok((child, input))
        }
    }

    /// Parse a NOT expression.
    ///
    /// `not = t_not top | group`
    fn parse_not<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        if let Ok((input, _)) = match_token(input, &SearchToken::Not) {
            let (child, input) = self.parse_top(input)?;

            Ok((json!({"bool": {"must_not": child}}), input))
        } else {
            self.parse_group(input)
        }
    }

    /// Parse a grouping expression.
    ///
    /// `group = t_lparen top t_rparen | term`
    pub fn parse_group<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        if let Ok((input, _)) = match_token(input, &SearchToken::Lparen) {
            let (child, input) = self.parse_top(input)?;

            if let Ok((input, _)) = match_token(input, &SearchToken::Rparen) {
                Ok((child, input))
            } else {
                Err(ParseError::InvalidInput(
                    "Imbalanced parentheses".to_string(),
                ))
            }
        } else {
            self.parse_term(input)
        }
    }

    /// Parse a term expression.
    ///
    /// ```txt
    ///
    /// term =
    ///   t_quot non_wildcardable t_quot |
    ///   t_quot literal_field range_eq t_qterm (t_fuzz t_float)? t_quot |
    ///   t_quot ngram_field range_eq t_qterm (t_fuzz t_float)? t_quot |
    ///   literal_field range_eq t_term (t_fuzz t_float)? |
    ///   ngram_field range_eq t_term (t_fuzz t_float)? |
    ///   non_wildcardable |
    ///   t_term (t_fuzz t_float)?;
    /// ```
    pub fn parse_term<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        if let Ok((input, term)) = match_tokens(
            input,
            &[
                SearchToken::Quote,
                SearchToken::QuotedTerm,
                SearchToken::Quote,
            ],
        ) {
            Ok((
                json!({"term": {self.default_field.clone(): term[1]}}),
                input,
            ))
        } else if let Ok((input, term)) = match_token(input, &SearchToken::Term) {
            Ok((json!({"term": {self.default_field.clone(): term}}), input))
        } else {
            Err(ParseError::NoMatch)
        }
    }

    /// Parse a boolean term expression.
    ///
    /// `bool = bool_field range_eq t_bool | ip`
    fn parse_bool<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        if let Ok((input, field)) =
            match_alternatives(input, &[&fields(self.bool_fields.iter()), &RANGE_EQ])
        {
            let (input, value) = match_token(input, &SearchToken::Boolean)
                .map_err(|_| ParseError::InvalidInput("Expected a boolean".to_string()))?;

            return Ok((json!({"term": {field[0]: value}}), input));
        }

        self.parse_ip(input)
    }

    /// Parse an IP term expression.
    ///
    /// `ip = ip_field range_eq t_ip | int`
    fn parse_ip<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        if let Ok((input, field)) =
            match_alternatives(input, &[&fields(self.ip_fields.iter()), &RANGE_EQ])
        {
            let (input, value) = match_token(input, &SearchToken::IpCidr)
                .map_err(|_| ParseError::InvalidInput("Expected an IP address".to_string()))?;

            return Ok((json!({"term": {field[0]: value}}), input));
        }

        self.parse_int(input)
    }

    /// Parse an integer term expression.
    ///
    /// `int = int_field a_range t_int (t_fuzz t_int)? | float`
    fn parse_int<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        if let Ok((input, field)) =
            match_alternatives(input, &[&fields(self.int_fields.iter()), &RANGES])
        {
            let (input, value) = match_token(input, &SearchToken::Integer)
                .map_err(|_| ParseError::InvalidInput("Expected an integer".to_string()))?;
            let value = value.parse::<i32>().unwrap_or(0);

            // Handle fuzzing expressions
            if let Ok((input, _)) = match_token(input, &SearchToken::Fuzz) {
                let (input, fuzz) = match_token(input, &SearchToken::Integer)
                    .map_err(|_| ParseError::InvalidInput("Expected an integer".to_string()))?;
                let fuzz = fuzz.parse::<i32>().unwrap_or(0).abs();

                return match field[1] {
                    ":" => Ok((
                        json!({"range": {field[0]: {"gte": value - fuzz, "lte": value + fuzz}}}),
                        input,
                    )),
                    _ => Err(ParseError::InvalidInput(
                        "Multiple ranges specified".to_string(),
                    )),
                };
            }

            // Everything else
            return Ok((term_range(field[0], field[1], value), input));
        }

        self.parse_float(input)
    }

    /// Parse a float term expression.
    ///
    /// `float = float_field a_range t_float (t_fuzz t_float)? | date`
    fn parse_float<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        if let Ok((input, field)) =
            match_alternatives(input, &[&fields(self.float_fields.iter()), &RANGES])
        {
            let (input, value) = match_token(input, &SearchToken::Float)
                .map_err(|_| ParseError::InvalidInput("Expected a float".to_string()))?;
            let value = value.parse::<f32>().unwrap_or(0.0);

            // Handle fuzzing expressions
            if let Ok((input, _)) = match_token(input, &SearchToken::Fuzz) {
                let (input, fuzz) = match_token(input, &SearchToken::Float)
                    .map_err(|_| ParseError::InvalidInput("Expected a float".to_string()))?;
                let fuzz = fuzz.parse::<f32>().unwrap_or(0.0).abs();

                return match field[1] {
                    ":" => Ok((
                        json!({"range": {field[0]: {"gte": value - fuzz, "lte": value + fuzz}}}),
                        input,
                    )),
                    _ => Err(ParseError::InvalidInput(
                        "Multiple ranges specified".to_string(),
                    )),
                };
            }

            // Everything else
            return Ok((term_range(field[0], field[1], value), input));
        }

        self.parse_date(input)
    }

    /// Parse a date expression.
    ///
    /// `date = date_field a_range (relative_date | absolute_date)`
    fn parse_date<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        if let Ok((input, field)) =
            match_alternatives(input, &[&fields(self.date_fields.iter()), &RANGES])
        {
            return match self.parse_relative_date(input, field[0], field[1]) {
                Ok(r) => Ok(r),
                Err(_) => self.parse_absolute_date(input, field[0], field[1]),
            };
        }

        Err(ParseError::NoMatch)
    }

    /// Parse a relative date expression.
    ///
    /// `relative_date = t_int t_multiplier t_direction`
    fn parse_relative_date<'a>(
        &mut self,
        input: &'a str,
        field: &'a str,
        range: &'a str,
    ) -> ParseResult<'a> {
        if let Ok((input, values)) = match_tokens(input, &REL_DATE) {
            let amount = values[0].parse::<i64>().unwrap_or(0);
            let multiplier = offset_multiplier(values[1]);
            let direction = offset_direction(values[2]);

            let now = Utc::now();
            let lower =
                (now + Duration::seconds(((amount * direction) + 1) * multiplier)).to_rfc3339();
            let upper = (now + Duration::seconds((amount * direction) * multiplier)).to_rfc3339();

            return Ok((date_range(field, range, lower, upper), input));
        }

        Err(ParseError::InvalidInput(
            "Expected a relative date".to_string(),
        ))
    }

    /// Parse an absolute date expression.
    ///
    /// `rfc3339_offset_frag = t_plusminus t_hour t_colon t_minute | t_zulu` \
    /// `rfc3339_time_frag = t_timesep t_hour t_colon (t_minute t_colon t_second?)?` \
    /// `rfc3339_date_frag = t_year (t_hyphen t_month (t_hyphen t_day rfc3339_time_frag?)?)?`
    ///
    /// `absolute_date = rfc3339_date_frag rfc3339_offset_frag?`
    fn parse_absolute_date<'a>(
        &mut self,
        input: &'a str,
        field: &'a str,
        range: &'a str,
    ) -> ParseResult<'a> {
        let (input, time) = match_at_most(input, &ABS_DATE_FRAG);
        let (input, tz) = match_alternatives(input, &ABS_DATE_OFFSET).unwrap_or((input, vec![]));

        if time.is_empty() {
            return Err(ParseError::InvalidInput(
                "Expected an absolute date".to_string(),
            ));
        }

        let (lower, upper) = date_fragment_to_naive(time)
            .ok_or_else(|| ParseError::InvalidInput("Expected an absolute date".to_string()))?;
        let offset = offset_fragment_to_fixed(tz);

        let lower = DateTime::<FixedOffset>::from_utc(lower, offset).to_rfc3339();
        let upper = DateTime::<FixedOffset>::from_utc(upper, offset).to_rfc3339();

        Ok((date_range(field, range, lower, upper), input))
    }
}

/// Converts a list of string fields into a form which can be passed to
/// [match_tokens] or [match_alternatives].
fn fields<'a, 'b: 'a, I>(fs: I) -> Vec<SearchToken<'a>>
where
    I: Iterator<Item = &'b String>,
{
    fs.map(|f| SearchToken::Field(f)).collect()
}

/// Creates an Elasticsearch JSON term range leaf from a field, range literal,
/// and serializable value.
fn term_range<T>(field: &str, range: &str, value: T) -> Value
where
    T: serde::ser::Serialize,
{
    match range {
        ".lt:" => json!({"range": {field: {"lt": value}}}),
        ".lte:" => json!({"range": {field: {"lte": value}}}),
        ".gt:" => json!({"range": {field: {"gt": value}}}),
        ".gte:" => json!({"range": {field: {"gte": value}}}),
        _ => json!({"term": {field: value}}),
    }
}

/// Creates an Elasticsearch JSON date range leaf from a field, range literal,
/// and lower and upper serializable values.
///
/// Note that there is some potentially counterintuitive range behavior due to
/// https://www.elastic.co/guide/en/elasticsearch/reference/current/query-dsl-range-query.html#ranges-on-dates.
fn date_range<T>(field: &str, range: &str, lower: T, upper: T) -> Value
where
    T: serde::ser::Serialize,
{
    match range {
        ".lt:" => json!({"range": {field: {"lt": lower}}}),
        ".lte:" => json!({"range": {field: {"lte": upper}}}),
        ".gt:" => json!({"range": {field: {"gt": upper}}}),
        ".gte:" => json!({"range": {field: {"gte": lower}}}),
        _ => json!({"range": {field: {"gte": lower, "lt": upper}}}),
    }
}

/// Converts a duration multiplier literal (like "minutes") into an
/// equivalent number of seconds.
///
/// TODO: convert this to a custom type
fn offset_multiplier(s: &str) -> i64 {
    match s {
        "minute" | "minutes" => 60,
        "hour" | "hours" => 60 * 60,
        "day" | "days" => 60 * 60 * 24,
        "week" | "weeks" => 60 * 60 * 24 * 7,
        "month" | "months" => 60 * 60 * 24 * 7 * 30,
        "year" | "years" => 60 * 60 * 24 * 365,
        _ => 1,
    }
}

/// Converts a duration multiplier literal (like "ago" or "from now") into
/// an equivalent sign (positive or negative).
///
/// TODO: convert this to a custom type
fn offset_direction(s: &str) -> i64 {
    match s {
        "from now" => 1,
        _ => -1,
    }
}

/// Used in `filter_map` to remove the separators from a sequence of date
/// fragment tokens.
fn date_token_filter((i, s): (usize, &str)) -> Option<usize> {
    // 2020-01-01T00:00:00
    //     ^  ^  ^  ^  ^
    //     1  3  5  7  9
    match i {
        1 | 3 | 5 | 7 | 9 => None,
        _ => Some(s.parse::<usize>().unwrap_or(0)),
    }
}

/// Convert a textual date fragment, like `2020-01-01`, into a naive date
/// range between lower and upper bounds.
fn date_fragment_to_naive(fragment: Vec<&str>) -> Option<(NaiveDateTime, NaiveDateTime)> {
    let mut ymdhms = [0, 1, 1, 0, 0, 0];
    let fragment: Vec<usize> = fragment
        .into_iter()
        .enumerate()
        .filter_map(date_token_filter)
        .collect();

    // Create date array
    for (i, d) in fragment.iter().enumerate().take(6) {
        ymdhms[i] = *d as u32;
    }

    // Find appropriate offset in seconds to get upper amount
    let offset = match fragment.len() {
        1 => Duration::days(365),
        2 => Duration::days(30),
        3 => Duration::days(7),
        4 => Duration::hours(24),
        5 => Duration::minutes(60),
        _ => Duration::seconds(1),
    };

    let lower = Utc
        .ymd_opt(ymdhms[0] as i32, ymdhms[1], ymdhms[2])
        .and_hms_opt(ymdhms[3], ymdhms[4], ymdhms[5])
        .single()?;
    let upper = lower + offset;

    Some((lower.naive_utc(), upper.naive_utc()))
}

/// Convert a textual offset like `Z` or `+05:00` to a timezone offset
fn offset_fragment_to_fixed(fragment: Vec<&str>) -> FixedOffset {
    // Offset length is 1 -> Zulu (UTC)
    if fragment.len() == 1 {
        return FixedOffset::east(0);
    }

    // Otherwise some offset exists
    let sign = if fragment[0] == "+" { 1 } else { -1 };
    let hours = fragment[1].parse::<i32>().unwrap_or(0);
    let minutes = fragment[3].parse::<i32>().unwrap_or(0);

    FixedOffset::east(sign * ((hours * 60) + minutes))
}
