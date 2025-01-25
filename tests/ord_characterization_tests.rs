//! * Rationale
//!
//! These characterization tests intend to capture my assumptions about the Ordering of collections.
//! They function as a visual reference on how they get sorted. Additionally, this prevents me from
//! overlooking a breaking change by resulting in test failures.
//!
//! * Sorting collections
//!
//! According to std::cmp::Ord: strs, vectors, slices, and arrays are ordered lexicographically. Ord
//! is implemented for tuples up to twelve elements, but it isn't mentioned if the comparison is
//! lexicographical.
//!
//! * Sorting strs
//!
//! From the docs:
//! "Strings are ordered lexicographically by their byte values. This orders Unicode code points
//! based on their positions in the code charts. This is not necessarily the same as “alphabetical”
//! order, which varies by language and locale. Sorting strings according to culturally-accepted
//! standards requires locale-specific data that is outside the scope of the str type."
//!
//! My understanding: As long as UTF-8 doesn't move characters (breaking backwards compatibility)
//! and Rust doesn't change this requirement, I should be fine.
//!
//! * Lexicographical Comparison
//!
//! From the docs:
//! "Lexicographical comparison is an operation with the following properties:
//!
//! - Two sequences are compared element by element.
//! - The first mismatching element defines which sequence is lexicographically less or greater than
//!   the other.
//! - If one sequence is a prefix of another, the shorter sequence is lexicographically less than
//!   the other.
//! - If two sequences have equivalent elements and are of the same length, then the sequences are
//!   lexicographically equal.
//! - An empty sequence is lexicographically less than any non-empty sequence.
//! - Two empty sequences are lexicographically equal."

use std::cmp::Ordering;

#[test]
fn u8_array_equal() {
    assert_eq!([1, 2, 3].cmp(&[1, 2, 3]), Ordering::Equal);
}

#[test]
fn u8_array_first_element_different() {
    assert_eq!([4, 2, 3].cmp(&[1, 2, 3]), Ordering::Greater);
}

#[test]
fn u8_array_middle_element_different() {
    assert_eq!([1, 4, 3].cmp(&[1, 2, 3]), Ordering::Greater);
}

#[test]
fn u8_array_last_element_different() {
    assert_eq!([1, 2, 4].cmp(&[1, 2, 3]), Ordering::Greater);
}

#[test]
fn u8_vec_equal_but_different_lengths() {
    assert_eq!(vec![1, 2, 3].cmp(&vec![1, 2]), Ordering::Greater);
}

#[test]
fn u8_vec_one_empty() {
    assert_eq!(vec![1, 2, 3].cmp(&vec![]), Ordering::Greater);
}

#[test]
fn u8_vec_both_empty() {
    let v: Vec<u8> = Vec::new();
    assert_eq!(v.cmp(&vec![]), Ordering::Equal);
}

#[test]
fn str_equal() {
    assert_eq!("123".cmp("123"), Ordering::Equal);
}

#[test]
fn str_first_char_different() {
    assert_eq!("423".cmp("123"), Ordering::Greater);
}

#[test]
fn str_middle_char_different() {
    assert_eq!("143".cmp("123"), Ordering::Greater);
}

#[test]
fn str_last_char_different() {
    assert_eq!("124".cmp("123"), Ordering::Greater);
}

#[test]
fn str_equal_but_different_lengths() {
    assert_eq!("123".cmp("12"), Ordering::Greater);
}

#[test]
fn str_one_empty() {
    assert_eq!("123".cmp(""), Ordering::Greater);
}

#[test]
fn str_both_empty() {
    assert_eq!("".cmp(""), Ordering::Equal);
}

#[test]
fn str_parens_less_than_hyphen() {
    assert_eq!("(".cmp("-"), Ordering::Less);
}

#[test]
fn str_hyphen_less_than_number() {
    assert_eq!("-".cmp("0"), Ordering::Less);
}

#[test]
fn str_number_less_than_comparison() {
    assert_eq!("0".cmp("<"), Ordering::Less);
}

#[test]
fn str_comparison_less_than_upper() {
    assert_eq!("<".cmp("A"), Ordering::Less);
}

#[test]
fn str_upper_less_than_underscore() {
    assert_eq!("A".cmp("_"), Ordering::Less);
}

#[test]
fn str_underscore_less_than_lower() {
    assert_eq!("_".cmp("a"), Ordering::Less);
}

#[test]
fn str_lower_less_than_braces() {
    assert_eq!("a".cmp("{"), Ordering::Less);
}

#[test]
fn str_braces_less_than_umlaut() {
    assert_eq!("{".cmp("ä"), Ordering::Less);
}

#[test]
fn tuple_equal() {
    assert_eq!(
        (1, 'a', [1, 2, 3]).cmp(&(1, 'a', [1, 2, 3])),
        Ordering::Equal
    );
}

#[test]
fn tuple_first_element_different() {
    assert_eq!(
        (4, 'a', [1, 2, 3]).cmp(&(1, 'a', [1, 2, 3])),
        Ordering::Greater
    );
}

#[test]
fn tuple_middle_element_different() {
    assert_eq!(
        (1, 'z', [1, 2, 3]).cmp(&(1, 'a', [1, 2, 3])),
        Ordering::Greater
    );
}

#[test]
fn tuple_last_element_different() {
    assert_eq!(
        (1, 'a', [1, 4, 3]).cmp(&(1, 'a', [1, 2, 3])),
        Ordering::Greater
    );
}

#[test]
fn tuple_both_empty() {
    assert_eq!(().cmp(&()), Ordering::Equal);
}
