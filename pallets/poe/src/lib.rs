#![cfg_attr(not(feature = "std"), no_std)]

// 导入poe模块需要的内容，如存储单元等给外部使用
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// 定义poe模块，放入模块空间
#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		// pallet::constant宏表示该值是一个常量
		#[pallet::constant]
		// 存证最大能接受的长度限制，太长会导致链上的状态爆炸，链上存储的存证是一个hash值且长度固定
		type MaxClaimLength: Get<u32>;
		// 该通用的关联类型，在runtime进行配置接口实现时，会把runtime定义的Event设置在这个类型里
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	// 定义模块所需的结构体
	#[pallet::pallet]
	// 模块会定义自己所需的存储项，因此需要pallet::generate_store宏，它包括生成Store的trait接口
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// 存储
	#[pallet::storage]
	pub type Proofs<T: Config> = StorageMap<
		_,
		// Blake2是一个密码安全的hash算法，用来将存储项存储到底层数据库时对存储位置进行hash计算
		Blake2_128Concat,
		// 新版本里Runtime不能直接使用Vec集合类型，BoundedVec是一个更安全的长度受限的集合类型
		BoundedVec<u8, T::MaxClaimLength>,
		// Value是包含两个元素的tuple, 表示存证属于哪个用户哪个区块
		(T::AccountId, T::BlockNumber),
	>;

	/// 事件
	// 在交易执行过程中进行触发
	#[pallet::event]
	// generate_deposit宏会生成deposit_event方法，方便生成事件
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimTransfered(T::AccountId, Vec<u8>),
	}

	/// error处理
	// 开发时发现新的错误类型再添加到这里
	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimTooLong,
		ClaimNotExist,
		NotClaimOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// 可调用函数
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		/// 创建存证，origin表示交易的发送方；claim表示存证的内容，通常是hash值
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			// 判断是否是签名用户
			let sender = ensure_signed(origin)?;

			// 接着校验存证内容的hash值，包括所需要的最大长度
			// 使用BoundedVec的try_from方法将claim类型转换为BoundedVec类型
			// 长度超限就返回Error
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// 校验Proofs存储项里不包含存证内容hash值的Key, 即这个键还没存储到Proofs存证存储项里
			// 否则说明这个存证已经被别人申请过了
			ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);

			// 若不存在则进行insert
			Proofs::<T>::insert(
				&bounded_claim,
				(sender.clone(), frame_system::Pallet::<T>::block_number()),
			);

			// 触发create事件
			Self::deposit_event(Event::ClaimCreated(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(0)]
		/// 撤销存证
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			// 判断是否是签名用户
			let sender = ensure_signed(origin)?;

			// 接着校验存证内容的hash值，包括所需要的最大长度
			// 使用BoundedVec的try_from方法将claim的Vec类型转换为BoundedVec类型
			// 长度超限就返回Error
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// 只有已存储的存证才能被吊销
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			// 确认发送方sender与存证owner一致，否则返回Error
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// 删除存证
			Proofs::<T>::remove(&bounded_claim);

			// 触发revoke事件
			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(0)]
		/// 转移存证
		pub fn transfer_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
			dest: T::AccountId,
		) -> DispatchResultWithPostInfo {
			// 判断是否是签名用户
			let sender = ensure_signed(origin)?;

			// 接着校验存证内容的hash值，包括所需要的最大长度
			// 使用BoundedVec的try_from方法将claim类型转换为BoundedVec类型
			// 长度超限就返回Error
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// 只有已存储到链上的存证才能被转移
			let (owner, _block_number) =
				Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			// 确认发送方sender与存证owner一致，否则返回Error
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// 转移存证
			Proofs::<T>::insert(&bounded_claim, (dest, frame_system::Pallet::<T>::block_number()));

			// 触发revoke事件
			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}
	}
}
