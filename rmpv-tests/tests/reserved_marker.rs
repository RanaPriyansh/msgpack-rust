use std::io::{Cursor, ErrorKind};

use rmpv::decode::{read_value, read_value_ref, Error};
use rmpv::{Value, ValueRef};

fn assert_invalid_data<T>(
    result: Result<T, Error>,
    cursor: &Cursor<&[u8]>,
    consumed: u64,
    trailing: &[u8],
) where
    T: std::fmt::Debug,
{
    match result {
        Err(error) => assert_eq!(ErrorKind::InvalidData, error.kind()),
        Ok(value) => panic!("unexpected decoded value: {value:?}"),
    }
    assert_eq!(consumed, cursor.position());
    assert_eq!(Some(trailing), cursor.get_ref().get(consumed as usize..));
}

#[test]
fn owned_reserved_marker_at_root_returns_invalid_data() {
    let bytes = [0xc1, 0x2a, 0x7e];
    let mut cursor = Cursor::new(&bytes[..]);
    let result = read_value(&mut cursor);

    assert_invalid_data(result, &cursor, 1, &[0x2a, 0x7e]);
}

#[test]
fn owned_reserved_marker_in_array_returns_invalid_data() {
    let bytes = [0x91, 0xc1, 0x2a, 0x7e];
    let mut cursor = Cursor::new(&bytes[..]);
    let result = read_value(&mut cursor);

    assert_invalid_data(result, &cursor, 2, &[0x2a, 0x7e]);
}

#[test]
fn owned_reserved_marker_as_map_key_returns_invalid_data() {
    let bytes = [0x81, 0xc1, 0xc0, 0x2a, 0x7e];
    let mut cursor = Cursor::new(&bytes[..]);
    let result = read_value(&mut cursor);

    assert_invalid_data(result, &cursor, 2, &[0xc0, 0x2a, 0x7e]);
}

#[test]
fn owned_reserved_marker_as_map_value_returns_invalid_data() {
    let bytes = [0x81, 0x01, 0xc1, 0x2a, 0x7e];
    let mut cursor = Cursor::new(&bytes[..]);
    let result = read_value(&mut cursor);

    assert_invalid_data(result, &cursor, 3, &[0x2a, 0x7e]);
}

#[test]
fn borrowed_reserved_marker_at_root_returns_invalid_data() {
    let bytes = [0xc1, 0x2a, 0x7e];
    let mut cursor = Cursor::new(&bytes[..]);
    let result = read_value_ref(&mut cursor);

    assert_invalid_data(result, &cursor, 1, &[0x2a, 0x7e]);
}

#[test]
fn borrowed_reserved_marker_in_array_returns_invalid_data() {
    let bytes = [0x91, 0xc1, 0x2a, 0x7e];
    let mut cursor = Cursor::new(&bytes[..]);
    let result = read_value_ref(&mut cursor);

    assert_invalid_data(result, &cursor, 2, &[0x2a, 0x7e]);
}

#[test]
fn borrowed_reserved_marker_as_map_key_returns_invalid_data() {
    let bytes = [0x81, 0xc1, 0xc0, 0x2a, 0x7e];
    let mut cursor = Cursor::new(&bytes[..]);
    let result = read_value_ref(&mut cursor);

    assert_invalid_data(result, &cursor, 2, &[0xc0, 0x2a, 0x7e]);
}

#[test]
fn borrowed_reserved_marker_as_map_value_returns_invalid_data() {
    let bytes = [0x81, 0x01, 0xc1, 0x2a, 0x7e];
    let mut cursor = Cursor::new(&bytes[..]);
    let result = read_value_ref(&mut cursor);

    assert_invalid_data(result, &cursor, 3, &[0x2a, 0x7e]);
}

#[test]
fn nil_remains_valid_at_root_and_in_containers() {
    let root = [0xc0, 0x2a];
    let mut owned_root = Cursor::new(&root[..]);
    assert_eq!(Value::Nil, read_value(&mut owned_root).unwrap());
    assert_eq!(1, owned_root.position());
    assert_eq!(Some(&[0x2a][..]), owned_root.get_ref().get(1..));

    let mut borrowed_root = Cursor::new(&root[..]);
    assert_eq!(ValueRef::Nil, read_value_ref(&mut borrowed_root).unwrap());
    assert_eq!(1, borrowed_root.position());
    assert_eq!(Some(&[0x2a][..]), borrowed_root.get_ref().get(1..));

    let array = [0x91, 0xc0, 0x2a];
    let mut owned_array = Cursor::new(&array[..]);
    assert_eq!(Value::Array(vec![Value::Nil]), read_value(&mut owned_array).unwrap());
    assert_eq!(2, owned_array.position());
    assert_eq!(Some(&[0x2a][..]), owned_array.get_ref().get(2..));

    let mut borrowed_array = Cursor::new(&array[..]);
    assert_eq!(ValueRef::Array(vec![ValueRef::Nil]), read_value_ref(&mut borrowed_array).unwrap());
    assert_eq!(2, borrowed_array.position());
    assert_eq!(Some(&[0x2a][..]), borrowed_array.get_ref().get(2..));

    let map = [0x81, 0xc0, 0xc0, 0x2a];
    let mut owned_map = Cursor::new(&map[..]);
    assert_eq!(Value::Map(vec![(Value::Nil, Value::Nil)]), read_value(&mut owned_map).unwrap());
    assert_eq!(3, owned_map.position());
    assert_eq!(Some(&[0x2a][..]), owned_map.get_ref().get(3..));

    let mut borrowed_map = Cursor::new(&map[..]);
    assert_eq!(
        ValueRef::Map(vec![(ValueRef::Nil, ValueRef::Nil)]),
        read_value_ref(&mut borrowed_map).unwrap(),
    );
    assert_eq!(3, borrowed_map.position());
    assert_eq!(Some(&[0x2a][..]), borrowed_map.get_ref().get(3..));
}

#[test]
fn reserved_marker_byte_remains_valid_binary_data() {
    let bytes = [0xc4, 0x01, 0xc1, 0x2a];
    let mut owned = Cursor::new(&bytes[..]);
    assert_eq!(Value::Binary(vec![0xc1]), read_value(&mut owned).unwrap());
    assert_eq!(3, owned.position());
    assert_eq!(Some(&[0x2a][..]), owned.get_ref().get(3..));

    let mut borrowed = Cursor::new(&bytes[..]);
    assert_eq!(ValueRef::Binary(&[0xc1]), read_value_ref(&mut borrowed).unwrap());
    assert_eq!(3, borrowed.position());
    assert_eq!(Some(&[0x2a][..]), borrowed.get_ref().get(3..));
}

#[test]
fn reserved_marker_byte_remains_valid_extension_type_and_data() {
    let bytes = [0xd4, 0xc1, 0xc1, 0x2a];
    let mut owned = Cursor::new(&bytes[..]);
    assert_eq!(Value::Ext(-63, vec![0xc1]), read_value(&mut owned).unwrap());
    assert_eq!(3, owned.position());
    assert_eq!(Some(&[0x2a][..]), owned.get_ref().get(3..));

    let mut borrowed = Cursor::new(&bytes[..]);
    assert_eq!(ValueRef::Ext(-63, &[0xc1]), read_value_ref(&mut borrowed).unwrap());
    assert_eq!(3, borrowed.position());
    assert_eq!(Some(&[0x2a][..]), borrowed.get_ref().get(3..));
}
