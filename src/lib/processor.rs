// DISCUSSION POINT, I could extract a trait representing Processors (in memmory, database backed, etc)
// accepting transactions (possibly as traits, see transaction.rs)
// but i opted to keep it simple since refactoring one in, would be a trivial task
//  eg.
// use crate::transaction::Transaction;
// pub trait Processor<P, T: Transaction<P>, R> {
// 	fn accept(&mut self, tx: T) -> R;
// 	fn output(&self);
// }

use crate::fixed_decimal::FixedDecimal as FixDec;
use crate::transaction::{PaymentsTransaction, TransactionPayload as TrPl};
use std::collections::HashMap;

/// Describes the kinds of errors that may arise while a [PaymentsProcessor] processes [PaymentsTransaction]s
#[derive(Debug, PartialEq)]
pub enum ProcessTransactionError {
	NoAvailableFunds,
	AssociatedTransactionNoFound,
	TransactionCouldNotBeDisputed,
	TransactionCouldNotBeResolved,
	TransactionCouldNotBeChargedBack,
	TransactionAlreadyDisputed,
	UndisputedTransactionCannotBeResolved,
	UndisputedTransactionCannotBeChargedBack,
	ClientNotFound,
	AccountFrozen,
}
use ProcessTransactionError as TrErr;

#[doc(hidden)]
#[derive(Debug, PartialEq)]
enum TransactionState {
	Executed,
	UnderDispute,
	Resolved,
	ChargedBack,
}
use TransactionState as TrS;

#[doc(hidden)]
#[derive(Debug, Default)]
struct ClientState {
	available: FixDec,
	held: FixDec,
	locked: bool,
	transactions: HashMap<u32, (TrPl, TransactionState)>,
}

/// A type that accepts and processes [PaymentsTransaction]s and can output its state per specification
#[derive(Debug, Default)]
pub struct PaymentsProcessor {
	data: HashMap<u16, ClientState>,
}

#[doc(hidden)]
fn get_withdrawal_amount_or_err(tx: &TrPl, er: TrErr) -> Result<FixDec, TrErr> {
	match tx {
		TrPl::Withdrawal(amount) => Ok(*amount),
		_ => Err(er),
	}
}

#[doc(hidden)]
fn find_transaction(
	clientstate: &mut ClientState,
	tx_id: u32,
) -> Result<
	(
		&mut TrPl,
		&mut TransactionState,
		&mut FixDec,
		&mut FixDec,
		&mut bool,
	),
	TrErr,
> {
	if let Some((a, b)) = clientstate.transactions.get_mut(&tx_id) {
		Ok((
			a,
			b,
			&mut clientstate.available,
			&mut clientstate.held,
			&mut clientstate.locked,
		))
	} else {
		Err(TrErr::AssociatedTransactionNoFound)
	}
}

#[doc(hidden)]
impl PaymentsProcessor {
	fn find_client(&mut self, cl: u16) -> Result<&mut ClientState, TrErr> {
		if let Some(entry) = self.data.get_mut(&cl) {
			Ok(entry)
		} else {
			Err(TrErr::ClientNotFound)
		}
	}

	fn process_deposit(&mut self, cl: u16, tx_id: u32, amount: FixDec) -> Result<(), TrErr> {
		let entry = self.data.entry(cl).or_insert_with(Default::default);
		entry
			.transactions
			.insert(tx_id, (TrPl::Deposit(amount), TransactionState::Executed))
			.is_none()
			.then(|| ())
			.expect("Duplicate transaction ID enountered during deposit! which is against spec");
		entry.available = entry
			.available
			.checked_add(amount)
			.expect("Invariant Violation... Available funds underflow");
		Ok(())
	}

	//any transaction other than deposits not referencing an existing client will not add one to the Processor so that space is not wasted
	//this is ofcourse only due to spec output requirements , a proper implementation would record all (valid) transactions
	fn process_withdrawal(&mut self, cl: u16, tx_id: u32, amount: FixDec) -> Result<(), TrErr> {
		let entry = self.find_client(cl)?;
		// appplying logic from https://www.google.com/search?client=firefox-b-d&q=can+you+deposit+on+a+frozen+account%3F
		(!entry.locked).then(|| ()).ok_or(TrErr::AccountFrozen)?;
		entry
			.transactions
			.insert(tx_id, (TrPl::Withdrawal(amount), TrS::Executed))
			.is_none()
			.then(|| ())
			.expect("Duplicate transaction ID enountered during withdrawal! which is against spec");
		entry.available = entry
			.available
			.checked_sub(amount)
			.ok_or(TrErr::NoAvailableFunds)?;
		Ok(())
	}

	fn process_dispute(&mut self, cl: u16, tx_id: u32) -> Result<(), TrErr> {
		let (tx, tr_state, _, held, _) = find_transaction(self.find_client(cl)?, tx_id)?;
		//it only makes sense for withdrawals to be disputed
		let amount = get_withdrawal_amount_or_err(tx, TrErr::TransactionCouldNotBeDisputed)?;
		if *tr_state != TrS::Executed {
			return Err(TrErr::TransactionAlreadyDisputed);
		}
		//Discussion point, invariant violation (held <= total, and total cannot overflow *see deposit logic*)
		//do we panic, assert, return error, unreachable!()?
		let new_held = held
			.checked_add(amount)
			.expect("Invariant Violation... Held funds amount overflow");
		*tr_state = TrS::UnderDispute;
		*held = new_held;
		Ok(())
	}

	fn process_resolve(&mut self, cl: u16, tx_id: u32) -> Result<(), TrErr> {
		let (tx, tr_state, available, held, _) = find_transaction(self.find_client(cl)?, tx_id)?;
		let amount = get_withdrawal_amount_or_err(tx, TrErr::TransactionCouldNotBeResolved)?;
		if *tr_state != TrS::UnderDispute {
			return Err(TrErr::UndisputedTransactionCannotBeResolved);
		}
		let new_held = held
			.checked_sub(amount)
			.expect("Invariant Violation... Held funds amount underflow");
		let new_available = available
			.checked_add(amount)
			.expect("Invariant Violation... Available funds amount overflow");
		*tr_state = TrS::Resolved;
		*held = new_held;
		*available = new_available;
		Ok(())
	}

	fn process_chargeback(&mut self, cl: u16, tx_id: u32) -> Result<(), TrErr> {
		let (tx, tr_state, _, held, locked) = find_transaction(self.find_client(cl)?, tx_id)?;
		let amount = get_withdrawal_amount_or_err(tx, TrErr::TransactionCouldNotBeChargedBack)?;
		if *tr_state != TrS::UnderDispute {
			return Err(TrErr::UndisputedTransactionCannotBeChargedBack);
		}
		let new_held = held
			.checked_sub(amount)
			.expect("Invariant Violation... Held funds amount underflow");
		*tr_state = TrS::ChargedBack;
		*held = new_held;
		*locked = true;
		Ok(())
	}
}

impl PaymentsProcessor {
	///Attempts to process the provided [PaymentsTransaction]
	pub fn process_transaction(&mut self, tx: PaymentsTransaction) -> Result<(), TrErr> {
		match tx.payload {
			TrPl::Deposit(amount) => self.process_deposit(tx.client, tx.tx, amount),
			TrPl::Withdrawal(amount) => self.process_withdrawal(tx.client, tx.tx, amount),
			TrPl::Dispute => self.process_dispute(tx.client, tx.tx),
			TrPl::Resolve => self.process_resolve(tx.client, tx.tx),
			TrPl::ChargeBack => self.process_chargeback(tx.client, tx.tx),
		}
	}
}

impl std::fmt::Display for PaymentsProcessor {
	///Outputs the state of Self according to specification\
	///**client,available,held,total,locked**\
	///**{u16},{FixedDecimal},{FixedDecimal},{FixedDecimal},{true/false}**\
	///**.**\
	///**.**\
	///**.**
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
		//spec is unclear if we need to print the header if there are no clients
		writeln!(f, "client,available,held,total,locked")?;
		//spec sugests that the output is sorted by client ID (so unit tests are also built using that assumption)
		//I will not use a std::collections::BTreeMap internally since it makes my searches O(logN)
		//but i will collect into a Vec<> and sort before printing (heavy operation but very infrequent)
		let mut sorted_client_records: Vec<_> = self.data.iter().collect();
		sorted_client_records.sort_unstable_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
		for (cl, record) in sorted_client_records {
			writeln!(
				f,
				"{},{},{},{},{}",
				cl,
				record.available,
				record.held,
				record
					.available
					.checked_add(record.held)
					.expect("Invariant Violation, total funds overflow"),
				record.locked,
			)?;
		}
		Ok(())
	}
}
