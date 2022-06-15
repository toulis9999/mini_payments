#![doc(html_no_source)]

use std::env;
use std::io::BufReader;
use std::num::NonZeroUsize;
use std::str::FromStr;
extern crate lib;

use lib::{
	FixedDecimalMAXDISPLEN, PaymentsProcessor, PaymentsTransaction, SpamReaderError,
	SpamTolerantReader,
};

const MAX_TRANSACTION_LEN: usize =
	"withdrawal".len() + 5 /*u16 max digits*/ + 10 /*u32 max digits*/+ FixedDecimalMAXDISPLEN + 1 /*\n*/;

macro_rules! skip_fail {
	($res:expr, $id:ident) => {
		match $res {
			Ok(val) => val,
			Err(e) => {
				eprintln!("Skipping transaction [{:?}]. Error [{:?}]", $id, e);
				continue;
			}
		}
	};
}

#[doc(hidden)]
fn main() -> Result<(), String> {
	let input_file = env::args().nth(1).ok_or("No input file detected")?;
	let f = std::fs::File::open(input_file).map_err(|_| "File not found!")?;
	let tolerance =
		NonZeroUsize::new(MAX_TRANSACTION_LEN * 5).ok_or("Zero bytes spam tolerance is not allowed")?;
	let mut sp = SpamTolerantReader::new(f, b'\n', tolerance);
	let mut pr = PaymentsProcessor::default();
	loop {
		let n = sp.get_next();
		match n {
			Err(SpamReaderError::ToleranceExceeded) => {
				return Err(
					"Terminating... incorrect buffer beyond Tolerance threshold detected!".to_owned(),
				)
			}
			Err(SpamReaderError::IOError(e)) => match e {
				std::io::ErrorKind::Interrupted => continue, //retry according to https://doc.rust-lang.org/std/io/trait.Read.html#tymethod.read
				_ => return Err(format!("Terminating... Irrecoverable IO error: [{}]", e)),
			},
			Err(SpamReaderError::EOFReached) => break,
			Ok(buf) => {
				let tr_str = skip_fail!(std::str::from_utf8(buf), buf);
				let tr = skip_fail!(PaymentsTransaction::from_str(tr_str), tr_str);
				skip_fail!(pr.process_transaction(tr), tr_str);
			}
		}
	}
	print!("{}", pr);
	Ok(())
}
