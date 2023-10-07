//pub fn strtok<'a>(s: &'a mut &'a str, delimiter: char) -> &'a str {}
// this won't work because &'a mut T is invariant over T
// meaning it can't change the lifetime of T
// In practice, if we pass &'a mut &'static str
// the compiler can't downgrade 'static <: 'a => 'static mut 'static str
// this mutable borrows &str for the duration of the program, making

pub fn strtok<'a, 'b>(s: &'a mut &'b str, delimiter: char) -> &'b str {
    if let Some(idx) = s.find(delimiter) {
        let prefix = &s[..idx];
        let suffix = &s[(idx + delimiter.len_utf8())..];
        *s = suffix;
        prefix
    } else {
        let prefix = *s;
        *s = "";
        prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut s /*:'static */ = "hello world";
        let hello = strtok(&mut s, ' ');
        assert_eq!(s, "world");
        assert_eq!(hello, "hello");
    }
}

/// ```compile_fail
/// use six_lists_rs::prod::IterMut;
///
///pub fn strtok<'a>(s: &'a mut &'a str, delimiter: char) -> &'a str {
///  if let Some(idx) = s.find(delimiter) {
///      let prefix = &s[..idx];
///      let suffix = &s[(idx + delimiter.len_utf8())..];
///      *s = suffix;
///      prefix
///  } else {
///      let prefix = *s;
///      *s = "";
///      prefix
///}
///
/// let mut s /*:'static */ = "hello world";
/// let hello = strtok(&mut s, ' ');
/// assert_eq!(s, "world");
/// assert_eq!(hello, "hello");
///
/// ```
#[allow(dead_code)]
fn _bad_strtok() {}
