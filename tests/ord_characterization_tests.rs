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

#[ignore]
#[test]
fn strings() {
    assert!(false);
}

#[ignore]
#[test]
fn tuples() {
    assert!(false);
}
