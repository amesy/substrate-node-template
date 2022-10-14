//! Benchmarking setup for pallet-poe

use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;


// fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
// 	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
// }

benchmarks! {
	create_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = vec![0; d as usize];
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller.clone()), claim.clone())
	// 验证基准测试过程中数据是否正确
	// verify {
	// 	assert_eq!(Proofs::<T>::get(), Some((caller, Pallet::<T>::block_number())));
	// }

	revoke_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = vec![0; d as usize];
		let caller: T::AccountId = whitelisted_caller();
		assert!(Pallet::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone()).is_ok());
	}: _(RawOrigin::Signed(caller.clone()), claim.clone())

	transfer_claim {
		let d in 0 .. T::MaxClaimLength::get();
		let claim = vec![0; d as usize];
		let caller: T::AccountId = whitelisted_caller();
		let target: T::AccountId = account("target", 0, 0);
		assert!(Pallet::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone()).is_ok());
	}: _(RawOrigin::Signed(caller), claim, target)

	impl_benchmark_test_suite!(PoeModule, crate::mock::new_test_ext(), crate::mock::Test);

}
