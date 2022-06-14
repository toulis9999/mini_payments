use std::io::BufRead;
use std::num::NonZeroUsize;

//nightly feature 😞
//use std::str::pattern::Pattern;

//this does not adhere to Iterator trait of BufRead/Read traits for speed of implementation
//so the adaptor patternt is not fully realised 😞

#[derive(Debug)]
pub struct SpamTolerantReader<T: BufRead> {
	#[doc(hidden)]
	reader: T,
	aux: Vec<u8>,
	delimiter: u8,
}

#[derive(Debug)]
pub enum ErrorKind {
	IOError(std::io::Error),
	ToleranceExceeded,
}

impl<T: BufRead> SpamTolerantReader<T> {
	pub fn new(reader: T, delimiter: u8, tolerance: NonZeroUsize) -> Self {
		SpamTolerantReader {
			reader,
			aux: vec![0; tolerance.into()],
			delimiter,
		}
	}

	pub fn get_next(&mut self) -> Result<&[u8], ErrorKind> {
		let aux_idx = {
			let buf = self.reader.fill_buf().map_err(ErrorKind::IOError)?;
			let bytes_to_copy = std::cmp::min(self.aux.len(), buf.len());
			self.aux[..bytes_to_copy].copy_from_slice(&buf[..bytes_to_copy]);
			bytes_to_copy
		};
		if let Some(n) = self.aux[..aux_idx]
			.iter()
			.position(|&x| x == self.delimiter)
		{
			self.reader.consume(self.aux[..n].len() + 1);
			Ok(&self.aux[..n])
		} else if aux_idx < self.aux.len() {
			self.reader.consume(self.aux[..aux_idx].len());
			Ok(&self.aux[..aux_idx])
		} else {
			Err(ErrorKind::ToleranceExceeded)
		}
	}
}
