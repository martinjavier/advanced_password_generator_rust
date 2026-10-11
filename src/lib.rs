//! Advanced Password Generator (adg)
//!
//! Argument parsing and password generation logic.

use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use rand::Rng;

pub const MIN_LEN: usize = 1;
pub const MAX_LEN: usize = 256;
pub const MIN_COUNT: usize = 1;
pub const MAX_COUNT: usize = 1000;

const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// Available character groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Upper,
    Lower,
    Digits,
    Symbols,
}

impl Group {
    pub const ALL: [Group; 4] = [Group::Upper, Group::Lower, Group::Digits, Group::Symbols];

    pub fn chars(self) -> &'static [u8] {
        match self {
            Group::Upper => UPPER,
            Group::Lower => LOWER,
            Group::Digits => DIGITS,
            Group::Symbols => SYMBOLS,
        }
    }

    fn from_char(c: char) -> Option<Group> {
        match c.to_ascii_uppercase() {
            'U' => Some(Group::Upper),
            'L' => Some(Group::Lower),
            'N' => Some(Group::Digits),
            'S' => Some(Group::Symbols),
            _ => None,
        }
    }
}

/// Configuration built from the arguments.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub length: usize,
    pub groups: Vec<Group>,
    pub count: usize,
}

/// Parse result: help, version, or a valid configuration.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Version,
    Generate(Config),
}

/// Parses the arguments (without the program name).
pub fn parse_args<I, S>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();

    if args.is_empty() {
        return Err("missing password length.".into());
    }
    if args.iter().any(|a| a == "-h" || a == "--help") {
        return Ok(Command::Help);
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        return Ok(Command::Version);
    }

    let mut length: Option<usize> = None;
    let mut count: Option<usize> = None;
    let mut groups: Vec<Group> = Vec::new();
    let mut all = false;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        if arg == "-c" || arg == "--count" {
            if count.is_some() {
                return Err("option -c can only be given once.".into());
            }
            let value = args
                .get(i + 1)
                .ok_or("option -c requires a number (e.g. -c 5).")?;
            count = Some(parse_count(value)?);
            i += 2;
            continue;
        }

        if arg.starts_with('-') && arg.len() > 1 {
            return Err(format!("unknown option: '{arg}'."));
        }

        if length.is_none() {
            length = Some(parse_length(arg)?);
        } else if arg.eq_ignore_ascii_case("all") {
            all = true;
        } else {
            for c in arg.chars() {
                let g = Group::from_char(c).ok_or_else(|| {
                    format!("unknown group '{c}' in '{arg}'. Use U, L, N, S or all.")
                })?;
                if !groups.contains(&g) {
                    groups.push(g);
                }
            }
        }
        i += 1;
    }

    let length = length.ok_or("missing password length.")?;

    // No groups (or 'all') means all four.
    if all || groups.is_empty() {
        groups = Group::ALL.to_vec();
    } else {
        // Canonical order, so "SU" and "US" produce the same configuration.
        groups.sort_by_key(|g| Group::ALL.iter().position(|x| x == g));
    }

    if length < groups.len() {
        return Err(format!(
            "length ({length}) is less than the number of selected groups ({}); \
             cannot guarantee one character from each group.",
            groups.len()
        ));
    }

    Ok(Command::Generate(Config {
        length,
        groups,
        count: count.unwrap_or(1),
    }))
}

fn parse_length(s: &str) -> Result<usize, String> {
    let n: usize = s
        .parse()
        .map_err(|_| format!("length must be a number between {MIN_LEN} and {MAX_LEN} (got: '{s}')."))?;
    if !(MIN_LEN..=MAX_LEN).contains(&n) {
        return Err(format!("length must be between {MIN_LEN} and {MAX_LEN} (got: {n})."));
    }
    Ok(n)
}

fn parse_count(s: &str) -> Result<usize, String> {
    let n: usize = s
        .parse()
        .map_err(|_| format!("-c must be a number between {MIN_COUNT} and {MAX_COUNT} (got: '{s}')."))?;
    if !(MIN_COUNT..=MAX_COUNT).contains(&n) {
        return Err(format!("-c must be between {MIN_COUNT} and {MAX_COUNT} (got: {n})."));
    }
    Ok(n)
}

/// Generates a password using a cryptographically secure RNG.
///
/// Guarantees at least one character from each group and shuffles the result
/// so those characters are not always at the start.
pub fn generate(length: usize, groups: &[Group]) -> String {
    generate_with(&mut OsRng, length, groups)
}

pub fn generate_with<R: Rng + ?Sized>(rng: &mut R, length: usize, groups: &[Group]) -> String {
    assert!(!groups.is_empty() && length >= groups.len());

    let pool: Vec<u8> = groups.iter().flat_map(|g| g.chars().iter().copied()).collect();

    let mut out: Vec<u8> = Vec::with_capacity(length);
    for g in groups {
        let set = g.chars();
        out.push(set[rng.gen_range(0..set.len())]);
    }
    while out.len() < length {
        out.push(pool[rng.gen_range(0..pool.len())]);
    }
    out.shuffle(rng);

    // All characters are ASCII, so the conversion is safe.
    String::from_utf8(out).expect("valid ASCII")
}

pub fn help_text() -> String {
    format!(
        "\
Advanced Password Generator (adg) v{version}

USAGE:
    adg <length> [groups] [-c <n>]

ARGUMENTS:
    <length>      Password length ({MIN_LEN}-{MAX_LEN})
    [groups]      Group combination (default: all)
                    U    Uppercase   (A-Z)
                    L    Lowercase   (a-z)
                    N    Digits      (0-9)
                    S    Symbols     (!\"#$%&'()*+,-./:;<=>?@[\\]^_`{{|}}~)
                    all  All four groups (ULNS)
                  Case-insensitive, and may be combined or separate:
                  'ULN' is the same as 'U L N' or 'uln'.

OPTIONS:
    -c, --count <n>   Generate n passwords ({MIN_COUNT}-{MAX_COUNT}), one per line
    -h, --help        Show this help
    -V, --version     Show version

At least one character from each selected group is guaranteed.

EXAMPLES:
    adg 128 all
    adg 16 ULNS
    adg 64 NS
    adg 32 S
    adg 20 -c 5
    adg 12 U L N -c 3",
        version = env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(args: &[&str]) -> Config {
        match parse_args(args).unwrap() {
            Command::Generate(c) => c,
            other => panic!("expected Generate, got {other:?}"),
        }
    }

    #[test]
    fn defaults_to_all_and_one() {
        let c = cfg(&["16"]);
        assert_eq!(c.length, 16);
        assert_eq!(c.groups, Group::ALL.to_vec());
        assert_eq!(c.count, 1);
    }

    #[test]
    fn all_is_case_insensitive() {
        assert_eq!(cfg(&["128", "ALL"]).groups, Group::ALL.to_vec());
        assert_eq!(cfg(&["128", "all"]).groups, Group::ALL.to_vec());
    }

    #[test]
    fn groups_combined_split_and_lowercase_are_equivalent() {
        let a = cfg(&["16", "ULNS"]).groups;
        let b = cfg(&["16", "U", "L", "N", "S"]).groups;
        let c = cfg(&["16", "snlu"]).groups;
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn repeated_groups_are_ignored() {
        assert_eq!(cfg(&["8", "NNS"]).groups, vec![Group::Digits, Group::Symbols]);
    }

    #[test]
    fn count_option_anywhere() {
        assert_eq!(cfg(&["20", "-c", "5"]).count, 5);
        assert_eq!(cfg(&["-c", "3", "20", "NS"]).count, 3);
        assert_eq!(cfg(&["20", "--count", "2"]).count, 2);
    }

    #[test]
    fn length_bounds() {
        assert!(parse_args(["0"]).is_err());
        assert!(parse_args(["257"]).is_err());
        assert!(parse_args(["abc"]).is_err());
        assert_eq!(cfg(&["1", "S"]).length, 1);
        assert_eq!(cfg(&["256"]).length, 256);
    }

    #[test]
    fn invalid_inputs() {
        assert!(parse_args(Vec::<&str>::new()).is_err());
        assert!(parse_args(["16", "X"]).is_err());
        assert!(parse_args(["16", "-c"]).is_err());
        assert!(parse_args(["16", "-c", "0"]).is_err());
        assert!(parse_args(["16", "-c", "1001"]).is_err());
        assert!(parse_args(["16", "-z"]).is_err());
        assert!(parse_args(["2", "all"]).is_err()); // 4 groups > length 2
    }

    #[test]
    fn help_and_version() {
        assert_eq!(parse_args(["--help"]).unwrap(), Command::Help);
        assert_eq!(parse_args(["16", "-h"]).unwrap(), Command::Help);
        assert_eq!(parse_args(["-V"]).unwrap(), Command::Version);
    }

    #[test]
    fn generated_password_respects_length_and_groups() {
        let combos: &[&[Group]] = &[
            &Group::ALL,
            &[Group::Digits, Group::Symbols],
            &[Group::Symbols],
            &[Group::Upper, Group::Lower],
        ];
        for groups in combos {
            for len in [groups.len(), groups.len() + 1, 16, 256] {
                for _ in 0..200 {
                    let p = generate(len, groups);
                    assert_eq!(p.len(), len);
                    // Every selected group appears at least once.
                    for g in *groups {
                        assert!(p.bytes().any(|b| g.chars().contains(&b)), "missing {g:?} in {p}");
                    }
                    // No characters outside the selected groups.
                    assert!(p.bytes().all(|b| groups.iter().any(|g| g.chars().contains(&b))));
                }
            }
        }
    }

    #[test]
    fn symbols_are_all_printable_ascii() {
        // 32 printable ASCII symbols (no space, letters or digits).
        assert_eq!(SYMBOLS.len(), 32);
        assert!(SYMBOLS.iter().all(|b| b.is_ascii_punctuation()));
    }
}
