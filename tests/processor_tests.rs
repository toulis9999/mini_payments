extern crate lib;

use lib::{PaymentsProcessor, PaymentsTransaction, ProcessTransactionError as PTErr};
use std::str::FromStr;

#[test]
fn deposit_adds_client() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 100.0").unwrap());
	assert!(res.is_ok());
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,100.0,0.0,100.0,false"));
}

#[test]
fn deposits_adds_to_available() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 4, 1, 100.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 4, 2, 500.0").unwrap());
	assert!(res.is_ok());
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("4,600.0,0.0,600.0,false"));
}

#[test]
fn different_clients_balances_are_separate() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 4, 1, 100.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 3, 2, 500.0").unwrap());
	assert!(res.is_ok());
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 3);
	assert_eq!(output.lines().nth(1), Some("3,500.0,0.0,500.0,false"));
	assert_eq!(output.lines().nth(2), Some("4,100.0,0.0,100.0,false"));
}

#[test]
fn withdrawal_unknown_client_is_err() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 321, 1, 100.0").unwrap());
	assert_eq!(res, Err(PTErr::ClientNotFound));
}

#[test]
fn withdrawal_with_enough_funds_is_ok() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 4, 1, 600.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 4, 3, 150.0").unwrap());
	assert!(res.is_ok());
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("4,450.0,0.0,450.0,false"));
}

#[test]
fn withdrawal_not_enough_funds_is_err_and_not_processed() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 4, 1, 100.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 4, 3, 150.0").unwrap());
	assert_eq!(res, Err(PTErr::NoAvailableFunds));
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("4,100.0,0.0,100.0,false"));
}

#[test]
fn dispute_unknown_client_is_err() {
	let mut proc: PaymentsProcessor = Default::default();
	let res = proc.process_transaction(PaymentsTransaction::from_str("dispute, 321, 1").unwrap());
	assert_eq!(res, Err(PTErr::ClientNotFound));
}

#[test]
fn dispute_on_deposit_is_err_and_not_processed() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("dispute, 321, 1").unwrap());
	assert_eq!(res, Err(PTErr::TransactionCouldNotBeDisputed));
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,150.0,0.0,150.0,false"));
}

#[test]
fn dispute_on_withdrawal_is_ok_and_reduces_available_and_increases_held() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 321, 2, 50.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("dispute, 321, 2").unwrap());
	assert!(res.is_ok());
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,100.0,50.0,150.0,false"));
}

#[test]
fn resolve_unknown_client_is_err() {
	let mut proc: PaymentsProcessor = Default::default();
	let res = proc.process_transaction(PaymentsTransaction::from_str("resolve, 321, 1").unwrap());
	assert_eq!(res, Err(PTErr::ClientNotFound));
}

#[test]
fn resolve_on_deposit_is_err_and_not_processed() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("resolve, 321, 1").unwrap());
	assert_eq!(res, Err(PTErr::TransactionCouldNotBeResolved));
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,150.0,0.0,150.0,false"));
}

#[test]
fn resolve_on_undisputed_withdrawal_is_err_and_not_processed() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 321, 2, 50.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("resolve, 321, 2").unwrap());
	assert_eq!(res, Err(PTErr::UndisputedTransactionCannotBeResolved));
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,100.0,0.0,100.0,false"));
}

#[test]
fn resolve_on_disputed_withdrawal_is_ok_and_processed() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 321, 2, 50.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("dispute, 321, 2").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("resolve, 321, 2").unwrap());
	assert!(res.is_ok());
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,150.0,0.0,150.0,false"));
}

#[test]
fn chargeback_unknown_client_is_err() {
	let mut proc: PaymentsProcessor = Default::default();
	let res = proc.process_transaction(PaymentsTransaction::from_str("chargeback, 321, 1").unwrap());
	assert_eq!(res, Err(PTErr::ClientNotFound));
}

#[test]
fn chargeback_on_deposit_is_err_and_not_processed() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("chargeback, 321, 1").unwrap());
	assert_eq!(res, Err(PTErr::TransactionCouldNotBeChargedBack));
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,150.0,0.0,150.0,false"));
}

#[test]
fn chargeback_on_undisputed_withdrawal_is_err_and_not_processed() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 321, 2, 50.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("chargeback, 321, 2").unwrap());
	assert_eq!(res, Err(PTErr::UndisputedTransactionCannotBeChargedBack));
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,100.0,0.0,100.0,false"));
}

#[test]
fn chargeback_on_disputed_withdrawal_is_ok_and_processed() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 321, 2, 50.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("dispute, 321, 2").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("chargeback, 321, 2").unwrap());
	assert!(res.is_ok());
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,100.0,0.0,100.0,true"));
}

#[test]
fn withdrawal_on_locked_account_is_err_and_not_processed() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 321, 2, 50.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("dispute, 321, 2").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("chargeback, 321, 2").unwrap());
	assert!(res.is_ok());
	let output_before = format!("{}", proc);
	assert_eq!(output_before.lines().count(), 2);
	assert_eq!(
		output_before.lines().nth(1),
		Some("321,100.0,0.0,100.0,true")
	);
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 321, 3, 10.0").unwrap());
	assert_eq!(res, Err(PTErr::AccountFrozen));
	let output_after = format!("{}", proc);
	assert_eq!(output_after, output_before);
}

#[test]
fn dispute_on_invalid_transaction_id_is_err() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 321, 2, 50.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("dispute, 321, 4").unwrap());
	assert_eq!(res, Err(PTErr::AssociatedTransactionNoFound));
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,100.0,0.0,100.0,false"));
}

#[test]
fn resolve_on_invalid_transaction_id_is_err() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 321, 2, 50.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("resolve, 321, 4").unwrap());
	assert_eq!(res, Err(PTErr::AssociatedTransactionNoFound));
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,100.0,0.0,100.0,false"));
}

#[test]
fn chargeback_on_invalid_transaction_id_is_err() {
	let mut proc: PaymentsProcessor = Default::default();
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("deposit, 321, 1, 150.0").unwrap());
	assert!(res.is_ok());
	let res =
		proc.process_transaction(PaymentsTransaction::from_str("withdrawal, 321, 2, 50.0").unwrap());
	assert!(res.is_ok());
	let res = proc.process_transaction(PaymentsTransaction::from_str("chargeback, 321, 4").unwrap());
	assert_eq!(res, Err(PTErr::AssociatedTransactionNoFound));
	let output = format!("{}", proc);
	assert_eq!(output.lines().count(), 2);
	assert_eq!(output.lines().nth(1), Some("321,100.0,0.0,100.0,false"));
}
