use std::borrow::Cow;
use std::path::Path;
use lindera::tokenizer::{Token, Tokenizer};
use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use crate::natural_ordered_str::NaturalOrderedStr;

mod natural_ordered_str;

#[derive(Default)]
struct Entry {
    path: String,
    name: String,
    text: String,
    ruby: String,
    category: String,
}

impl Entry {
    fn into_array(self) -> [String; 5] {
        let Entry { path, name, text, ruby, category } = self;
        [path, name, text, ruby, category]
    }
}

#[wasm_bindgen]
pub fn natural_sort(value: JsValue) -> Box<[usize]> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let list: Box<[String]> = serde_wasm_bindgen::from_value(value).unwrap();
    let mut list = list.iter().enumerate().map(|(i, item)| (i, NaturalOrderedStr::new(item))).collect::<Box<[_]>>();
    list.sort_by(|(_, s1), (_, s2)| s1.cmp(s2));
    list.iter().map(|&(i, _)| i).collect()
}

#[wasm_bindgen]
pub fn process(value: JsValue, cat_pattern: &str) -> JsValue {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let list: Box<[Box<[String]>]> = serde_wasm_bindgen::from_value(value).unwrap();
    assert!(!list.is_empty());
    list.iter().for_each(|path| assert!(!path.is_empty()));
    let entries = process_path_list(&list, cat_pattern);
    serde_wasm_bindgen::to_value(&entries.into_iter().map(Entry::into_array).collect::<Box<[_]>>()).unwrap()
}

fn process_path_list(list: &[Box<[String]>], cat_pattern: &str) -> Vec<Entry> {
    let normalized_names = list.iter().map(|path| path.last().unwrap().nfkc().collect::<String>()).collect::<Box<[_]>>();
    let mut entries = list.iter().zip(&*normalized_names).map(|(path, normalized_name)| {
        let (_, dir) = path.split_last().unwrap();
        let name = Path::new(normalized_name.as_str()).file_stem().unwrap().to_str().unwrap();
        (
            dir,
            dir.iter().map(|item| NaturalOrderedStr::new(item)).collect::<Box<[_]>>(),
            name,
            NaturalOrderedStr::new(name),
            Entry {
                path: join_separated(path, '/'),
                ..Entry::default()
            }
        )
    }).collect::<Vec<_>>();
    entries.sort_by(|(_, natural_ordered_dir1, _, natural_ordered_name1, _), (_, natural_ordered_dir2, _, natural_ordered_name2, _)| natural_ordered_dir1.cmp(natural_ordered_dir2).then_with(|| natural_ordered_name1.cmp(natural_ordered_name2)));

    let numbers = Regex::new("\\d+").unwrap();
    let (common_prefix, common_suffix) = entries.iter().map(|(_, _, name, _, _)| numbers.replace_all(name, "\0")).map(|s| (s.clone(), s))
        .reduce(|(prefix, suffix), (b1, b2)| (str_common_prefix(prefix, &b1), str_common_suffix(suffix, &b2))).unwrap();
    let common_prefix = Regex::new(&format!("^{}", common_prefix.replace("\0", "\\d+"))).unwrap();
    let common_suffix = Regex::new(&format!("{}$", common_suffix.replace("\0", "\\d+"))).unwrap();
    let tokenizer = Tokenizer::new().unwrap();
    for (dir, _, name, _, entry) in entries.iter_mut() {
        entry.name = name.to_string();
        let text = common_prefix.replace(name, "");
        let text = common_suffix.replace(&text, "");
        entry.text = text.into_owned();
        let text = tokenizer.tokenize(name).unwrap().iter_mut().map(|Token { text, detail }| {
            if detail.len() > 7 {
                Cow::Owned(detail.swap_remove(7))
            } else {
                Cow::Borrowed(*text)
            }
        }).reduce(|a, b| a + b).unwrap();
        entry.ruby = text.into_owned();
        entry.category = cat_pattern.replace("${DIR}", &join_separated(dir, '/'));
    }
    entries.into_iter().map(|(_, _, _, _, entry)| entry).collect()
}

fn join_separated(items: &[impl AsRef<str>], separator: char) -> String {
    let capacity = items.iter().map(|item| item.as_ref().len() + 1).sum();
    items.iter().fold(String::with_capacity(capacity), |mut acc, item| {
        if !acc.is_empty() {
            acc.push(separator);
        }
        acc.push_str(item.as_ref());
        acc
    })
}

fn str_common_prefix<'a>(s1: Cow<'a, str>, s2: &str) -> Cow<'a, str> {
    let Some(((diff_index, _), _)) = s1.char_indices().zip(s2.char_indices())
        .skip_while(|((_, c1), (_, c2))| c1 == c2)
        .next() else { return s1; };
    match s1 {
        Cow::Borrowed(s) => Cow::Borrowed(&s[..diff_index]),
        Cow::Owned(mut s) => {
            s.drain(diff_index..);
            Cow::Owned(s)
        }
    }
}

fn str_common_suffix<'a>(s1: Cow<'a, str>, s2: &str) -> Cow<'a, str> {
    let Some(((diff_index, c), _)) = s1.char_indices().rev().zip(s2.char_indices().rev())
        .skip_while(|((_, c1), (_, c2))| c1 == c2)
        .next() else { return s1; };
    match s1 {
        Cow::Borrowed(s) => Cow::Borrowed(&s[diff_index + c.len_utf8()..]),
        Cow::Owned(mut s) => {
            s.drain(..diff_index + c.len_utf8());
            Cow::Owned(s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_separated() {
        assert_eq!(join_separated(&[] as &[String], '@'), "");
        assert_eq!(join_separated(&["a"], '\\'), "a");
        assert_eq!(join_separated(&["a", "b"], ':'), "a:b");
    }

    #[test]
    fn test_str_common_prefix() {
        let s1 = Cow::from("abc");
        let s2 = "abd";
        assert_eq!(str_common_prefix(s1, s2), "ab");
        let s1 = Cow::from("abc".to_string());
        let s2 = "abd";
        assert_eq!(str_common_prefix(s1, s2), "ab");
        let s1 = Cow::from("あいう");
        let s2 = "あいく";
        assert_eq!(str_common_prefix(s1, s2), "あい");
        let s1 = Cow::from("あいう".to_string());
        let s2 = "あいく";
        assert_eq!(str_common_prefix(s1, s2), "あい");
    }

    #[test]
    fn test_str_common_suffix() {
        let s1 = Cow::from("abc");
        let s2 = "pbc";
        assert_eq!(str_common_suffix(s1, s2), "bc");
        let s1 = Cow::from("abc".to_string());
        let s2 = "pbc";
        assert_eq!(str_common_suffix(s1, s2), "bc");
        let s1 = Cow::from("あいう");
        let s2 = "かいう";
        assert_eq!(str_common_suffix(s1, s2), "いう");
        let s1 = Cow::from("あいう".to_string());
        let s2 = "かいう";
        assert_eq!(str_common_suffix(s1, s2), "いう");
    }
}
