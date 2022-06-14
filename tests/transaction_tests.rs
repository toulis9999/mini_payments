extern crate lib;

use lib::{FixedDecimal, PaymentsTransaction, TransactionError, TransactionPayload};
use std::str::FromStr;

#[test]
fn parse_valid_deposit_pattern_is_ok() {
	let tx = PaymentsTransaction::from_str("deposit, 321, 1, 100.0");
	assert!(tx.is_ok());
	let tx = tx.unwrap();
	assert_eq!(tx.client, 321);
	assert_eq!(tx.tx, 1);
	assert_eq!(
		tx.payload,
		TransactionPayload::Deposit(FixedDecimal::from_str("100.0").unwrap())
	);
}

#[test]
fn parse_valid_deposit_without_amount_is_err() {
	let tx = PaymentsTransaction::from_str("deposit, 321, 1");
	assert_eq!(tx, Err(TransactionError::MissingTransactionAmount));
}

#[test]
fn parse_valid_withdrawal_pattern_is_ok() {
	let tx = PaymentsTransaction::from_str("withdrawal, 321, 1, 100.0");
	assert!(tx.is_ok());
	let tx = tx.unwrap();
	assert_eq!(tx.client, 321);
	assert_eq!(tx.tx, 1);
	assert_eq!(
		tx.payload,
		TransactionPayload::Withdrawal(FixedDecimal::from_str("100.0").unwrap())
	);
}

#[test]
fn parse_valid_withdrawal_without_amount_is_err() {
	let tx = PaymentsTransaction::from_str("deposit, 321, 1");
	assert_eq!(tx, Err(TransactionError::MissingTransactionAmount));
}

#[test]
fn parse_valid_dispute_pattern_is_ok() {
	let tx = PaymentsTransaction::from_str("dispute, 111, 321");
	assert!(tx.is_ok());
	let tx = tx.unwrap();
	assert_eq!(tx.client, 111);
	assert_eq!(tx.tx, 321);
	assert_eq!(tx.payload, TransactionPayload::Dispute);
}

#[test]
fn parse_valid_resolve_pattern_is_ok() {
	let tx = PaymentsTransaction::from_str("resolve, 666, 777");
	assert!(tx.is_ok());
	let tx = tx.unwrap();
	assert_eq!(tx.client, 666);
	assert_eq!(tx.tx, 777);
	assert_eq!(tx.payload, TransactionPayload::Resolve);
}

#[test]
fn parse_valid_chargeback_pattern_is_ok() {
	let tx = PaymentsTransaction::from_str("chargeback, 111, 321");
	assert!(tx.is_ok());
	let tx = tx.unwrap();
	assert_eq!(tx.client, 111);
	assert_eq!(tx.tx, 321);
	assert_eq!(tx.payload, TransactionPayload::ChargeBack);
}

#[test]
fn parse_unknown_transaction_type_is_err() {
	let tx = PaymentsTransaction::from_str("another_transaction, 111, 321, 456");
	assert_eq!(tx, Err(TransactionError::UnknownTransactionType));
}

#[test]
fn transaction_with_extra_trailing_sections_is_err() {
	let tx = PaymentsTransaction::from_str("deposit, 111, 321, 456.456, qwerty");
	assert_eq!(tx, Err(TransactionError::UnexpectedTrailingSection));
}

#[test]
fn transaction_with_incorrect_amount_is_err() {
	let tx = PaymentsTransaction::from_str("deposit, 111, 321, 456");
	assert_eq!(tx, Err(TransactionError::CouldNotParseSection));
}

#[test]
fn transaction_with_out_of_bounds_client_is_err() {
	let tx = PaymentsTransaction::from_str("withdrawal, 99999, 321, 100.0");
	assert_eq!(tx, Err(TransactionError::OutOfBoundsSection));
}

#[test]
fn transaction_with_out_of_bounds_id_is_err() {
	let tx = PaymentsTransaction::from_str("withdrawal, 0, 999999999999999999, 100.0");
	assert_eq!(tx, Err(TransactionError::OutOfBoundsSection));
}
