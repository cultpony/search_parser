use std::{iter::{Peekable, Enumerate}, str::Chars};

use tracing::trace;

#[derive(Debug, Clone, Copy)]
pub struct TagLexem<'a>(FieldOrTagLexem<'a>);

impl<'a> TagLexem<'a> {
    pub const fn new(data: &'a str) -> Self {
        Self(FieldOrTagLexem::new(data))
    }

    pub fn find_end_str(self) -> &'a str {
        let e = self.0;
        let q = self.find_end();
        &e.0[..q]
    }

    pub fn find_end(self) -> usize {
        let inp = self.0.0;
        let eot = self.0.find_end();
        if let Some(field_char) = inp.chars().nth(eot) {
            if field_char != '.' {
                trace!("didn't find field dot, is tag");
                return eot
            }
            trace!("found field dot, not a tag");
            return 0
        }
        eot
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FieldLexem<'a>(FieldOrTagLexem<'a>);

impl<'a> FieldLexem<'a> {
    pub const fn new(data: &'a str) -> Self {
        Self(FieldOrTagLexem::new(data))
    }

    pub fn find_end_str(self) -> &'a str {
        let e = self.0.0;
        let q = self.find_end();
        &e[..q]
    }

    pub fn find_end(self) -> usize {
        let inp = self.0.0;
        let eot = self.0.find_end();
        if let Some(field_char) = inp.chars().nth(eot) {
            if field_char == '.' {
                trace!("found field dot, is field, not tag");
                return eot + 1
            }
            trace!("no field dot, not a field");
            return 0
        }
        0
    }
}

/// Matches on tokens the terminate a field or tag
/// and return the length of the lexem
#[derive(Debug, Clone, Copy)]
pub struct FieldOrTagLexem<'a>(&'a str);

const fn is_single_char_termination(c: char) -> bool {
    matches!(c, 
        '(' | ')' | '*' | '?' | ',' | '"' | '~' | '^'
    )
}

type CList<'q> = Peekable<Enumerate<Chars<'q>>>;

impl<'a> FieldOrTagLexem<'a> {
    pub const fn new(data: &'a str) -> Self {
        Self(data)
    }

    pub fn find_end_str(self) -> &'a str {
        let e = self.0;
        let q = self.find_end();
        &e[..q]
    }

    pub fn find_end(self) -> usize {
        let odata = self.0;
        let data_size = odata.chars().count();
        let data: &mut CList = &mut odata.chars().enumerate().peekable();
        let cret = |data_size| {
            trace!("found termination at {data_size}");
            let data = &odata[..data_size];
            data.trim().len()
        };
        while let Some((pos, chr)) = data.next() {
            trace!("checking if char {chr:?} at {pos} terminates");
            match chr {
                // on escape, advance by one
                '\\' if match_next_any(data, ['A', 'O', 'N', '&', '|']) => { data.next(); },
                '\\' if pos == 0 && match_next_any(data, ['!', '-']) => { data.next(); },
                '\\' if data.peek().map(|x| is_single_char_termination(x.1)).unwrap_or(false) => { data.next(); },
                c if is_single_char_termination(c) => return cret(pos),
                '.' => return cret(pos),
                '-' | '!' if pos == 0 => return cret(pos),
                'A' if match_n(data, ['N', 'D']) => return cret(pos),
                'O' if match_1(data, 'R') => return cret(pos),
                'N' if match_n(data, ['O', 'T'])=> return cret(pos),
                '&' if match_1(data, '&') => return cret(pos),
                '|' if match_1(data, '|') => return cret(pos),
                // TODO: add "" quoting
                _ => (),
            }
        }
        // all remaining characters matched
        cret(data_size)
    }
}

fn match_n<const N: usize>(d: &mut CList<'_>, m: [char; N]) -> bool {
    let mut c = 0;
    while let Some(peek) = d.peek() {
        if m[c] != peek.1 {
            return false;
        }
        c += 1;
        if c >= N {
            return true;
        }
        d.next();
    }
    true
}

fn match_next_any<const N: usize>(d: &mut CList<'_>, m: [char; N]) -> bool {
    let c = match d.peek() {
        Some(v) => *v,
        None => return false,
    };
    for i in m {
        if i == c.1 {
            d.next();
            return true;
        }
    }
    false
}

#[tracing::instrument]
fn match_1(d: &mut CList<'_>, m: char) -> bool {
    d.peek().map(|x| x.1) == Some(m)
}

#[cfg(test)]
mod test {
    use super::FieldOrTagLexem;

    #[test]
    #[tracing_test::traced_test]
    pub fn test_name_termination() {
        assert_eq!("rose", FieldOrTagLexem::new(r"rose     ").find_end_str());
        assert_eq!("rose", FieldOrTagLexem::new(r"rose (flower)").find_end_str());
        assert_eq!("rose", FieldOrTagLexem::new(r"rose)").find_end_str());
        assert_eq!(r"rose \(flower\)", FieldOrTagLexem::new(r"rose \(flower\)").find_end_str());
        assert_eq!(r"", FieldOrTagLexem::new(r"-_-").find_end_str());
        assert_eq!(r"\\-_-", FieldOrTagLexem::new(r"\\-_-").find_end_str());
        assert_eq!("hello\\friend", FieldOrTagLexem::new("hello\\friend").find_end_str())
    }
}