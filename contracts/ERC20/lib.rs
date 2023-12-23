#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod ERC20 {
    use ink::primitives::AccountId as OtherAccountId;
    use ink::storage::Mapping as StorageHashMap;
    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Erc20 {
        id: u64,
        balance_of: StorageHashMap<OtherAccountId, Balance>,
        total_supply: Balance,
    }

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(id: u64) -> Self {
            Self {
                id: id,
                balance_of: StorageHashMap::default(),
                total_supply: 0,
            }
        }

        //add the tokens in the supply
        #[ink(message, payable)]
        pub fn add(&mut self) {
            let caller = self.env().caller();
            let caller_balance = self.balance_of.get(caller).unwrap();
            let val = self.env().transferred_value();
            self.total_supply = val + self.total_supply;
            self.balance_of.insert(caller, &(caller_balance + val));
        }

        //get the balance of an address.
        #[ink(message)]
        pub fn get_balance_of(&self, account: AccountId) -> Balance {
            self.balance_of.get(&account).unwrap()
        }
    }

    // Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    // module and test functions are marked with a `#[test]` attribute.
    // The below code is technically just normal Rust code.
    // #[cfg(test)]
    // mod tests {
    //     /// Imports all the definitions from the outer scope so we can use them here.
    //     use super::*;

    //     /// We test if the default constructor does its job.
    //     #[ink::test]
    //     fn default_works() {
    //         let ERC20 = Erc20::default();
    //         assert_eq!(ERC20.get(), false);
    //     }

    //     /// We test a simple use case of our contract.
    //     #[ink::test]
    //     fn it_works() {
    //         let mut ERC20 = Erc20::new(false);
    //         assert_eq!(ERC20.get(), false);
    //         ERC20.flip();
    //         assert_eq!(ERC20.get(), true);
    //     }
    // }

    // This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    //
    // When running these you need to make sure that you:
    // - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    // - Are running a Substrate node which contains `pallet-contracts` in the background
    // #[cfg(all(test, feature = "e2e-tests"))]
    // mod e2e_tests {
    //     /// Imports all the definitions from the outer scope so we can use them here.
    //     use super::*;

    //     /// A helper function used for calling contract messages.
    //     use ink_e2e::build_message;

    //     /// The End-to-End test `Result` type.
    //     type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    //     /// We test that we can upload and instantiate the contract using its default constructor.
    //     #[ink_e2e::test]
    //     async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    //         // Given
    //         let constructor = Erc20Ref::default();

    //         // When
    //         let contract_account_id = client
    //             .instantiate("ERC20", &ink_e2e::alice(), constructor, 0, None)
    //             .await
    //             .expect("instantiate failed")
    //             .account_id;

    //         // Then
    //         let get =
    //             build_message::<Erc20Ref>(contract_account_id.clone()).call(|ERC20| ERC20.get());
    //         let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
    //         assert!(matches!(get_result.return_value(), false));

    //         Ok(())
    //     }

    //     /// We test that we can read and write a value from the on-chain contract contract.
    //     #[ink_e2e::test]
    //     async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    //         // Given
    //         let constructor = Erc20Ref::new(false);
    //         let contract_account_id = client
    //             .instantiate("ERC20", &ink_e2e::bob(), constructor, 0, None)
    //             .await
    //             .expect("instantiate failed")
    //             .account_id;

    //         let get =
    //             build_message::<Erc20Ref>(contract_account_id.clone()).call(|ERC20| ERC20.get());
    //         let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
    //         assert!(matches!(get_result.return_value(), false));

    //         // When
    //         let flip =
    //             build_message::<Erc20Ref>(contract_account_id.clone()).call(|ERC20| ERC20.flip());
    //         let _flip_result = client
    //             .call(&ink_e2e::bob(), flip, 0, None)
    //             .await
    //             .expect("flip failed");

    //         // Then
    //         let get =
    //             build_message::<Erc20Ref>(contract_account_id.clone()).call(|ERC20| ERC20.get());
    //         let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
    //         assert!(matches!(get_result.return_value(), true));

    //         Ok(())
    //     }
    // }
}
