use support::{decl_module, decl_storage, StorageValue, StorageMap};
use codec::{Encode, Decode};
use runtime_io::blake2_128;
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash, Zero};

pub trait Trait: system::Trait {
}


#[derive(Encode, Decode, Default)]
pub struct Kitty(pub [u8; 16]);

pub trait Subkitty{
	 type fatherHash;
     type motherHash;
	 fn Add(&mut self,dna:[u8; 16],x: Hash, y: Hash) ;
}
//子猫 --待完善
impl  Subkitty for Kitty {
     type fatherHash= Hash;
     type motherHash= Hash;
    fn Add(&mut self, dna:[u8; 16],x: Hash, y: Hash) {
        //fatherHash = x;
        //motherHash = y; 
		&Self(dna);
    }
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		/// Stores all the kitties, key is the kitty id / index
		pub Kitties get(kitty): map u32 => Kitty;
		/// Stores the total number of kitties. i.e. the next kitty index
		pub KittiesCount get(kitties_count): u32;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Create a new kitty
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;
			let count = Self::kitties_count();
			if count == u32::max_value() {
				return Err("Kitties count overflow");
			}
			let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
			let dna = payload.using_encoded(blake2_128);
			let kitty = Kitty(dna);
			Kitties::insert(count, kitty);
			KittiesCount::put(count + 1);
		}
		//生新baby
		pub fn reproduction(origin,fatherId: T::Hash, motherId: T::Hash){
			let sender = ensure_signed(origin)?;
			let count = Self::kitties_count();
			if count == u32::max_value() {
				return Err("Kitties count overflow");
			}
			let payload = (<system::Module<T>>::random_seed(), sender, <system::Module<T>>::extrinsic_index(), <system::Module<T>>::block_number());
			let dna = payload.using_encoded(blake2_128);
			//let fatherId_1 = 
			let kitty = Kitty::Add(dna,fatherId,motherId);
			Kitties::insert(count, kitty);
			KittiesCount::put(count + 1);
		}
	}
}
