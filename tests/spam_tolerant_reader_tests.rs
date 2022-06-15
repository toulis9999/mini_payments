extern crate lib;

use lib::{SpamReaderError, SpamTolerantReader};
use std::num::NonZeroUsize;

const EMPTY_SLICE: &[u8] = &[];

#[test]
fn reader_of_empty_returns_eof_err() {
	let buf = "".as_bytes();
	let mut spr = SpamTolerantReader::new(buf, b' ', NonZeroUsize::new(5).unwrap());
	let next_res = spr.get_next();
	assert_eq!(next_res, Err(SpamReaderError::EOFReached))
}

#[test]
fn reader_with_consecutive_delimiters_returns_empty_slices() {
	let buf = "   ".as_bytes();
	let mut spr = SpamTolerantReader::new(buf, b' ', NonZeroUsize::new(4).unwrap());
	assert_eq!(spr.get_next(), Ok(EMPTY_SLICE));
	assert_eq!(spr.get_next(), Ok(EMPTY_SLICE));
	assert_eq!(spr.get_next(), Ok(EMPTY_SLICE));
	assert_eq!(spr.get_next(), Err(SpamReaderError::EOFReached))
}

#[test]
fn queries_after_tolerance_exceeded_result_in_tolerance_exceeded_err() {
	let buf = "qqqe   qwe  qwee qq qq qq".as_bytes();
	let mut spr = SpamTolerantReader::new(buf, b' ', NonZeroUsize::new(4).unwrap());
	assert_eq!(spr.get_next(), Err(SpamReaderError::ToleranceExceeded));
	assert_eq!(spr.get_next(), Err(SpamReaderError::ToleranceExceeded));
}

#[test]
fn reader_with_slice_of_more_than_tolerance_returns_err() {
	let buf = "qqq   qwe  qwee ".as_bytes();
	let mut spr = SpamTolerantReader::new(buf, b' ', NonZeroUsize::new(4).unwrap());
	assert_eq!(spr.get_next(), Ok("qqq".as_bytes()));
	assert_eq!(spr.get_next(), Ok(EMPTY_SLICE));
	assert_eq!(spr.get_next(), Ok(EMPTY_SLICE));
	assert_eq!(spr.get_next(), Ok("qwe".as_bytes()));
	assert_eq!(spr.get_next(), Ok(EMPTY_SLICE));
	assert_eq!(spr.get_next(), Err(SpamReaderError::ToleranceExceeded));
}
