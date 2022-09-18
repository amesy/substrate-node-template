use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::BoundedVec;

/// 创建存证
#[test]
fn create_claim_works() {
	// 给予测试环境
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		// 断言创建存证
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		// 断言存储项,对应lib.rs里的storage部分
		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
	})
}

/// 创建存证失败 - 存证已存在
#[test]
fn create_claim_failed_when_claim_already_exist() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		// 断言创建一个已存在的存证,预期返回错误
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	})
}

/// 创建存证失败 - 存证过长
#[test]
fn create_claim_failed_when_claim_too_long() {
	new_test_ext().execute_with(|| {
		let claim = vec![1; 513];

		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimTooLong
		);
	})
}

/// 撤销存证
#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		// 断言存证撤销成功
		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));

		// 断言存证在Storage里的值为空
		assert_eq!(Proofs::<Test>::get(&bounded_claim), None);
	})
}

/// 撤销存证失败- 待撤销的存证不存在
#[test]
fn revoke_claim_failed_when_claim_already_revoke() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		let _ = PoeModule::revoke_claim(Origin::signed(1), claim.clone());

		// 在上一步存证已撤销的情况下，再次撤销存证, 预期返回错误
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimNotExist
		);
	})
}

/// 撤销存证失败 - 撤销不属于自己的存证
#[test]
fn revoke_claim_failed_when_claim_not_have_permission() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		// 要撤销的存证不属于自己, 预期返回错误
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	})
}

/// 转移存证
#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		// 断言存证转移成功
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 1));
	})
}

/// 转移存证失败
#[test]
fn transfer_claim_failed_when_claim_already_transfer() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		// 存证没有存储到链上，转移存证将失败
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 1),
			Error::<Test>::ClaimNotExist
		);
	})
}
