use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

/// 创建Kitty成功, 且质押成功、创建Kitty后存储该账号拥有的此Kitty成功
#[test]
fn create_kitty_works() {
	new_test_ext().execute_with(|| {
		// 用账户1创建一个kitty,kitty编号为0
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));

		// 查询kitty数量，预期为1个
		assert_eq!(NextKittyId::<Test>::get(), 1);

		// 根据kitty编号0查所属账户为1
		assert_eq!(KittyOwner::<Test>::try_get(0), Ok(1));
	})
}

/// 创建Kitty失败 - KittyId无效，超过了最大值,同时也无法存储
#[test]
fn create_kitty_failed() {
	new_test_ext().execute_with(|| {
		// 超过KittyId数支持的最大值，便会报错
		NextKittyId::<Test>::put(u32::MAX);

		assert_noop!(KittiesModule::create(RuntimeOrigin::signed(1)), Error::<Test>::InvalidKittyId);
	})
}

/// 创建Kitty时质押token失败 - 余额数太小不足以质押
#[test]
fn create_kitty_and_reversed_failed() {
	new_test_ext().execute_with(|| {
		// 已知账户3中的余额为999，小于质押要求的数目1000，所以质押会失败
		assert_noop!(KittiesModule::create(RuntimeOrigin::signed(3)), Error::<Test>::TokenNotEnough);
	})
}

/// 繁殖Kitty成功
#[test]
fn breed_kitty_works() {
	new_test_ext().execute_with(|| {
		// 用账户1创建一个kitty,kitty编号为0
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));

		// 用账户1再创建一个kitty,kitty编号经过累加变为1
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));

		// 繁殖一个新kitty, 对应账户1
		assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(1), 0, 1));

		// 查询kitty数量，预期为3个
		assert_eq!(NextKittyId::<Test>::get(), 3);

		// 经过繁殖，预期账户1下有3个kitty
		assert_eq!(KittyAll::<Test>::take(1).len(), 3);
	})
}

/// 繁殖Kitty失败 - KittyId相同
#[test]
fn breed_kitty_failed_same_kitty_id() {
	new_test_ext().execute_with(|| {
		// 用账户1创建一个kitty,kitty编号为0
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));

		// 使用两个相同的kitty来繁殖新kitty,预期将报错
		assert_noop!(KittiesModule::breed(RuntimeOrigin::signed(1), 0, 0), Error::<Test>::SameKittyId);
	})
}

/// 繁殖Kitty失败 - 父母KittyId无效
#[test]
fn breed_kitty_failed_invalid_parent_kitty_id() {
	new_test_ext().execute_with(|| {
		// 用账户1创建一个kitty,kitty编号为0
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));

		// kitty_id_1存在，kitty_id_2不存在，预期将报错
		assert_noop!(KittiesModule::breed(RuntimeOrigin::signed(1), 0, 1), Error::<Test>::InvalidKittyId);
	})
}

/// 繁殖Kitty失败 - KittyId数超限无法再获取
#[test]
fn breed_kitty_failed_get_new_kitty_id() {
	new_test_ext().execute_with(|| {
		// 用账户1创建一个kitty,kitty编号为0
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));

		// 用账户2创建一个kitty,kitty编号经过累加变为1
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(2)));

		// 填充kittyId的最大值
		NextKittyId::<Test>::put(u32::MAX);

		// 超过KittyId数支持的最大值，预期会报错
		assert_noop!(KittiesModule::breed(RuntimeOrigin::signed(1), 0, 1), Error::<Test>::InvalidKittyId);
	})
}

/// 繁殖Kitty时质押token失败 - 余额数太小不足以质押
#[test]
fn breed_kitty_failed_token_not_enough_to_reversed() {
	new_test_ext().execute_with(|| {
		// 用账户1创建一个kitty,kitty编号为0
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));

		// 用账户2创建一个kitty,kitty编号经过累加变为1
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(2)));

		// 已知账户3中的余额为999，小于质押要求的数目1000，所以预期质押会失败
		assert_noop!(KittiesModule::breed(RuntimeOrigin::signed(3), 0, 1), Error::<Test>::TokenNotEnough);
	})
}

/// 转移Kitty成功
#[test]
fn transfer_kitty_works() {
	new_test_ext().execute_with(|| {
		// 用账户1创建一个kitty,kitty编号为0
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));

		// 根据kitty编号0查所属账户1
		assert_eq!(KittyOwner::<Test>::try_get(0), Ok(1));

		// 用账户2创建一个kitty,kitty编号经过累加变为1
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(2)));

		// 转移账户1下的kitty编号为0的kitty给账户2，预期成功
		assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(1), 0, 2));

		// 经过上一步的转移，账户1没有kitty；账户2将有两个kitty，kitty编号分别为0和1
		// 预期kittyId数为2
		assert_eq!(NextKittyId::<Test>::get(), 2);

		// 根据kitty编号0查所属账户为2
		assert_eq!(KittyOwner::<Test>::try_get(0), Ok(2));
	})
}

/// 转移Kitty失败 - 待转移的KittyId不存在
#[test]
fn transfer_kitty_failed_kitty_id_not_exists() {
	new_test_ext().execute_with(|| {
		// 用账户1创建一个kitty,kitty编号为0
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));

		// 将账户1下的kitty编号为1的kitty转移给账户2，由于kitty编号为1的kitty不存在，预期将报错
		assert_noop!(
			KittiesModule::transfer(RuntimeOrigin::signed(1), 1, 2),
			Error::<Test>::InvalidKittyId
		);
	})
}

/// 转移Kitty失败 - 待转移的Kitty不属于自己，无权限转移
#[test]
fn transfer_kitty_failed_not_owner() {
	new_test_ext().execute_with(|| {
		// 用账户1创建一个kitty,kitty编号为0
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));

		// 用账户2创建一个kitty,kitty编号为1
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(2)));

		// 账户1把账户2下的编号为1的kitty转移给自己，由于账户1不是kitty编号为1的kitty的owner，
		// 预期将报错
		assert_noop!(KittiesModule::transfer(RuntimeOrigin::signed(1), 1, 1), Error::<Test>::NotOwner);
	})
}

/// 转移Kitty时质押token失败 - 余额数太小不足以质押
#[test]
fn transfer_kitty_failed_token_not_enough_to_reversed() {
	new_test_ext().execute_with(|| {
		// 用账户1创建一个kitty,kitty编号为0
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(1)));

		// 账户1把自己的编号为0的kitty转移给账户3
		// 由于账户3的待质押token为999，不满足质押条件1000，预期将报错
		assert_noop!(KittiesModule::transfer(RuntimeOrigin::signed(1), 0, 3), Error::<Test>::TokenNotEnough);
	})
}