#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		// 存证能支持的最大长度限制
		type MaxClaimLength: Get<u32>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Proofs<T: Config> = StorageMap<
		_,
		// 密码安全的hash算法
		Blake2_128Concat,
		// BoundedVec是一个更安全的长度受限的集合类型
		BoundedVec<u8, T::MaxClaimLength>,
		// tuple, 表示存证属于哪个用户哪个区块
		(T::AccountId, T::BlockNumber),
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimTransfered(T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimTooLong,
		ClaimNotExist,
		NotClaimOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		/// 创建存证
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			// 判断是否是签名用户
			let sender = ensure_signed(origin)?;

			// 校验存证内容的hash值，包括所需要的最大长度
			// BoundedVec的try_from方法将claim类型转换为BoundedVec类型
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// 校验存证不存在
			ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);

			// 存储存证
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
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// 只有已存储的存证才能被吊销
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			// 确认发送方sender与存证owner一致
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
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// 只有已存储到链上的存证才能被转移
			let (owner, _block_number) =
				Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;

			// 确认发送方sender与存证owner一致
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// 转移存证
			Proofs::<T>::insert(&bounded_claim, (dest, frame_system::Pallet::<T>::block_number()));

			// 触发transfer事件
			Self::deposit_event(Event::ClaimTransfered(sender, claim));

			Ok(().into())
		}
	}
}
