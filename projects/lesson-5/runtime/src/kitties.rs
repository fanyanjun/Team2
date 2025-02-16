use support::{decl_module, decl_storage, ensure, StorageValue, StorageMap, dispatch::Result, Parameter, traits::Currency};
use sr_primitives::traits::{SimpleArithmetic, Bounded, Member, Zero};
use codec::{Encode, Decode};
use runtime_io::blake2_128;
use system::ensure_signed;
use rstd::result;

pub trait Trait: balances::Trait {
	type KittyIndex: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
}

#[derive(Encode, Decode)]
pub struct Kitty<T: Trait> {
    pub dna: [u8; 16],
    pub price: T::Balance,
}

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct KittyLinkedItem<T: Trait> {
	pub prev: Option<T::KittyIndex>,
	pub next: Option<T::KittyIndex>,
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		/// Stores all the kitties, key is the kitty id / index
		pub Kitties get(kitty): map T::KittyIndex => Option<Kitty<T>>;
		/// Stores the total number of kitties. i.e. the next kitty index
		pub KittiesCount get(kitties_count): T::KittyIndex;

		pub OwnedKitties get(owned_kitties): map (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Create a new kitty
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?;

			// Generate a random 128bit value
			let dna = Self::random_value(&sender);

			// Create and store kitty
			let kitty = Kitty {
                dna: dna, 
                price: Zero::zero()
            };
			Self::insert_kitty(&sender, kitty_id, kitty);
		}

		/// Breed kitties
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
		}

		// 作业：实现 transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex)
		// 使用 ensure! 来保证只有主人才有权限调用 transfer
		// 使用 OwnedKitties::append 和 OwnedKitties::remove 来修改小猫的主人
        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
            let sender = ensure_signed(origin)?;

            Self::do_transfer(&sender, to, kitty_id)?;
        }

        /// Set price
        pub fn set_price(origin, kitty_id: T::KittyIndex, price: T::Balance) {
            let sender = ensure_signed(origin)?;

            ensure!(<Kitties<T>>::exists(kitty_id), "Not exist kitty_id");

            //check kitty ownership
            let kitty = <OwnedKitties<T>>::read(&sender, Some(kitty_id));
            ensure!(kitty.prev != None, "Not own this kitty");

            let mut kitty = Self::kitty(kitty_id).unwrap();
            kitty.price = price;
            <Kitties<T>>::insert(kitty_id, kitty);
        }

        /// Buy kitty
        pub fn buy_kitty(origin, owner: T::AccountId, kitty_id: T::KittyIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(<Kitties<T>>::exists(kitty_id), "Not exist kitty_id");
            
            //check kitty ownership
            let kitty = <OwnedKitties<T>>::read(&owner, Some(kitty_id));
            ensure!(kitty.prev != None, "Not own this kitty");

            <balances::Module<T> as Currency<_>>::transfer(&sender, &owner, Self::kitty(kitty_id).unwrap().price)?;

            Self::do_transfer(&owner, sender, kitty_id)?;
        }
	}
}

impl<T: Trait> OwnedKitties<T> {
	fn read_head(account: &T::AccountId) -> KittyLinkedItem<T> {
 		Self::read(account, None)
 	}

	fn write_head(account: &T::AccountId, item: KittyLinkedItem<T>) {
 		Self::write(account, None, item);
 	}

	fn read(account: &T::AccountId, key: Option<T::KittyIndex>) -> KittyLinkedItem<T> {
 		<OwnedKitties<T>>::get(&(account.clone(), key)).unwrap_or_else(|| KittyLinkedItem {
 			prev: None,
 			next: None,
 		})
 	}

	fn write(account: &T::AccountId, key: Option<T::KittyIndex>, item: KittyLinkedItem<T>) {
 		<OwnedKitties<T>>::insert(&(account.clone(), key), item);
 	}

	pub fn append(account: &T::AccountId, kitty_id: T::KittyIndex) {
		let head = Self::read_head(account);
		let new_head = KittyLinkedItem {
 			prev: Some(kitty_id),
 			next: head.next,
 		};

		Self::write_head(account, new_head);

		let prev = Self::read(account, head.prev);
		let new_prev = KittyLinkedItem {
 			prev: prev.prev,
 			next: Some(kitty_id),
 		};
		Self::write(account, head.prev, new_prev);

		let item = KittyLinkedItem {
 			prev: head.prev,
 			next: None,
 		};
 		Self::write(account, Some(kitty_id), item);
	}

	pub fn remove(account: &T::AccountId, kitty_id: T::KittyIndex) {
		if let Some(item) = <OwnedKitties<T>>::take(&(account.clone(), Some(kitty_id))) {
			let prev = Self::read(account, item.prev);
			let new_prev = KittyLinkedItem {
 				prev: prev.prev,
 				next: item.next,
 			};

			Self::write(account, item.prev, new_prev);

			let next = Self::read(account, item.next);
 			let new_next = KittyLinkedItem {
 				prev: item.prev,
 				next: next.next,
 			};

  			Self::write(account, item.next, new_next);
		}
	}
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	((selector & dna1) | (!selector & dna2))
}

impl<T: Trait> Module<T> {
	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
		payload.using_encoded(blake2_128)
	}

	fn next_kitty_id() -> result::Result<T::KittyIndex, &'static str> {
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {
			return Err("Kitties count overflow");
		}
		Ok(kitty_id)
	}

	fn insert_owned_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex) {
		// 作业：调用 OwnedKitties::append 完成实现
        <OwnedKitties<T>>::append(owner, kitty_id);
  	}

	fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty<T>) {
		// Create and store kitty
		<Kitties<T>>::insert(kitty_id, kitty);
		<KittiesCount<T>>::put(kitty_id + 1.into());

		Self::insert_owned_kitty(owner, kitty_id);
	}

	fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> Result {
		let kitty1 = Self::kitty(kitty_id_1);
		let kitty2 = Self::kitty(kitty_id_2);

		ensure!(kitty1.is_some(), "Invalid kitty_id_1");
		ensure!(kitty2.is_some(), "Invalid kitty_id_2");
		ensure!(kitty_id_1 != kitty_id_2, "Needs different parent");

		let kitty_id = Self::next_kitty_id()?;

        let kitty1_dna = kitty1.unwrap().dna;
		let kitty2_dna = kitty2.unwrap().dna;

		// Generate a random 128bit value
		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8; 16];

		// Combine parents and selector to create new kitty
		for i in 0..kitty1_dna.len() {
			new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}

		Self::insert_kitty(sender, kitty_id, Kitty{dna: new_dna, price: Zero::zero()});

		Ok(())
	}

    fn do_transfer(from: &T::AccountId, to: T::AccountId, kitty_id: T::KittyIndex) -> Result {
        ensure!(<Kitties<T>>::exists(kitty_id), "Not exist kitty_id");

        let kitty = <OwnedKitties<T>>::read(&from, Some(kitty_id));
        ensure!(kitty.prev != None, "Not own this kitty");

        <OwnedKitties<T>>::remove(&from, kitty_id);
        <OwnedKitties<T>>::append(&to, kitty_id);

        Ok(())
    }
}

