use crate::fixed_decimal::FixedDecimal;

//Discussion point, I could make Client and Tx types using:
//https://doc.rust-lang.org/rust-by-example/generics/new_types.html
//didn't do it so i can keep the code compact

/// Describes the type of transaction (with associated amounts for deposits and withdrawals)
#[derive(Debug, PartialEq)]
pub enum TransactionPayload {
	Deposit(FixedDecimal),
	Withdrawal(FixedDecimal),
	Dispute,
	Resolve,
	ChargeBack,
}

/// Describes a payment transaction
#[derive(Debug, PartialEq)]
pub struct PaymentsTransaction {
	pub client: u16,
	pub tx: u32,
	pub payload: TransactionPayload,
}

// DISCUSSION POINT, I could extract a trait representing transactions
// The difficult part is to decide on "associated types vs generics for client, id and payload types,
// and wether or not any of those types would apply for all kinds of transactions in the 1st place!
// but i opted to keep it simple since refactoring one in, would be a trivial task
//  eg.
// pub trait Transaction<PayloadT>: std::str::FromStr {
// 	type ClientID;
// 	type TransactionID;
// 	fn create(txt: &str) -> Result<Self, Self::Err>;
// 	fn get_client_id(&self) -> Self::ClientID;
// 	fn get_id(&self) -> Self::TransactionID;
// 	fn get_payload(&self) -> PayloadT;
// }

/// Type of error that happens during &str to -> PaymentsTransaction conversion
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum ErrorKind {
	EmptySection,
	CouldNotParseSection,
	OutOfBoundsSection,
	UnknownTransactionType,
	UnexpectedErrorType,
	UnexpectedTrailingSection,
	MissingTransactionAmount,
}

impl std::str::FromStr for PaymentsTransaction {
	type Err = ErrorKind;
	/// Valid input for conversion is a comma separated list of fields matching one of the following patterns (ws=whitespace):
	/// - {ws}**deposit**{ws},{ws}**u16**{ws},{ws}**u32**{ws},{ws}**{number}.{(1..=4)digits}**{ws}
	/// - {ws}**withdrawal**{ws},{ws}**u16**{ws},{ws}**u32**{ws},{ws}**{number}.{(1..=4)digits}**{ws}
	/// - {ws}**dispute**{ws},{ws}**u16**{ws},{ws}**u32**{ws}
	/// - {ws}**resolve**{ws},{ws}**u16**{ws},{ws}**u32**{ws}
	/// - {ws}**chargeback**{ws},{ws}**u16**{ws},{ws}**u32**{ws}
	/// ```
	/// use lib::PaymentsTransaction;
	/// use std::str::FromStr;
	/// use lib::TransactionPayload;
	/// use lib::FixedDecimal;
	///
	/// let tr = PaymentsTransaction::from_str("deposit, 321, 1, 100.0");
	/// assert!(tr.is_ok());
	/// let tr = tr.unwrap();
	/// assert_eq!(tr.client, 321);
	/// assert_eq!(tr.tx, 1);
	/// let amount = FixedDecimal::from_str("100.0");
	/// assert!(amount.is_ok());
	/// assert_eq!(tr.payload, TransactionPayload::Deposit(amount.unwrap()));
	/// ```
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut it = s.split(',').map(|x| x.trim());
		let transaction_sections = [it.next(), it.next(), it.next()];
		// Deliberately not iterating over the amount here
		// since extract_payload() will conditionally next() the iterator on the correct conditions
		let payload = extract_payload(transaction_sections[0], &mut it)?;
		if it.next().is_some() {
			Err(ErrorKind::UnexpectedTrailingSection)
		} else {
			Ok(PaymentsTransaction {
				client: extract_number(transaction_sections[1])?,
				tx: extract_number(transaction_sections[2])?,
				payload,
			})
		}
	}
}

#[doc(hidden)]
fn extract_number<T>(txt: Option<&str>) -> Result<T, ErrorKind>
where
	T: std::str::FromStr<Err = std::num::ParseIntError>,
{
	let parse_res = txt.ok_or(ErrorKind::EmptySection)?.parse::<T>();
	if let Err(t) = parse_res {
		Err(match t.kind() {
			std::num::IntErrorKind::Empty => ErrorKind::EmptySection,
			std::num::IntErrorKind::InvalidDigit => ErrorKind::CouldNotParseSection,
			std::num::IntErrorKind::PosOverflow => ErrorKind::OutOfBoundsSection,
			std::num::IntErrorKind::NegOverflow => ErrorKind::OutOfBoundsSection,
			//std::num::IntErrorKind::Zero => unreachable!("Invariant violation, zeroes allowed for client and transaction IDs"),
			_ => ErrorKind::UnexpectedErrorType,
		})
	} else {
		Ok(parse_res.unwrap())
	}
}

#[doc(hidden)]
fn extract_payload<'a>(
	txt_type: Option<&str>,
	it: &mut impl Iterator<Item = &'a str>,
) -> Result<TransactionPayload, ErrorKind> {
	fn get_amount<'a>(it: &mut impl Iterator<Item = &'a str>) -> Result<FixedDecimal, ErrorKind> {
		it.next()
			.ok_or(ErrorKind::MissingTransactionAmount)?
			.parse::<FixedDecimal>()
			//TODO, maybe make this error more fine grained
			.map_err(|_| ErrorKind::CouldNotParseSection)
	}
	match txt_type.ok_or(ErrorKind::EmptySection)? {
		"deposit" => Ok(TransactionPayload::Deposit(get_amount(it)?)),
		"withdrawal" => Ok(TransactionPayload::Withdrawal(get_amount(it)?)),
		"dispute" => Ok(TransactionPayload::Dispute),
		"resolve" => Ok(TransactionPayload::Resolve),
		"chargeback" => Ok(TransactionPayload::ChargeBack),
		_ => Err(ErrorKind::UnknownTransactionType),
	}
}
