#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::MaxEncodedLen;
	use frame_support::{
		pallet_prelude::{Member, *},
		traits::{Currency, Randomness, ReservableCurrency},
		Parameter,
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Bounded, One},
	};

	// 对每个kitty进行标识
	// type KittyIndex = u32;

	// 为 Storage NextKittyId 设置一个默认值0
	#[pallet::type_value]
	pub fn GetDefaultValue<T: Config>() -> T::KittyIndex {
		0_u32.into()
	}

	// 存放Kitty的特征属性
	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type KittyIndex: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaxEncodedLen
			+ Bounded;
		// Kitty标识最大能接受的长度限制
		type MaxKittyIndexLength: Get<u32>;
		// 创建Kitty需要质押token保留的数量
		type KittyReserve: Get<BalanceOf<Self>>;
		// 用于质押等于资产相关的操作
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex, Kitty),
		KittyBred(T::AccountId, T::KittyIndex, Kitty),
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		NotOwner,
		SameKittyId,
		KittyIdOverflow,
		ExceedMaxKittyOwned,
		TokenNotEnough,
	}

	// 存储KittyId
	// ValueQuery也可以定义成OptionQuery的数据类型，见template/src/lib.rs_89行
	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> =
		StorageValue<_, T::KittyIndex, ValueQuery, GetDefaultValue<T>>;

	// 存储Kitty的特征属性
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;

	// 转移Kitty时存储所属owner
	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;

	// 存储一个账户拥有的所有Kitty
	#[pallet::storage]
	#[pallet::getter(fn kitty_all)]
	pub type KittyAll<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<Kitty, T::MaxKittyIndexLength>,
		ValueQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// 新增
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			let dna = Self::random_value(&who);
			let kitty = Kitty(dna);

			// 质押token
			T::Currency::reserve(&who, T::KittyReserve::get())
				.map_err(|_| Error::<T>::TokenNotEnough)?;

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			NextKittyId::<T>::set(kitty_id + One::one());
			KittyAll::<T>::try_mutate(&who, |kitty_vec| kitty_vec.try_push(kitty.clone()))
				.map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;

			// Emit an event.
			Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));
			Ok(())
		}

		// 繁殖
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 质押token
			T::Currency::reserve(&who, T::KittyReserve::get())
				.map_err(|_| Error::<T>::TokenNotEnough)?;

			// check kitty id
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
			let kitty_1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;

			// get next id
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			// selector for breeding
			let selector = Self::random_value(&who);

			let mut data = [0u8; 16];
			for i in 0..kitty_1.0.len() {
				// 0 choose kitty2, and 1 choose kitty1
				data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
			}
			let new_kitty = Kitty(data);

			<Kitties<T>>::insert(kitty_id, &new_kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			NextKittyId::<T>::set(kitty_id + One::one());
			KittyAll::<T>::try_mutate(&who, |kitty_vec| kitty_vec.try_push(new_kitty.clone()))
				.map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;

			Self::deposit_event(Event::KittyCreated(who, kitty_id, new_kitty));

			Ok(())
		}

		// 转移
		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let exist_kitty = Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;

			// 验证只有自己才能操作自己的owner
			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);

			// 新Owner质押token
			T::Currency::reserve(&new_owner, T::KittyReserve::get())
				.map_err(|_| Error::<T>::TokenNotEnough)?;

			// 删除原拥有者KittyAll存储项需转移的kitty
			KittyAll::<T>::try_mutate(&who, |owned| {
				if let Some(index) = owned.iter().position(|kitty| kitty == &exist_kitty) {
					owned.swap_remove(index);
					return Ok(())
				}
				Err(())
			}).map_err(|_| Error::<T>::NotOwner)?;

			// 解押原来已质押的token
			T::Currency::unreserve(&who, T::KittyReserve::get());

			<KittyOwner<T>>::insert(kitty_id, new_owner.clone());

			// 追加转移的kitty到新拥有者KittyAll存储项中
			KittyAll::<T>::try_mutate(&new_owner, |kitty_vec| kitty_vec.try_push(exist_kitty))
				.map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;

			Self::deposit_event(Event::KittyTransferred(who, new_owner.clone(), kitty_id));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// 取一个随机值
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		// 获取一个新的kitty id
		fn get_next_id() -> Result<T::KittyIndex, DispatchError> {
			let kitty_id = Self::next_kitty_id();
			if kitty_id == T::KittyIndex::max_value() {
				return Err(Error::<T>::KittyIdOverflow.into())
			}
			Ok(kitty_id)
		}

		// 通过kitty id获取kitty
		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}
	}
}
