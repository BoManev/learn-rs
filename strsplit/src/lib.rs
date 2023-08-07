//!
//#![warn(missing_copy_implementations, missing_docs)]

/*
* Weird '_
* fn foo(x: &str, y: &'_ str) -> &'_ str {}
* fn foo<'x, 'y>(x: &'x str, y: &'y str) -> &'x str {}
*/
#[derive(Debug)]
pub struct StrSplit<'haystack, D> {
    remainder: Option<&'haystack str>,
    delimeter: D,
}

// str -> [char] (pointer without size) (can be anywhere)
// &str -> &[char] (reference with size)
// String -> Vec<char> (heap allocated)
//
// String -> &str (cheap -- AsRef)
// &str -> String (expensice -- memcpy)

impl<'haystack, D> StrSplit<'haystack, D> {
    pub fn new(haystack: &'haystack str, delimeter: D) -> Self {
        Self {
            remainder: Some(haystack),
            delimeter,
        }
    }
}

pub trait Delimiter {
    fn find_next(&self, s: &str) -> Option<(usize, usize)>;
}

impl Delimiter for &str {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        s.find(self).map(|start| (start, start + self.len()))
    }
}

impl Delimiter for char {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        s.char_indices()
            .find(|(_, c)| c == self)
            .map(|(start, _)| (start, start + self.len_utf8()))
    }
}
// let x: StrSplit
// for part in x {}
impl<'haystack, D> Iterator for StrSplit<'haystack, D>
where
    D: Delimiter
{
    type Item = &'haystack str;
    fn next(&mut self) -> Option<Self::Item> {
        let remainder = self.remainder.as_mut()?;
        if let Some((start, end)) = self.delimeter.find_next(remainder) {
            let until_delim = &remainder[..start];
            *remainder = &remainder[end..];
            Some(until_delim)
        } else {
            self.remainder.take()
        }
    }
}

pub fn until_char(s: &str, c: char) -> &'_ str {
    StrSplit::new(s, c)
        .next()
        .expect("StrSplit always gives at least one result")
}

#[test]
fn until_char_test() {
    assert_eq!(until_char("hello world", 'o'), "hell");
}
#[test]
fn it_works() {
    let haystack = "a b c d e";
    let letters: Vec<_> = StrSplit::new(haystack, " ").collect();
    assert_eq!(letters, vec!["a", "b", "c", "d", "e"]);
}

#[test]
fn tail() {
    let haystack = "a b c d ";
    let letters: Vec<_> = StrSplit::new(haystack, " ").collect();
    assert_eq!(letters, vec!["a", "b", "c", "d", ""]);
}
