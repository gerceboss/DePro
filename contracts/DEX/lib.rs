#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod DEX {
    use ink::prelude::vec::Vec;
    use ink::primitives::AccountId as OtherAccountId;
    use ink::storage::Mapping as StorageHashMap;
    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct DEX {
        whale_addresses: StorageHashMap<OtherAccountId, (u64, Balance)>,
        token_addresses: StorageHashMap<u64, OtherAccountId>,
        liquidity_pools: StorageHashMap<(u64, u64), (Balance, Balance)>, //pairs and their balances
        token_total_balance: StorageHashMap<u64, Balance>, //some token's total balance in the contract
    }

    #[ink(event)]
    pub struct TokensSwapped {
        caller: OtherAccountId,
        token_in: u64,
        token_out: u64,
        amount_in: Balance,
        amount_out: Balance,
    }

    #[ink(event)]
    pub struct LiquidityAdded {
        provider: OtherAccountId,
        token_a: u64,
        token_b: u64,
        amount_a: Balance,
        amount_b: Balance,
    }

    /// DEX error
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NoPoolFound,
        ZeroAmount,
        SameTokensNotAllowed,
        LiquidityRatioProblem,
        ReduceTheFunds,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl DEX {
        #[ink(constructor)]
        pub fn new() -> Self {
            let liquidity_pools = StorageHashMap::default();
            let token_total_balance = StorageHashMap::default();
            let token_addresses = StorageHashMap::default();

            let whale_addresses = StorageHashMap::default();
            Self {
                liquidity_pools,
                token_total_balance,
                token_addresses,
                whale_addresses,
            }
        }

        #[ink(message)]
        pub fn get_token_info(&self, id: u64) -> (OtherAccountId, Balance) {
            (
                self.token_addresses.get(&id).unwrap(),
                self.token_total_balance.get(&id).unwrap(),
            )
        }

        #[ink(message)]
        pub fn get_liquidity_pool(&self, id_a: u64, id_b: u64) -> Result<(Balance, Balance)> {
            if id_a == id_b {
                return Err(Error::SameTokensNotAllowed);
            }
            Ok(self.liquidity_pools.get((id_a, id_b)).unwrap())
            //return error when the balances of any are zero or if the ids are same
        }
        // #[ink(message)]
        // pub fn get_all_tokens(&self) {
        //     // let keys_vec: Vec<&u64> = self.token_total_balance;
        // }

        #[ink(message, payable)]
        pub fn add_liquidity(
            &mut self,
            id_a: u64,
            id_b: u64,
            amount_a: Balance,
            amount_b: Balance,
        ) -> Result<()> {
            //also check for the fact if token is not there in the DEX
            //return errors if ids are same or the amounts are 0
            if id_a == id_b {
                return Err(Error::SameTokensNotAllowed);
            } else if amount_a == 0 || amount_b == 0 {
                return Err(Error::ZeroAmount);
            }
            //liquidity ratio must be maintained  0.5:1while adding
            if 2 * amount_a == amount_b {
                return Err(Error::LiquidityRatioProblem);
            }

            //get the tokenAddress
            let account_a = self.token_addresses.get(&id_a).unwrap();
            let account_b = self.token_addresses.get(&id_b).unwrap();
            //get the provider and transfer its tokens to the generated addresses
            let provider = self.env().caller();
            let result_a = self.env().transfer(account_a, amount_a).unwrap(); //but amount a must be token a how to implement that
            let result_b = self.env().transfer(account_b, amount_b).unwrap();
            //also add that the amounts are added by provider in the erc20 contract

            //edit the mapping value first of the token info
            let mut val_a = self.token_total_balance.get(id_a).unwrap();
            val_a += amount_a;
            self.token_total_balance.insert(id_a, &val_a);

            let mut val_b = self.token_total_balance.get(id_b).unwrap();
            val_b += amount_b;
            self.token_total_balance.insert(id_b, &val_b);

            //edit the liquidity pool balances(no particular ratio is needed for now)
            let mut pool = self.liquidity_pools.get((id_a, id_b)).unwrap();
            pool.0 += amount_a;
            pool.1 += amount_b;
            self.liquidity_pools.insert((id_a, id_b), &pool);
            //edit the balances of the ehale providers
            self.whale_addresses.insert(&provider, &(id_a, amount_a));
            self.whale_addresses.insert(&provider, &(id_b, amount_b));
            //emit the event
            self.env().emit_event(LiquidityAdded {
                provider,
                token_a: id_a,
                token_b: id_b,
                amount_a,
                amount_b,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn get_output_amt(&self, token_in: u64, token_out: u64, amount_in: Balance) -> Balance {
            let (balance_in, balance_out) =
                self.liquidity_pools.get((&token_in, &token_out)).unwrap();
            //if after the swap ratio is 0.2:1
            let balance_final = (balance_in * balance_out) / (balance_in + &amount_in);
            if balance_final >= 5 * (balance_in + &amount_in) {
                return balance_final;
            }

            return 0;
        }

        #[ink(message, payable)]
        pub fn swap_tokens(
            &mut self,
            token_in: u64,
            token_out: u64,
            amount_in: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            //get the real amount
            let amount_final = self.get_output_amt(token_in, token_out, amount_in);
            //write the token transfer logic from the caller's account
            if amount_final == 0 {
                return Err(Error::ReduceTheFunds);
            }
            let (.., balance_out) = self.liquidity_pools.get((&token_in, &token_out)).unwrap();
            let amount_out = balance_out - amount_final;
            self.env().transfer(caller, amount_out).unwrap();

            //take the amount in thiss contract and update the balancemapping in the erc20
            //figure out about how to charge fees
            //emit the event
            self.env().emit_event(TokensSwapped {
                caller,
                token_in,
                token_out,
                amount_in,
                amount_out,
            });
            Ok(())
        }
    }

    // /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    // /// module and test functions are marked with a `#[test]` attribute.
    // /// The below code is technically just normal Rust code.
    // #[cfg(test)]
    // mod tests {
    //     /// Imports all the definitions from the outer scope so we can use them here.
    //     use super::*;

    //     /// We test if the default constructor does its job.
    //     #[ink::test]
    //     fn default_works() {
    //         let DEX = DEX::default();
    //         assert_eq!(DEX.get(), false);
    //     }

    //     /// We test a simple use case of our contract.
    //     #[ink::test]
    //     fn it_works() {
    //         let mut DEX = DEX::new(false);
    //         assert_eq!(DEX.get(), false);
    //         DEX.flip();
    //         assert_eq!(DEX.get(), true);
    //     }
    // }

    // /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    // ///
    // /// When running these you need to make sure that you:
    // /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    // /// - Are running a Substrate node which contains `pallet-contracts` in the background
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
    //         let constructor = DeProRef::default();

    //         // When
    //         let contract_account_id = client
    //             .instantiate("DEX", &ink_e2e::alice(), constructor, 0, None)
    //             .await
    //             .expect("instantiate failed")
    //             .account_id;

    //         // Then
    //         let get =
    //             build_message::<DeProRef>(contract_account_id.clone()).call(|DEX| DEX.get());
    //         let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
    //         assert!(matches!(get_result.return_value(), false));

    //         Ok(())
    //     }

    //     /// We test that we can read and write a value from the on-chain contract contract.
    //     #[ink_e2e::test]
    //     async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
    //         // Given
    //         let constructor = DeProRef::new(false);
    //         let contract_account_id = client
    //             .instantiate("DEX", &ink_e2e::bob(), constructor, 0, None)
    //             .await
    //             .expect("instantiate failed")
    //             .account_id;

    //         let get =
    //             build_message::<DeProRef>(contract_account_id.clone()).call(|DEX| DEX.get());
    //         let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
    //         assert!(matches!(get_result.return_value(), false));

    //         // When
    //         let flip =
    //             build_message::<DeProRef>(contract_account_id.clone()).call(|DEX| DEX.flip());
    //         let _flip_result = client
    //             .call(&ink_e2e::bob(), flip, 0, None)
    //             .await
    //             .expect("flip failed");

    //         // Then
    //         let get =
    //             build_message::<DeProRef>(contract_account_id.clone()).call(|DEX| DEX.get());
    //         let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
    //         assert!(matches!(get_result.return_value(), true));

    //         Ok(())
    //     }
    // }
}
