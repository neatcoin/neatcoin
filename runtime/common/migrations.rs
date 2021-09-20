use frame_support::{traits::OnRuntimeUpgrade, weights::constants::RocksDbWeight};
use crate::{Runtime, TechnicalMembership, TechnicalCommittee, Tips, Council, AllPalletsWithSystem};

/// Migrate from `PalletVersion` to the new `StorageVersion`
pub struct MigratePalletVersionToStorageVersion;

impl OnRuntimeUpgrade for MigratePalletVersionToStorageVersion {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		frame_support::migrations::migrate_from_pallet_version_to_storage_version::<
			AllPalletsWithSystem,
		>(&RocksDbWeight::get())
	}
}

const COUNCIL_OLD_PREFIX: &str = "Instance1Collective";
/// Migrate from `Instance1Collective` to the new pallet prefix `Council`
pub struct CouncilStoragePrefixMigration;

impl OnRuntimeUpgrade for CouncilStoragePrefixMigration {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		pallet_collective::migrations::v4::migrate::<Runtime, Council, _>(COUNCIL_OLD_PREFIX)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		pallet_collective::migrations::v4::pre_migrate::<Council, _>(COUNCIL_OLD_PREFIX);
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		pallet_collective::migrations::v4::post_migrate::<Council, _>(COUNCIL_OLD_PREFIX);
		Ok(())
	}
}

const TECHNICAL_COMMITTEE_OLD_PREFIX: &str = "Instance2Collective";
/// Migrate from 'Instance2Collective' to the new pallet prefix `TechnicalCommittee`
pub struct TechnicalCommitteeStoragePrefixMigration;

impl OnRuntimeUpgrade for TechnicalCommitteeStoragePrefixMigration {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		pallet_collective::migrations::v4::migrate::<Runtime, TechnicalCommittee, _>(
			TECHNICAL_COMMITTEE_OLD_PREFIX,
		)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		pallet_collective::migrations::v4::pre_migrate::<TechnicalCommittee, _>(
			TECHNICAL_COMMITTEE_OLD_PREFIX,
		);
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		pallet_collective::migrations::v4::post_migrate::<TechnicalCommittee, _>(
			TECHNICAL_COMMITTEE_OLD_PREFIX,
		);
		Ok(())
	}
}

const TECHNICAL_MEMBERSHIP_OLD_PREFIX: &str = "Instance1Membership";
/// Migrate from `Instance1Membership` to the new pallet prefix `TechnicalMembership`
pub struct TechnicalMembershipStoragePrefixMigration;

impl OnRuntimeUpgrade for TechnicalMembershipStoragePrefixMigration {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		use frame_support::traits::PalletInfo;
		let name = <Runtime as frame_system::Config>::PalletInfo::name::<TechnicalMembership>()
			.expect("TechnialMembership is part of runtime, so it has a name; qed");
		pallet_membership::migrations::v4::migrate::<Runtime, TechnicalMembership, _>(
			TECHNICAL_MEMBERSHIP_OLD_PREFIX,
			name,
		)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::PalletInfo;
		let name = <Runtime as frame_system::Config>::PalletInfo::name::<TechnicalMembership>()
			.expect("TechnicalMembership is part of runtime, so it has a name; qed");
		pallet_membership::migrations::v4::pre_migrate::<TechnicalMembership, _>(
			TECHNICAL_MEMBERSHIP_OLD_PREFIX,
			name,
		);
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::PalletInfo;
		let name = <Runtime as frame_system::Config>::PalletInfo::name::<TechnicalMembership>()
			.expect("TechnicalMembership is part of runtime, so it has a name; qed");
		pallet_membership::migrations::v4::post_migrate::<TechnicalMembership, _>(
			TECHNICAL_MEMBERSHIP_OLD_PREFIX,
			name,
		);
		Ok(())
	}
}

const TIPS_OLD_PREFIX: &str = "Treasury";
/// Migrate pallet-tips from `Treasury` to the new pallet prefix `Tips`
pub struct MigrateTipsPalletPrefix;

impl OnRuntimeUpgrade for MigrateTipsPalletPrefix {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		pallet_tips::migrations::v4::migrate::<Runtime, Tips, _>(TIPS_OLD_PREFIX)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		pallet_tips::migrations::v4::pre_migrate::<Runtime, Tips, _>(TIPS_OLD_PREFIX);
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		pallet_tips::migrations::v4::post_migrate::<Runtime, Tips, _>(TIPS_OLD_PREFIX);
		Ok(())
	}
}
