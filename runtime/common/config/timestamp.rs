parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}
