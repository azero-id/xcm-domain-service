#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod who_am_i {
    use ink::storage::Mapping;

    #[ink(event)]
    pub struct UserWalkedIn {
        id: u128,
        user: AccountId,
    }

    #[ink(storage)]
    pub struct WhoAmI {
        counter: u128,
        visitors: Mapping<u128, AccountId>,
    }

    impl WhoAmI {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                counter: 0,
                visitors: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn counter(&self) -> u128 {
            self.counter
        }

        #[ink(message)]
        pub fn last_visitor(&self) -> Option<AccountId> {
            self.visitors.get(self.counter)
        }

        #[ink(message)]
        pub fn who_am_i(&self, id: u128) -> Option<AccountId> {
            self.visitors.get(id)
        }

        #[ink(message)]
        pub fn walk_in(&mut self) -> (u128, AccountId) {
            let caller = self.env().caller();

            self.counter += 1;
            self.visitors.insert(self.counter, &caller);

            self.env().emit_event(UserWalkedIn {
                id: self.counter,
                user: caller,
            });

            (self.counter, caller)
        }
    }
}
