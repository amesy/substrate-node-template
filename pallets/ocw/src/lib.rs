#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_core::crypto::KeyTypeId;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");

pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		MultiSignature, MultiSigner,
	};

	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	// implemented for runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::{ValueQuery, *};
	use frame_system::{
		offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer},
		pallet_prelude::*,
	};
	use sp_runtime::{
		offchain::storage::StorageValueRef,
		traits::{AtLeast32BitUnsigned, Bounded},
	};
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct StudentAttribute([u8; 8], u8);

	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		// pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type StudentId: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaxEncodedLen
			+ Bounded;
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	// 需求，存储课程的学生信息
	#[pallet::storage]
	#[pallet::getter(fn student_info)]
	pub type StudentInfo<T: Config> =
		StorageMap<_, Blake2_128Concat, T::StudentId, StudentAttribute, ValueQuery>;

	// 存储学生编号与Owner
	#[pallet::storage]
	#[pallet::getter(fn student_id_owner)]
	pub type StudentIdOwner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::StudentId, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		StudentInfoCreated(T::AccountId, T::StudentId, StudentAttribute),
		StudentInfoUpdated(T::AccountId, T::StudentId, u8),
		StudentInfoDeleted(T::AccountId, T::StudentId),
	}

	#[pallet::error]
	pub enum Error<T> {
		StuIdLengthTooLong,
		NotOwner,
		NotStudentInfo,
	}

	const ONCHAIN_TX_KEY: &[u8] = b"student";

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			let key_student_id = Self::derived_key(block_number);
			let storage_ref_student_id = StorageValueRef::persistent(&key_student_id);

			let key_student_info = Self::derived_key(block_number);
			let storage_ref_student_info = StorageValueRef::persistent(&key_student_info);

			if let (Ok(Some(value_id)), Ok(Some(value_info))) = (
				storage_ref_student_id.get::<T::StudentId>(),
				storage_ref_student_info.get::<StudentAttribute>(),
			) {
				let _ = Self::send_signed_tx(value_id, value_info.1);
				log::info!(
					"send_signed_tx success: key: {:?}, value: {:?}",
					value_id,
					value_info.1
				);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 新增
		#[pallet::weight(10_000)]
		pub fn create(
			origin: OriginFor<T>,
			student_id: T::StudentId,
			student_name: [u8; 8],
			age: u8,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let student_attribute = StudentAttribute(student_name, age);

			// 存储学生编号与Owner
			StudentIdOwner::<T>::insert(student_id, &who);

			// 存储StudentId和Student_attribute
			StudentInfo::<T>::insert(student_id, student_attribute.clone());

			// 存储学生信息到链下
			Self::storage_student_id(student_id.clone());
			Self::storage_student_info(student_attribute.clone());

			// Emit an event.
			Self::deposit_event(Event::StudentInfoCreated(who, student_id, student_attribute));
			Ok(())
		}

		/// 删除
		#[pallet::weight(10_000)]
		pub fn delete(origin: OriginFor<T>, student_id: T::StudentId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 验证自己是owner后才能执行删除操作
			ensure!(Self::student_id_owner(student_id) == Some(who.clone()), Error::<T>::NotOwner);

			// 删除学生信息
			StudentInfo::<T>::remove(&student_id);

			// 删除学生和Owner的对应关系
			StudentIdOwner::<T>::remove(&student_id);

			// Emit an event.
			Self::deposit_event(Event::StudentInfoDeleted(who, student_id));
			Ok(())
		}

		/// 更新
		#[pallet::weight(10_000)]
		pub fn update(
			origin: OriginFor<T>,
			student_id: T::StudentId,
			new_age: u8,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// 验证自己是owner后才能执行更新操作
			ensure!(Self::student_id_owner(student_id) == Some(who.clone()), Error::<T>::NotOwner);

			// 更新
			let student_attribute = StudentAttribute(StudentInfo::<T>::get(&student_id).0, new_age);
			StudentInfo::<T>::insert(student_id, &student_attribute);

			// Emit an event.
			Self::deposit_event(Event::StudentInfoUpdated(who, student_id, new_age));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn derived_key(block_number: T::BlockNumber) -> Vec<u8> {
			block_number.using_encoded(|encoded_bn| {
				ONCHAIN_TX_KEY
					.clone()
					.into_iter()
					.chain(b"/".into_iter())
					.chain(encoded_bn)
					.copied()
					.collect::<Vec<u8>>()
			})
		}

		// 存储学生信息到链下
		fn storage_student_id(student_id: T::StudentId) {
			let key = Self::derived_key(frame_system::Pallet::<T>::block_number());
			sp_io::offchain_index::set(&key, &student_id.encode());
			log::info!(target:"storage_student_id", "Student id has been stored locally {:?}", student_id);
		}

		fn storage_student_info(student_info: StudentAttribute) {
			let key = Self::derived_key(frame_system::Pallet::<T>::block_number());
			sp_io::offchain_index::set(&key, &student_info.encode());
			log::info!(target:"storage_student_info", "Student info has been stored locally {:?}", student_info);
		}

		// https://github.com/paritytech/substrate/blob/polkadot-v0.9.28/frame/examples/offchain-worker/src/lib.rs
		fn send_signed_tx(student_id: T::StudentId, new_age: u8) -> Result<(), &'static str> {
			let signer = Signer::<T, T::AuthorityId>::all_accounts();
			if !signer.can_sign() {
				return Err(
                    "pallet-ocw No local accounts available. Consider adding one via `author_insertKey` RPC.",
                    )
			}

			let results =
				signer.send_signed_transaction(|_account| Call::update { student_id, new_age });

			for (_, res) in &results {
				match res {
					Ok(()) => log::info!("[{:?}] Submitted data:{:?}", student_id, new_age),
					Err(e) =>
						log::error!("[{:?}] Failed to submit transaction: {:?}", student_id, e),
				}
			}

			Ok(())
		}
	}
}
