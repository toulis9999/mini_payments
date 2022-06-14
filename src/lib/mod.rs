mod fixed_decimal;
mod processor;
mod spam_tolerant_reader;
mod transaction;

pub use fixed_decimal::ErrorKind as FixedDecimalError;
pub use fixed_decimal::FixedDecimal;
pub use fixed_decimal::MAX as FixedDecimalMAX;
pub use fixed_decimal::MAX_DISP_LEN as FixedDecimalMAXDISPLEN;

pub use transaction::ErrorKind as TransactionError;
pub use transaction::{PaymentsTransaction, TransactionPayload};

pub use processor::{PaymentsProcessor, ProcessTransactionError};

pub use spam_tolerant_reader::ErrorKind as SpamReaderError;
pub use spam_tolerant_reader::SpamTolerantReader;
