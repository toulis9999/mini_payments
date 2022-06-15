extern crate lib;

use lib::FixedDecimalMAX;
use lib::FixedDecimalMAXDISPLEN;
use lib::{FixedDecimal, FixedDecimalError};
use std::str::FromStr;

#[test]
fn zero_with_at_least_one_decimal_digit_is_ok() {
	let res = FixedDecimal::from_str("0.0");
	assert!(res.is_ok());
	let dec = res.unwrap();
	assert_eq!(dec.get_whole_part(), 0);
	assert_eq!(dec.get_decimal_part(), (0, 0));
}

#[test]
fn no_decimal_digits_present_is_err() {
	let res = FixedDecimal::from_str("1234");
	assert_eq!(res, Err(FixedDecimalError::InvalidFormat));
}

#[test]
fn no_whole_digits_present_is_err() {
	let res = FixedDecimal::from_str(".1234");
	assert_eq!(res, Err(FixedDecimalError::WholePartParseError));
}

#[test]
fn more_that_4_digits_is_err() {
	let res = FixedDecimal::from_str("0.00001");
	assert!(res.is_err());
}

#[test]
fn more_that_4_zero_decimals_is_err() {
	let res = FixedDecimal::from_str("1.00000");
	assert!(res.is_err());
}

#[test]
fn zero_whole_with_one_leading_zero_decimal_prints_as_expected() {
	let res = FixedDecimal::from_str("0.0321");
	assert!(res.is_ok());
	let dec = res.unwrap();
	assert_eq!(dec.get_whole_part(), 0);
	assert_eq!(dec.get_decimal_part(), (321, 1));
	assert_eq!(dec.to_string(), "0.0321");
}

#[test]
fn zero_whole_with_two_leading_zero_decimals_prints_as_expected() {
	let res = FixedDecimal::from_str("0.0021");
	assert!(res.is_ok());
	let dec = res.unwrap();
	assert_eq!(dec.get_whole_part(), 0);
	assert_eq!(dec.get_decimal_part(), (21, 2));
	assert_eq!(dec.to_string(), "0.0021");
}

#[test]
fn zero_whole_with_three_leading_zero_decimals_prints_as_expected() {
	let res = FixedDecimal::from_str("0.0009");
	assert!(res.is_ok());
	let dec = res.unwrap();
	assert_eq!(dec.get_whole_part(), 0);
	assert_eq!(dec.get_decimal_part(), (9, 3));
	assert_eq!(dec.to_string(), "0.0009");
}

#[test]
fn zero_whole_with_four_zero_decimals_prints_as_expected() {
	let res = FixedDecimal::from_str("0.0000");
	assert!(res.is_ok());
	let dec = res.unwrap();
	assert_eq!(dec.get_whole_part(), 0);
	assert_eq!(dec.get_decimal_part(), (0, 0));
	assert_eq!(dec.to_string(), "0.0");
}

#[test]
fn within_bounds_prints_as_expected() {
	let res = FixedDecimal::from_str("1234.9876");
	assert!(res.is_ok());
	let dec = res.unwrap();
	assert_eq!(dec.get_whole_part(), 1234);
	assert_eq!(dec.get_decimal_part(), (9876, 0));
	assert_eq!(dec.to_string(), "1234.9876");
}

#[test]
fn out_of_bounds_is_err() {
	let res = FixedDecimal::from_str("99999999999999999999999999999.9876");
	assert_eq!(res, Err(FixedDecimalError::WholePartParseError));
}

#[test]
fn incorrect_decimal_is_err() {
	let res = FixedDecimal::from_str("9999.$876");
	assert_eq!(res, Err(FixedDecimalError::DecimalPartParseError));
}

#[test]
fn non_overflowing_addition_works_as_expected() {
	let res1 = FixedDecimal::from_str("19.9999").unwrap();
	let res2 = FixedDecimal::from_str("0.0001").unwrap();
	let sum = res1.checked_add(res2);
	assert!(sum.is_some());
	let sum = sum.unwrap();
	assert_eq!(sum.get_whole_part(), 20);
	assert_eq!(sum.get_decimal_part(), (0, 0));
	assert_eq!(sum.to_string(), "20.0");
}

#[test]
fn overflowing_addition_is_err() {
	let res1 = FixedDecimalMAX;
	let res2 = FixedDecimal::from_str("0.0001").unwrap();
	let sum = res1.checked_add(res2);
	assert!(sum.is_none());
}

#[test]
fn max_plus_zero_positive_is_ok() {
	let res1 = FixedDecimalMAX;
	let res2 = FixedDecimal::from_str("0.0000").unwrap();
	let sum = res1.checked_add(res2);
	assert!(sum.is_some());
	let sum = sum.unwrap();
	assert_eq!(sum, FixedDecimalMAX);
}

#[test]
fn non_underflowing_subtraction_works_as_expected() {
	let res1 = FixedDecimal::from_str("19.8888").unwrap();
	let res2 = FixedDecimal::from_str("9.9999").unwrap();
	let sum = res1.checked_sub(res2);
	assert!(sum.is_some());
	let sum = sum.unwrap();
	assert_eq!(sum.get_whole_part(), 9);
	assert_eq!(sum.get_decimal_part(), (8889, 0));
	assert_eq!(sum.to_string(), "9.8889");
}

#[test]
fn underflowing_subtraction_is_err() {
	let res1 = FixedDecimal::from_str("765.432").unwrap();
	let res2 = FixedDecimal::from_str("765.433").unwrap();
	let sum = res1.checked_sub(res2);
	assert!(sum.is_none());
}

#[test]
fn zero_minus_zero_is_ok() {
	let res1: FixedDecimal = Default::default();
	let res2 = Default::default();
	let sum = res1.checked_sub(res2);
	assert!(sum.is_some());
	let sum = sum.unwrap();
	assert_eq!(sum, Default::default());
}

#[test]
fn max_has_correct_disp_len() {
	let res1 = FixedDecimalMAX;
	assert_eq!(res1.to_string().len(), FixedDecimalMAXDISPLEN);
}

#[test]
fn simple_subtraction_works_ok() {
	let res1 = FixedDecimal::from_str("3.01").unwrap();
	let res2 = FixedDecimal::from_str("1.95").unwrap();
	let res = res1.checked_sub(res2).unwrap();
	assert_eq!(res.get_decimal_part(), (600, 1));
	assert_eq!(res.get_whole_part(), 1);
	assert_eq!(res.to_string(), "1.06");
}

#[test]
fn zero_with_4rd_decimal_nonzero_only_is_ok_invariants_hold() {
	let res = FixedDecimal::from_str("0.0001").unwrap();
	assert_eq!(res.get_decimal_part(), (1, 3));
	assert_eq!(res.get_whole_part(), 0);
	assert_eq!(res.to_string(), "0.0001");
}

#[test]
fn zero_with_3rd_decimal_nonzero_only_is_ok_invariants_hold() {
	let res = FixedDecimal::from_str("0.0010").unwrap();
	assert_eq!(res.get_decimal_part(), (10, 2));
	assert_eq!(res.get_whole_part(), 0);
	assert_eq!(res.to_string(), "0.001");
}

#[test]
fn zero_with_2rd_decimal_nonzero_only_is_ok_invariants_hold() {
	let res = FixedDecimal::from_str("0.0100").unwrap();
	assert_eq!(res.get_decimal_part(), (100, 1));
	assert_eq!(res.get_whole_part(), 0);
	assert_eq!(res.to_string(), "0.01");
}

#[test]
fn nonzero_whole_with_4rd_decimal_nonzero_only_is_ok_invariants_hold() {
	let res = FixedDecimal::from_str("321.0001").unwrap();
	assert_eq!(res.get_decimal_part(), (1, 3));
	assert_eq!(res.get_whole_part(), 321);
	assert_eq!(res.to_string(), "321.0001");
}

#[test]
fn nonzero_whole_with_3rd_decimal_nonzero_only_is_ok_invariants_hold() {
	let res = FixedDecimal::from_str("321.0010").unwrap();
	assert_eq!(res.get_decimal_part(), (10, 2));
	assert_eq!(res.get_whole_part(), 321);
	assert_eq!(res.to_string(), "321.001");
}

#[test]
fn nonzero_whole_with_2rd_decimal_nonzero_only_is_ok_invariants_hold() {
	let res = FixedDecimal::from_str("321.0100").unwrap();
	assert_eq!(res.get_decimal_part(), (100, 1));
	assert_eq!(res.get_whole_part(), 321);
	assert_eq!(res.to_string(), "321.01");
}
