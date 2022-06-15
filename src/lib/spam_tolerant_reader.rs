use std::io::{BufRead, BufReader, Read};
use std::num::NonZeroUsize;

//nightly feature 😞
//use std::str::pattern::Pattern; for delimiter

//this does not adhere to Iterator trait of BufRead/Read traits for speed of implementation
//so the adaptor patternt is not fully realised 😞

//I could generalise the reader member to be a Box<dyn Read> and abstract away the type of reader
//but opted not to, due to speed of implementation & perf considerations

#[derive(Debug)]
pub struct SpamTolerantReader<T: Read> {
	#[doc(hidden)]
	reader: BufReader<T>,
	aux: Vec<u8>,
	delim: u8,
	//helps achieve fused iterator semantics
	limit_tripped: bool,
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
	IOError(std::io::ErrorKind),
	EOFReached,
	ToleranceExceeded,
}

#[doc(hidden)]
fn find_delimiter_in_buf(buf: &[u8], search_up_to: usize, delim: u8) -> Option<usize> {
	buf.iter().take(search_up_to).position(|&x| x == delim)
}

impl<T: Read> SpamTolerantReader<T> {
	pub fn new(reader: T, delim: u8, tolerance: NonZeroUsize) -> Self {
		SpamTolerantReader {
			reader: BufReader::new(reader),
			aux: Vec::with_capacity(tolerance.into()),
			delim,
			limit_tripped: false,
		}
	}
}

impl<T: Read> SpamTolerantReader<T> {
	pub fn get_next(&mut self) -> Result<&[u8], ErrorKind> {
		macro_rules! get_read_buf {
			() => {
				self
					.reader
					.fill_buf()
					.map_err(|x| ErrorKind::IOError(x.kind()))
			};
		}
		if self.limit_tripped {
			return Err(ErrorKind::ToleranceExceeded);
		}
		let mut buf = get_read_buf!()?;
		if buf.is_empty() {
			return Err(ErrorKind::EOFReached);
		}
		self.aux.clear();
		if let Some(n) = find_delimiter_in_buf(buf, self.aux.capacity() + 1, self.delim) {
			self.aux.extend_from_slice(&buf[..n]);
			self.reader.consume(n + 1);
			Ok(&self.aux)
		} else if buf.len() > self.aux.capacity() {
			self.limit_tripped = true;
			Err(ErrorKind::ToleranceExceeded)
		} else {
			//no delimiter found and buf is smaller than aux
			loop {
				let buf_len = buf.len();
				self.aux.extend_from_slice(buf);
				self.reader.consume(buf_len);
				buf = get_read_buf!()?;
				if buf.is_empty() {
					return Ok(&self.aux);
				}
				if let Some(n) =
					find_delimiter_in_buf(buf, self.aux.capacity() - self.aux.len() + 1, self.delim)
				{
					self.aux.extend_from_slice(&buf[..n]);
					self.reader.consume(n + 1);
					return Ok(&self.aux);
				}
				if buf.len() > self.aux.capacity() - self.aux.len() {
					self.limit_tripped = true;
					dbg!("YOLO");
					return Err(ErrorKind::ToleranceExceeded);
				}
			}
		}
	}
}
