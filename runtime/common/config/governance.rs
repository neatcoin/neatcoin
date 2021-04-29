// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of Neatcoin.
//
// Copyright (c) 2021 Wei Tang.
//
// Neatcoin is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Neatcoin is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Neatcoin. If not, see <http://www.gnu.org/licenses/>.

use static_assertions::const_assert;
use sp_core::u32_trait::{_1, _2, _4, _5};
use sp_runtime::{Permill, Percent, ModuleId};
use frame_system::{EnsureOneOf, EnsureRoot};
use frame_support::{parameter_types, traits::LockIdentifier};
use crate::{
	Runtime, Call, Event, Balances, CouncilCollectiveInstance, TechnicalCollectiveInstance,
	Treasury, Scheduler, OriginCaller, Origin, Council, TechnicalMembershipInstance,
	TechnicalCommittee, Bounties, ElectionsPhragmen,
	types::{BlockNumber, Balance, AccountId, MoreThanHalfCouncil},
	constants::{time::DAYS, currency::{deposit, DOLLARS, CENTS, MILLICENTS}},
};

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 7 * DAYS;
	pub const VotingPeriod: BlockNumber = 7 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 1 * DAYS;
	pub const MinimumDeposit: Balance = 100 * DOLLARS;
	pub const EnactmentPeriod: BlockNumber = 8 * DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	// One cent: $10,000 / MB
	pub const PreimageByteDeposit: Balance = 10 * MILLICENTS;
	pub const InstantAllowed: bool = false;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	type ExternalOrigin = frame_system::EnsureOneOf<AccountId,
		pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollectiveInstance>,
		frame_system::EnsureRoot<AccountId>,
	>;
	type ExternalMajorityOrigin = frame_system::EnsureOneOf<AccountId,
		pallet_collective::EnsureProportionAtLeast<_4, _5, AccountId, CouncilCollectiveInstance>,
		frame_system::EnsureRoot<AccountId>,
	>;
	type ExternalDefaultOrigin = frame_system::EnsureOneOf<AccountId,
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollectiveInstance>,
		frame_system::EnsureRoot<AccountId>,
	>;
	type FastTrackOrigin = frame_system::EnsureOneOf<AccountId,
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollectiveInstance>,
		frame_system::EnsureRoot<AccountId>,
	>;
	type InstantOrigin = frame_system::EnsureNever<AccountId>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type CancellationOrigin = EnsureOneOf<AccountId,
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollectiveInstance>,
		EnsureRoot<AccountId>,
	>;
	type CancelProposalOrigin = EnsureOneOf<AccountId,
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollectiveInstance>,
		EnsureRoot<AccountId>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollectiveInstance>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollectiveInstance>;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type WeightInfo = ();
	type MaxProposals = MaxProposals;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 3 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

impl pallet_collective::Config<CouncilCollectiveInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = ();
}

parameter_types! {
	pub const CandidacyBond: Balance = 1 * DOLLARS;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = deposit(0, 32);
	/// Weekly council elections; scaling up to monthly eventually.
	pub const TermDuration: BlockNumber = 1 * DAYS;
	/// 13 members initially, to be increased to 23 eventually.
	pub const DesiredMembers: u32 = 7;
	pub const DesiredRunnersUp: u32 = 30;
	pub const ElectionsPhragmenModuleId: LockIdentifier = *b"phrelect";
}
// Make sure that there are no more than `MaxMembers` members elected via phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
	type Event = Event;
	type ModuleId = ElectionsPhragmenModuleId;
	type Currency = Balances;
	type ChangeMembers = Council;
	type InitializeMembers = Council;
	type CurrencyToVote = frame_support::traits::U128CurrencyToVote;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type LoserCandidate = Treasury;
	type KickedMember = Treasury;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type WeightInfo = ();
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 3 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

impl pallet_collective::Config<TechnicalCollectiveInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

impl pallet_membership::Config<TechnicalMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = frame_system::EnsureRoot<AccountId>;
	type RemoveOrigin = frame_system::EnsureRoot<AccountId>;
	type SwapOrigin = frame_system::EnsureRoot<AccountId>;
	type ResetOrigin = frame_system::EnsureRoot<AccountId>;
	type PrimeOrigin = frame_system::EnsureRoot<AccountId>;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 20 * DOLLARS;
	pub const SpendPeriod: BlockNumber = 6 * DAYS;
	pub const Burn: Permill = Permill::from_percent(1);
	pub const TreasuryModuleId: ModuleId = ModuleId(*b"py/trsry");

	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * DOLLARS;
	pub const DataDepositPerByte: Balance = 1 * CENTS;
	pub const BountyDepositBase: Balance = 1 * DOLLARS;
	pub const BountyDepositPayoutDelay: BlockNumber = 8 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 16 * DAYS;
	pub const MaximumReasonLength: u32 = 16384;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 10 * DOLLARS;
}

type ApproveOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_4, _5, AccountId, CouncilCollectiveInstance>
>;

impl pallet_treasury::Config for Runtime {
	type ModuleId = TreasuryModuleId;
	type Currency = Balances;
	type ApproveOrigin = ApproveOrigin;
	type RejectOrigin = MoreThanHalfCouncil;
	type Event = Event;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = Bounties;
	type WeightInfo = ();
}

impl pallet_bounties::Config for Runtime {
	type Event = Event;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = ();
}

impl pallet_tips::Config for Runtime {
	type Event = Event;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type Tippers = ElectionsPhragmen;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type WeightInfo = ();
}
