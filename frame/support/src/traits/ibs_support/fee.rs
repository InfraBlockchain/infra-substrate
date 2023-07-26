use sp_core::H256;

/// Fee API.
/// Getting fee from fee table
pub trait FeeTableProvider<Balance> {
	fn get_fee_from_fee_table(key: H256) -> Option<Balance>;
}

impl<Balance> FeeTableProvider<Balance> for () {
	fn get_fee_from_fee_table(_key: H256) -> Option<Balance> {
		None
	}
}
