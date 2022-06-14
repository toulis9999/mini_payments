//Change Underlying, NUM_DIGITS and MAX_DISP_LEN to increase/decrease the precision of FixedDecimal

#[doc(hidden)]
type UnderLying = u64;

#[doc(hidden)]
const NUM_DECIMAL_DIGITS: u32 = 4;

#[doc(hidden)]
const DIVISOR: UnderLying = {
	let ten: UnderLying = 10;
	ten.pow(NUM_DECIMAL_DIGITS)
};

///The max number of chars a [FixedDecimal] can produce when displayed
pub const MAX_DISP_LEN: usize = "1844674407370955.1615".len();

///The Maximum value of a [FixedDecimal]
pub const MAX: FixedDecimal = FixedDecimal {
	#[doc(hidden)]
	data: UnderLying::MAX,
};

/// A type to represent 4 decimal digit precision numbers
/// This representation is enough for the problem constraints
/// but for a more generalised case, this project could adopt something like [rust_decimal](https://crates.io/crates/rust_decimal)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedDecimal {
	#[doc(hidden)]
	data: UnderLying,
}

impl std::fmt::Display for FixedDecimal {
	/// Display will always output the decimal separator and at least one decimal digit
	/// even for whole numbers according to spec
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}.", self.get_whole_part())?;
		let (dec, i) = self.get_decimal_part();
		for _ in 0..i {
			write!(f, "0")?;
		}
		write!(f, "{}", dec)
	}
}

// using advice from :
// https://stackoverflow.com/questions/55572098/how-to-construct-a-parseinterror-in-my-own-code
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
	InvalidFormat,
	WholePartParseError,
	DecimalPartParseError,
	DecimalOverflow,
	OverFlow,
}

impl std::str::FromStr for FixedDecimal {
	type Err = ErrorKind;
	/// Spec dictates that the number is coming in the form of **{whole part}.{decimal part}**
	/// with decimal part digits in the range **(1..=4)**
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (whole, decimal) = s.split_once('.').ok_or(ErrorKind::InvalidFormat)?;
		//TODO make this error checking more fine grained. aka report exact int parse errors
		let whole_num = UnderLying::from_str(whole).map_err(|_| ErrorKind::WholePartParseError)?;
		let decimal_num =
			UnderLying::from_str(decimal).map_err(|_| ErrorKind::DecimalPartParseError)?;
		if decimal.len() > NUM_DECIMAL_DIGITS as usize || decimal_num > DIVISOR {
			Err(ErrorKind::DecimalOverflow)
		} else {
			let ten: UnderLying = 10;
			let composed_num = whole_num
				.checked_mul(DIVISOR)
				.and_then(|x| {
					x.checked_add(decimal_num * ten.pow(NUM_DECIMAL_DIGITS - decimal.len() as u32))
				})
				.ok_or(ErrorKind::OverFlow)?;
			Ok(FixedDecimal { data: composed_num })
		}
	}
}

impl FixedDecimal {
	pub fn checked_add(self, rhs: FixedDecimal) -> Option<Self> {
		let inner = self.data.checked_add(rhs.data)?;
		Some(FixedDecimal { data: inner })
	}

	pub fn checked_sub(self, rhs: FixedDecimal) -> Option<Self> {
		let inner = self.data.checked_sub(rhs.data)?;
		Some(FixedDecimal { data: inner })
	}

	pub fn get_whole_part(&self) -> UnderLying {
		(self.data - self.data % DIVISOR) / DIVISOR
	}

	pub fn get_decimal_part(&self) -> (UnderLying, usize) {
		let mut d = self.data % DIVISOR;
		if d == 0 {
			return (0, 0);
		}
		let mut div = DIVISOR;
		while d % 10 == 0 {
			d /= 10;
			div /= 10;
		}
		let mut leading_zeroes = 0;
		while d / div == 0 {
			leading_zeroes += 1;
			div /= 10;
		}
		(d, leading_zeroes - 1)
	}
}
