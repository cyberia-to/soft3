//! Small text codecs the harness needs: hex, the nox textual data form, and
//! the `.nox` fixture file format.

/// Lowercase hex of bytes.
pub fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Decode lowercase or uppercase hex; `None` on odd length or bad digit.
pub fn from_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let digit = |c: u8| -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    };
    s.as_bytes()
        .chunks(2)
        .map(|p| Some(digit(p[0])? << 4 | digit(p[1])?))
        .collect()
}

/// A nox data value in textual form: `42` or `[a b c]` (right-nested pairs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data {
    /// a field element literal (must be < p)
    Atom(u64),
    /// a pair
    Pair(Box<Data>, Box<Data>),
}

/// Parse the nox textual data form used by `nox -e` and the jet fixtures:
/// `[a b c]` right-nests to `[a [b c]]`; `_` separators allowed in numbers.
pub fn parse_data(input: &str) -> Result<Data, String> {
    let bytes = input.as_bytes();
    let mut pos = 0;
    let d = parse_expr(bytes, &mut pos, 0)?;
    skip_ws(bytes, &mut pos);
    if pos != bytes.len() {
        return Err(format!("trailing input at byte {pos}"));
    }
    Ok(d)
}

fn skip_ws(b: &[u8], pos: &mut usize) {
    while *pos < b.len() && b[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
}

fn parse_expr(b: &[u8], pos: &mut usize, depth: usize) -> Result<Data, String> {
    if depth > 4096 {
        return Err("expression too deeply nested".into());
    }
    skip_ws(b, pos);
    match b.get(*pos) {
        None => Err("unexpected end of input".into()),
        Some(b'[') => {
            *pos += 1;
            let mut elems = Vec::new();
            loop {
                skip_ws(b, pos);
                match b.get(*pos) {
                    Some(b']') => {
                        *pos += 1;
                        break;
                    }
                    None => return Err("expected ']'".into()),
                    _ => elems.push(parse_expr(b, pos, depth + 1)?),
                }
            }
            if elems.len() < 2 {
                return Err("a pair needs at least two elements".into());
            }
            let mut acc = elems.pop().expect("len >= 2");
            while let Some(head) = elems.pop() {
                acc = Data::Pair(Box::new(head), Box::new(acc));
            }
            Ok(acc)
        }
        Some(c) if c.is_ascii_digit() => {
            let mut v: u64 = 0;
            while let Some(&c) = b.get(*pos) {
                if c == b'_' {
                    *pos += 1;
                    continue;
                }
                if !c.is_ascii_digit() {
                    break;
                }
                v = v
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(u64::from(c - b'0')))
                    .ok_or("numeric literal exceeds u64")?;
                *pos += 1;
            }
            if v >= nebu::field::P {
                return Err(format!("atom {v} is not a canonical Goldilocks element"));
            }
            Ok(Data::Atom(v))
        }
        Some(c) => Err(format!("unexpected character {:?}", *c as char)),
    }
}

/// One `.nox` fixture: a formula reduced against an object under a budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    /// object (subject) the formula is reduced against
    pub object: Data,
    /// the formula
    pub formula: Data,
    /// focus budget
    pub budget: u64,
    /// expected outcome, when the fixture states one (a spec test vector)
    pub expect: Option<Expect>,
}

/// An outcome a fixture asserts: checked as an invariant, independent of
/// the snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expect {
    /// `result <data>` with optional `cost <n>`
    Result(Data, Option<u64>),
    /// `error <Kind>` — the nox ErrorKind name
    Error(String),
    /// `halt` — budget exhausted
    Halt,
}

/// Parse a fixture file:
///
/// ```text
/// # comment
/// object  [1 2]
/// budget  100
/// formula [5 [[0 2] [0 3]]]
/// result  3          # optional: asserted outcome
/// cost    3          # optional, with result
/// ```
///
/// `error <Kind>` or `halt` assert a failing outcome instead of `result`.
///
/// A file whose only content is a bare formula (the nox repo's `jets/*.nox`)
/// parses with object `0` and budget `1_000_000`, the nox CLI defaults.
pub fn parse_program(src: &str) -> Result<Program, String> {
    let mut object = None;
    let mut formula = None;
    let mut budget = None;
    let mut bare = String::new();
    let mut result = None;
    let mut cost = None;
    let mut expect = None;
    for line in src.lines() {
        let line = line.split(" #").next().unwrap_or("").trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("object") {
            object = Some(parse_data(rest)?);
        } else if let Some(rest) = line.strip_prefix("formula") {
            formula = Some(parse_data(rest)?);
        } else if let Some(rest) = line.strip_prefix("result") {
            result = Some(parse_data(rest)?);
        } else if let Some(rest) = line.strip_prefix("cost") {
            cost = Some(rest.trim().replace('_', "").parse::<u64>().map_err(|e| e.to_string())?);
        } else if let Some(rest) = line.strip_prefix("error") {
            expect = Some(Expect::Error(rest.trim().to_string()));
        } else if line == "halt" {
            expect = Some(Expect::Halt);
        } else if let Some(rest) = line.strip_prefix("budget") {
            budget = Some(rest.trim().replace('_', "").parse::<u64>().map_err(|e| e.to_string())?);
        } else {
            bare.push_str(line);
            bare.push(' ');
        }
    }
    let formula = match (formula, bare.trim().is_empty()) {
        (Some(f), true) => f,
        (None, false) => parse_data(&bare)?,
        (Some(_), false) => return Err("formula given twice".into()),
        (None, true) => return Err("no formula".into()),
    };
    if let Some(r) = result {
        if expect.is_some() {
            return Err("result and error/halt are exclusive".into());
        }
        expect = Some(Expect::Result(r, cost));
    } else if cost.is_some() {
        return Err("cost needs a result".into());
    }
    Ok(Program {
        object: object.unwrap_or(Data::Atom(0)),
        formula,
        budget: budget.unwrap_or(1_000_000),
        expect,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        assert_eq!(from_hex(&to_hex(&[0, 1, 0xab, 0xff])), Some(vec![0, 1, 0xab, 0xff]));
        assert_eq!(from_hex("abc"), None);
        assert_eq!(from_hex("zz"), None);
    }

    #[test]
    fn data_right_nests() {
        let d = parse_data("[1 2 3]").unwrap();
        let want = Data::Pair(
            Box::new(Data::Atom(1)),
            Box::new(Data::Pair(Box::new(Data::Atom(2)), Box::new(Data::Atom(3)))),
        );
        assert_eq!(d, want);
        assert!(parse_data("[1]").is_err());
        assert!(parse_data("18446744069414584321").is_err());
        assert!(parse_data("[1 2] 3").is_err());
    }

    #[test]
    fn program_forms() {
        let p = parse_program("# x\nobject [1 2]\nbudget 1_00\nformula [5 [[0 2] [0 3]]]\n").unwrap();
        assert_eq!(p.budget, 100);
        let bare = parse_program("[1000 [0 1]]\n").unwrap();
        assert_eq!(bare.object, Data::Atom(0));
        assert_eq!(bare.budget, 1_000_000);
        assert_eq!(bare.expect, None);
        let e = parse_program("formula [1 7]\nresult 7 # comment\ncost 1\n").unwrap();
        assert_eq!(e.expect, Some(Expect::Result(Data::Atom(7), Some(1))));
        assert!(parse_program("formula [1 7]\nresult 7\nhalt\n").is_err());
    }
}
