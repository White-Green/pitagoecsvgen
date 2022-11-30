use std::cmp::Ordering;
use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NaturalOrderedStrItem<'a> {
    String(&'a str),
    Number(u128, &'a str),
}

impl<'a> NaturalOrderedStrItem<'a> {
    fn as_str(&self) -> &str {
        match self {
            NaturalOrderedStrItem::String(s) => s,
            NaturalOrderedStrItem::Number(_, s) => s,
        }
    }

    fn get_number(&self) -> Option<u128> {
        if let &NaturalOrderedStrItem::Number(num, _) = self {
            Some(num)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaturalOrderedStr<'a>(Box<[NaturalOrderedStrItem<'a>]>);

impl<'a> NaturalOrderedStr<'a> {
    pub fn new(value: &'a str) -> NaturalOrderedStr<'a> {
        let result = value.char_indices()
            .group_by(|(_, c)| c.is_ascii_digit())
            .into_iter()
            .map(|(is_ascii_digit, iter)| {
                let mut iter = iter;
                let begin = iter.next().unwrap();
                let end = iter.last().unwrap_or(begin);

                let string = &value[begin.0..end.0 + end.1.len_utf8()];
                if is_ascii_digit {
                    NaturalOrderedStrItem::Number(string.parse().unwrap(), string)
                } else {
                    NaturalOrderedStrItem::String(string)
                }
            }).collect::<Box<[_]>>();
        NaturalOrderedStr(result)
    }
}

impl<'a> PartialOrd for NaturalOrderedStr<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for NaturalOrderedStr<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut lhs = self.0.iter();
        let mut rhs = other.0.iter();
        let mut lhs_buffer = "".chars().peekable();
        let mut rhs_buffer = "".chars().peekable();
        let mut lhs_num_buffer = None::<u128>;
        let mut rhs_num_buffer = None::<u128>;
        loop {
            if let (Some(l), Some(r)) = (lhs_num_buffer, rhs_num_buffer) {
                if let ord @ (Ordering::Less | Ordering::Greater) = l.cmp(&r) {
                    return ord;
                }
                lhs_buffer = "".chars().peekable();
                rhs_buffer = "".chars().peekable();
                lhs_num_buffer = None;
                rhs_num_buffer = None;
            }
            match (lhs_buffer.peek(), rhs_buffer.peek()) {
                (Some(l), Some(r)) => {
                    if let ord @ (Ordering::Less | Ordering::Greater) = l.cmp(r) {
                        return ord;
                    }
                    lhs_buffer.next();
                    rhs_buffer.next();
                }
                (Some(_), None) => {
                    let Some(next_rhs_buffer) = rhs.next() else { return Ordering::Greater; };
                    rhs_buffer = next_rhs_buffer.as_str().chars().peekable();
                    rhs_num_buffer = next_rhs_buffer.get_number();
                }
                (None, Some(_)) => {
                    let Some(next_lhs_buffer) = lhs.next() else { return Ordering::Less; };
                    lhs_buffer = next_lhs_buffer.as_str().chars().peekable();
                    lhs_num_buffer = next_lhs_buffer.get_number();
                }
                (None, None) => {
                    match (lhs.next(), rhs.next()) {
                        (Some(next_lhs_buffer), Some(next_rhs_buffer)) => {
                            rhs_buffer = next_rhs_buffer.as_str().chars().peekable();
                            rhs_num_buffer = next_rhs_buffer.get_number();
                            lhs_buffer = next_lhs_buffer.as_str().chars().peekable();
                            lhs_num_buffer = next_lhs_buffer.get_number();
                        }
                        (Some(_), None) => return Ordering::Greater,
                        (None, Some(_)) => return Ordering::Less,
                        (None, None) => return Ordering::Equal,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_ordered_str_new() {
        assert_eq!(NaturalOrderedStr::new(""), NaturalOrderedStr(vec![].into_boxed_slice()));
        assert_eq!(NaturalOrderedStr::new("abc"), NaturalOrderedStr(vec![NaturalOrderedStrItem::String("abc")].into_boxed_slice()));
        assert_eq!(NaturalOrderedStr::new("あいう"), NaturalOrderedStr(vec![NaturalOrderedStrItem::String("あいう")].into_boxed_slice()));
        assert_eq!(NaturalOrderedStr::new("123abc"), NaturalOrderedStr(vec![NaturalOrderedStrItem::Number(123, "123"), NaturalOrderedStrItem::String("abc")].into_boxed_slice()));
        assert_eq!(NaturalOrderedStr::new("abc123"), NaturalOrderedStr(vec![NaturalOrderedStrItem::String("abc"), NaturalOrderedStrItem::Number(123, "123")].into_boxed_slice()));
        assert_eq!(NaturalOrderedStr::new("a1b2c3"),
                   NaturalOrderedStr(vec![NaturalOrderedStrItem::String("a"), NaturalOrderedStrItem::Number(1, "1"), NaturalOrderedStrItem::String("b"), NaturalOrderedStrItem::Number(2, "2"), NaturalOrderedStrItem::String("c"), NaturalOrderedStrItem::Number(3, "3")].into_boxed_slice()));
        assert_eq!(NaturalOrderedStr::new("123あいう"), NaturalOrderedStr(vec![NaturalOrderedStrItem::Number(123, "123"), NaturalOrderedStrItem::String("あいう")].into_boxed_slice()));
    }

    #[test]
    fn test_natural_ordered_str_ordering() {
        assert_eq!("".cmp(""), Ordering::Equal);
        assert_eq!("a".cmp(""), Ordering::Greater);
        assert_eq!("".cmp("a"), Ordering::Less);
        assert_eq!(NaturalOrderedStr::new("").cmp(&NaturalOrderedStr::new("")), Ordering::Equal);
        assert_eq!(NaturalOrderedStr::new("a").cmp(&NaturalOrderedStr::new("")), Ordering::Greater);
        assert_eq!(NaturalOrderedStr::new("").cmp(&NaturalOrderedStr::new("a")), Ordering::Less);
        assert_eq!(NaturalOrderedStr::new("abc").cmp(&NaturalOrderedStr::new("abc")), Ordering::Equal);
        assert_eq!(NaturalOrderedStr::new("abc").cmp(&NaturalOrderedStr::new("ab")), Ordering::Greater);
        assert_eq!(NaturalOrderedStr::new("ab").cmp(&NaturalOrderedStr::new("abc")), Ordering::Less);
        assert_eq!(NaturalOrderedStr::new("a1b2c3").cmp(&NaturalOrderedStr::new("a1b2c3")), Ordering::Equal);
        assert_eq!(NaturalOrderedStr::new("a1b2c3").cmp(&NaturalOrderedStr::new("a1b2c")), Ordering::Greater);
        assert_eq!(NaturalOrderedStr::new("a1b2c").cmp(&NaturalOrderedStr::new("a1b2c3")), Ordering::Less);
        assert_eq!("10_abc".cmp("10_abc"), Ordering::Equal);
        assert_eq!("10_abc".cmp("1_abc"), Ordering::Less);
        assert_eq!("1_abc".cmp("10_abc"), Ordering::Greater);
        assert_eq!("abc10_".cmp("abc10_"), Ordering::Equal);
        assert_eq!("abc10_".cmp("abc1_"), Ordering::Less);
        assert_eq!("abc1_".cmp("abc10_"), Ordering::Greater);
        assert_eq!(NaturalOrderedStr::new("10_abc").cmp(&NaturalOrderedStr::new("10_abc")), Ordering::Equal);
        assert_eq!(NaturalOrderedStr::new("10_abc").cmp(&NaturalOrderedStr::new("1_abc")), Ordering::Greater);
        assert_eq!(NaturalOrderedStr::new("1_abc").cmp(&NaturalOrderedStr::new("10_abc")), Ordering::Less);
        assert_eq!(NaturalOrderedStr::new("abc10_").cmp(&NaturalOrderedStr::new("abc10_")), Ordering::Equal);
        assert_eq!(NaturalOrderedStr::new("abc10_").cmp(&NaturalOrderedStr::new("abc1_")), Ordering::Greater);
        assert_eq!(NaturalOrderedStr::new("abc1_").cmp(&NaturalOrderedStr::new("abc10_")), Ordering::Less);
    }
}
