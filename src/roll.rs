#[derive(Debug, PartialEq, Eq)]
enum RollSuff {
    None,
    Advantage,
    Disadvantage,
    Keep(u32),
}

impl RollSuff {
    fn format(&self) -> String {
        match self {
            Self::None => String::new(),
            Self::Advantage => "a".to_string(),
            Self::Disadvantage => "d".to_string(),
            Self::Keep(n) => format!("k{n}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RollOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl RollOp {
    fn from(c: char) -> Option<RollOp> {
        match c {
            '+' => Some(RollOp::Add),
            '-' => Some(RollOp::Sub),
            '*' | 'x' => Some(RollOp::Mul),
            '/' => Some(RollOp::Div),
            _ => None,
        }
    }
    fn format(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
        }
    }
}

#[derive(Debug, PartialEq)]
struct RollMod {
    op: RollOp,
    amount: f64,
}

impl RollMod {
    fn apply(&self, to: f64) -> f64 {
        match self.op {
            RollOp::Add => to + self.amount,
            RollOp::Sub => to - self.amount,
            RollOp::Mul => to * self.amount,
            RollOp::Div => to / self.amount,
        }
    }

    fn format(&self) -> String {
        format!("{} {}", self.op.format(), self.amount)
    }
}

#[derive(Debug, PartialEq)]
pub struct Roll {
    quantity: u32,
    size: u32,
    suff: RollSuff,
    mods: Vec<RollMod>,
}

impl Roll {
    fn resolve(self) -> RollOutcome {
        let mut results = Vec::new();
        for _ in 0..self.quantity.max(1) {
            results.push(rand::random_range(1..=self.size));
        }

        let rolls_total = match self.suff {
            RollSuff::None => results.iter().copied().sum(),
            RollSuff::Advantage => results.iter().copied().max().unwrap_or(0),
            RollSuff::Disadvantage => {
                results.iter().copied().min().unwrap_or(0)
            }
            RollSuff::Keep(n) => {
                let mut sorted = results.clone();
                sorted.sort();
                sorted.reverse();
                let n = (n as usize).min(sorted.len());
                (&sorted[0..n]).iter().copied().sum()
            }
        };

        let mut value = rolls_total as f64;
        for modifier in &self.mods {
            value = modifier.apply(value);
        }

        RollOutcome {
            roll: self,
            results,
            value,
        }
    }

    fn format(&self) -> String {
        let mods = if self.mods.is_empty() {
            String::new()
        } else {
            let mods = self
                .mods
                .iter()
                .map(|m| m.format())
                .collect::<Vec<String>>()
                .join(" ");
            format!(" {mods}")
        };

        format!(
            "{}d{}{}{}",
            if self.quantity != 1 {
                self.quantity.to_string()
            } else {
                String::new()
            },
            self.size,
            self.suff.format(),
            mods
        )
    }
}

struct RollOutcome {
    roll: Roll,
    results: Vec<u32>,
    value: f64,
}

impl RollOutcome {
    fn into_roll(self) -> Roll {
        self.roll
    }

    fn format_results(&self) -> String {
        self.results
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn format_value(&self) -> String {
        if self.value.fract() == 0.0 {
            format!("{}", self.value)
        } else {
            format!("{:.2}", self.value)
        }
    }
}

fn take_leading_int(text: &[char]) -> Option<(&[char], u32)> {
    let mut started = false;
    let mut val = 0;
    for (i, c) in text.iter().enumerate() {
        match c {
            _ if c.is_whitespace() && !started => (),
            v if v.is_digit(10) => {
                val *= 10;
                val += v.to_digit(10).unwrap();
                started = true;
            }
            _ if started => return Some((&text[i..], val)),
            _ => return None,
        }
    }

    if started { Some((&[], val)) } else { None }
}

fn next_char(text: &[char]) -> Option<(&[char], char)> {
    let mut stripped = text;
    while !stripped.is_empty() && stripped[0].is_whitespace() {
        stripped = &stripped[1..];
    }

    if !stripped.is_empty() {
        Some((&stripped[1..], stripped[0]))
    } else {
        None
    }
}

fn take_leading_number(text: &[char]) -> Option<(&[char], f64)> {
    let mut num = String::new();
    let mut text = text;
    while let Some((rest, c)) = next_char(text) {
        match c {
            '.' => {
                if !num.contains('.') {
                    num.push('.');
                } else {
                    return num.parse().ok().map(|v| (text, v));
                }
            }
            _ if c.is_digit(10) => num.push(c),
            _ => return num.parse().ok().map(|v| (text, v)),
        }

        text = rest;
    }
    num.parse().ok().map(|v| (text, v))
}

fn expect(expected: char, text: &[char]) -> Option<&[char]> {
    let (rest, actual) = next_char(text)?;
    if actual == expected { Some(rest) } else { None }
}

fn trim_whitespace(mut text: &[char]) -> &[char] {
    while !text.is_empty() && text[0].is_whitespace() {
        text = &text[1..];
    }
    text
}

fn parse_roll_suff_mods(
    text: &[char],
) -> Option<(&[char], RollSuff, Vec<RollMod>)> {
    let mut suff = RollSuff::None;
    let mut mods = Vec::new();

    let (mut text, c) = next_char(text)?;
    match c {
        'a' => suff = RollSuff::Advantage,
        'd' => suff = RollSuff::Disadvantage,
        'k' => match take_leading_int(text) {
            Some((rest, num)) => {
                text = rest;
                suff = RollSuff::Keep(num);
            }
            None => suff = RollSuff::Keep(1),
        },
        c if RollOp::from(c).is_some() => {
            let op = RollOp::from(c).unwrap();
            let val;
            (text, val) = take_leading_number(text)?;
            mods.push(RollMod { op, amount: val });
        }
        _ => return None,
    }

    match parse_roll_suff_mods(text) {
        Some((text, o_suff, more_mods)) => {
            mods.extend(more_mods);
            let suff = if o_suff == RollSuff::None {
                suff
            } else {
                o_suff
            };
            Some((text, suff, mods))
        }
        None => Some((text, suff, mods)),
    }
}

fn parse_one_roll(text: &[char]) -> Option<(Roll, &[char])> {
    let mut roll = Roll {
        quantity: 0,
        size: 0,
        suff: RollSuff::None,
        mods: Vec::new(),
    };

    let (text, quantity) =
        if let Some((text, quantity)) = take_leading_int(text) {
            (expect('d', text)?, quantity)
        } else {
            (expect('d', trim_whitespace(text))?, 1)
        };
    roll.quantity = quantity;
    let (mut text, size) = take_leading_int(text)?;
    roll.size = size;
    if let Some((rest, suff, mods)) = parse_roll_suff_mods(text) {
        text = rest;
        roll.suff = suff;
        roll.mods = mods;
    }

    Some((roll, text))
}

fn parse_roll(text: &str) -> Option<Roll> {
    let roll =
        parse_one_roll(text.chars().collect::<Vec<char>>().as_slice())?.0;
    Some(roll)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_add_mod() {
        let modifier = RollMod {
            op: RollOp::Add,
            amount: 3.0,
        };
        assert_eq!(modifier.format(), "+ 3");
    }

    #[test]
    fn test_format_sub_mod() {
        let modifier = RollMod {
            op: RollOp::Sub,
            amount: 3.2,
        };
        assert_eq!(modifier.format(), "- 3.2");
    }

    #[test]
    fn test_format_mul_mod() {
        let modifier = RollMod {
            op: RollOp::Mul,
            amount: 123.0,
        };
        assert_eq!(modifier.format(), "* 123");
    }

    #[test]
    fn test_format_div_mod() {
        let modifier = RollMod {
            op: RollOp::Div,
            amount: 0.125,
        };
        assert_eq!(modifier.format(), "/ 0.125");
    }

    #[test]
    fn test_format_roll_suffs() {
        assert_eq!(RollSuff::None.format(), "");
        assert_eq!(RollSuff::Advantage.format(), "a");
        assert_eq!(RollSuff::Disadvantage.format(), "d");
        assert_eq!(RollSuff::Keep(3).format(), "k3");
    }

    #[test]
    fn test_format_roll() {
        let roll = Roll {
            quantity: 4,
            size: 6,
            suff: RollSuff::Keep(3),
            mods: vec![
                RollMod {
                    op: RollOp::Add,
                    amount: 10.0,
                },
                RollMod {
                    op: RollOp::Mul,
                    amount: 10.1,
                },
            ],
        };
        assert_eq!(roll.format(), "4d6k3 + 10 * 10.1");
    }

    #[test]
    fn test_parse_roll() {
        let roll = Roll {
            quantity: 4,
            size: 6,
            suff: RollSuff::Keep(3),
            mods: vec![
                RollMod {
                    op: RollOp::Add,
                    amount: 10.0,
                },
                RollMod {
                    op: RollOp::Mul,
                    amount: 10.1,
                },
            ],
        };
        assert_eq!(parse_roll(roll.format().as_str()).unwrap(), roll);
    }

    #[test]
    fn test_parse_keep_suff() {
        let expected: (&[char], RollSuff, Vec<RollMod>) =
            (&[], RollSuff::Keep(8), Vec::new());
        assert_eq!(parse_roll_suff_mods(&['k', '8']).unwrap(), expected);
    }

    #[test]
    fn test_take_leading_int() {
        let expected: (&[char], u32) = (&['d', '6', 'k', '3'], 4);
        assert_eq!(
            take_leading_int(&['4', 'd', '6', 'k', '3']).unwrap(),
            expected
        );
    }

    #[test]
    fn test_expect() {
        let expected: &[char] = &['6', 'k', '3'];
        assert_eq!(expect('d', &[' ', 'd', '6', 'k', '3']).unwrap(), expected);
    }

    #[test]
    fn test_leading_d() {
        assert_eq!(parse_roll("d20").unwrap().format(), "d20");
    }

    #[test]
    fn test_trim_whitespace() {
        assert_eq!(trim_whitespace(&[' ', ' ', 'd']), &['d']);
    }
}
