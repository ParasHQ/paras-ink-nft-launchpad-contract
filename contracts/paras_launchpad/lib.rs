#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod paras_launchpad {
    use ink::codegen::{EmitEvent, Env};
    use openbrush::{
        contracts::{
            ownable::*,
            psp34::extensions::{enumerable::*, metadata::*},
        },
        modifiers,
        traits::{Storage, String},
    };

    use ink::prelude::vec::Vec;

    use psp34_extension_pkg::{
        impls::launchpad::{types::Percentage, *},
        traits::launchpad::*,
        traits::psp34_traits::*,
    };

    // Shiden34Contract contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct ParasLaunchpadContract {
        #[storage_field]
        psp34: psp34::Data<enumerable::Balances>,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        metadata: metadata::Data,
        #[storage_field]
        launchpad: types::Data,
    }

    impl PSP34 for ParasLaunchpadContract {}
    impl PSP34Enumerable for ParasLaunchpadContract {}
    impl PSP34Metadata for ParasLaunchpadContract {}
    impl Ownable for ParasLaunchpadContract {}

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: Id,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        id: Option<Id>,
        approved: bool,
    }

    impl ParasLaunchpadContract {
        #[ink(constructor)]
        pub fn new(
            name: String,
            symbol: String,
            base_uri: String,
            max_supply: u64,
            prepresale_price_per_mint: Balance,
            presale_price_per_mint: Balance,
            price_per_mint: Balance,
            prepresale_start_at: u64,
            presale_start_at: u64,
            public_sale_start_at: u64,
            public_sale_end_at: Option<u64>,
            launchpad_fee: Percentage,
            project_treasury: AccountId,
            launchpad_treasury: AccountId,
        ) -> Self {
            let mut instance = Self::default();

            instance._init_with_owner(instance.env().caller());
            let collection_id = instance.collection_id();
            instance._set_attribute(collection_id.clone(), String::from("name"), name);
            instance._set_attribute(collection_id.clone(), String::from("symbol"), symbol);
            instance._set_attribute(collection_id, String::from("baseUri"), base_uri);

            instance.launchpad.max_supply = max_supply;

            // public
            instance.launchpad.price_per_mint = price_per_mint;
            instance.launchpad.prepresale_price_per_mint = prepresale_price_per_mint;
            instance.launchpad.presale_price_per_mint = presale_price_per_mint;

            instance.launchpad.max_amount = 10;
            instance.launchpad.token_set = (1..max_supply + 1).map(u64::from).collect::<Vec<u64>>();
            instance.launchpad.pseudo_random_salt = 0;
            instance.launchpad.project_treasury = Some(project_treasury);
            instance.launchpad.prepresale_start_at = prepresale_start_at;
            instance.launchpad.presale_start_at = presale_start_at;
            instance.launchpad.public_sale_start_at = public_sale_start_at;
            instance.launchpad.public_sale_end_at = public_sale_end_at;

            // validation
            assert!(launchpad_fee < 100);

            instance.launchpad.total_sales = 0;
            instance.launchpad.withdrawn_sales_launchpad = 0;
            instance.launchpad.withdrawn_sales_project = 0;
            instance.launchpad.launchpad_fee = launchpad_fee;
            instance.launchpad.launchpad_treasury = Some(launchpad_treasury);

            instance
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_code(&mut self, code_hash: [u8; 32]) -> Result<(), PSP34Error> {
            // TO DO: test set_code
            ink::env::set_code_hash(&code_hash).unwrap_or_else(|err| {
                panic!(
                    "Failed to `set_code_hash` to {:?} due to {:?}",
                    code_hash, err
                )
            });
            ink::env::debug_println!("Switched code hash to {:?}.", code_hash);
            Ok(())
        }
    }

    // Override event emission methods
    impl psp34::Internal for ParasLaunchpadContract {
        fn _emit_transfer_event(&self, from: Option<AccountId>, to: Option<AccountId>, id: Id) {
            self.env().emit_event(Transfer { from, to, id });
        }

        fn _emit_approval_event(
            &self,
            from: AccountId,
            to: AccountId,
            id: Option<Id>,
            approved: bool,
        ) {
            self.env().emit_event(Approval {
                from,
                to,
                id,
                approved,
            });
        }
    }

    impl Launchpad for ParasLaunchpadContract {}
    impl Psp34Traits for ParasLaunchpadContract {}

    // ------------------- T E S T -----------------------------------------------------
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{pay_with_call, test};
        use ink::prelude::string::String as PreludeString;
        use psp34_extension_pkg::impls::launchpad::{
            launchpad::Internal,
            types::{MintingStatus, Shiden34Error},
        };
        const PRICE: Balance = 100_000_000_000_000_000;
        const PREPRESALE_PRICE: Balance = 10_000_000_000_000_000;
        const PRESALE_PRICE: Balance = 20_000_000_000_000_000;
        const BASE_URI: &str = "ipfs://myIpfsUri/";
        const MAX_SUPPLY: u64 = 10;

        const PUBLIC_SALE_END_AT: u64 = 1682899200000;
        const ONE_MONTH_IN_MILLIS: u64 = 2592000000;

        #[ink::test]
        fn init_works() {
            let sh34 = init();
            let collection_id = sh34.collection_id();
            assert_eq!(
                sh34.get_attribute(collection_id.clone(), String::from("name")),
                Some(String::from("Shiden34"))
            );
            assert_eq!(
                sh34.get_attribute(collection_id.clone(), String::from("symbol")),
                Some(String::from("SH34"))
            );
            assert_eq!(
                sh34.get_attribute(collection_id, String::from("baseUri")),
                Some(String::from(BASE_URI))
            );
            assert_eq!(sh34.max_supply(), MAX_SUPPLY);
            assert_eq!(sh34.price(), PRICE);
        }

        fn init() -> ParasLaunchpadContract {
            let accounts = default_accounts();
            ParasLaunchpadContract::new(
                String::from("Shiden34"), // name: String,
                String::from("SH34"),     // symbol: String,
                String::from(BASE_URI),   // base_uri: String,
                MAX_SUPPLY,               // max_supply: u64,
                PREPRESALE_PRICE,         // prepresale_price_per_mint: Balance,
                PRESALE_PRICE,            // presale_price_per_mint: Balance
                PRICE,                    // price_per_mint: Balance,
                0,                        // prepresale_start_at: u64,
                0,                        // presale_start_at: u64,
                0,                        // public_sale_start_at: u64,
                PUBLIC_SALE_END_AT,       // public_sale_end_at: u64,
                10,
                accounts.charlie, // project_treasury: AccountId,
                accounts.django,
            )
        }

        #[ink::test]
        fn mint_single_works() {
            let mut sh34 = init();
            let accounts = default_accounts();
            assert_eq!(sh34.owner(), accounts.alice);

            set_sender(accounts.alice);
            assert!(sh34.set_minting_status(Some(3)).is_ok());

            set_sender(accounts.bob);

            assert_eq!(sh34.total_supply(), 0);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(PRICE);
            assert!(sh34.mint_next().is_ok());
            assert_eq!(sh34.total_supply(), 1);

            let bob_token_id = sh34.owners_token_by_index(accounts.bob, 0);
            assert_eq!(
                sh34.owner_of(bob_token_id.ok().unwrap()),
                Some(accounts.bob)
            );
            assert_eq!(sh34.balance_of(accounts.bob), 1);

            assert_eq!(1, ink::env::test::recorded_events().count());
        }

        #[ink::test]
        fn set_minting_status_works() {
            let mut sh34 = init();
            let accounts = default_accounts();
            set_sender(accounts.alice);
            assert!(sh34.set_minting_status(Some(1)).is_ok()); // prepresale
        }

        #[ink::test]
        fn set_minting_status_auth_error() {
            let mut sh34 = init();
            let accounts = default_accounts();
            set_sender(accounts.bob);
            assert!(sh34.set_minting_status(Some(1)).is_err()); // prepresale
        }

        #[ink::test]
        fn add_to_presale_and_prepresale_works() {
            let mut sh34 = init();
            let accounts = default_accounts();
            set_sender(accounts.alice);
            assert!(sh34.add_account_to_prepresale(accounts.bob, 1).is_ok());
            assert_eq!(
                sh34.get_account_prepresale_minting_amount(accounts.bob)
                    .unwrap(),
                1
            );

            assert!(sh34.add_account_to_presale(accounts.bob, 1).is_ok());
            assert_eq!(
                sh34.get_account_presale_minting_amount(accounts.bob)
                    .unwrap(),
                1
            );
        }

        #[ink::test]
        fn add_to_presale_and_prepresale_auth_error() {
            let mut sh34 = init();
            let accounts = default_accounts();
            set_sender(accounts.bob);
            assert!(sh34.add_account_to_prepresale(accounts.bob, 1).is_err());
            assert!(sh34.add_account_to_presale(accounts.bob, 1).is_err());
        }

        #[ink::test]
        fn mint_prepresale_works() {
            let mut sh34 = init();
            let accounts = default_accounts();
            assert_eq!(sh34.owner(), accounts.alice);

            set_sender(accounts.alice);
            assert!(sh34.set_minting_status(Some(1)).is_ok()); // prepresale
            assert!(sh34.add_account_to_prepresale(accounts.bob, 1).is_ok());

            set_sender(accounts.bob);

            assert_eq!(sh34.total_supply(), 0);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(PREPRESALE_PRICE);
            assert!(sh34.mint_next().is_ok());
            assert_eq!(sh34.total_supply(), 1);
            assert_eq!(
                sh34.get_account_prepresale_minting_amount(accounts.bob)
                    .unwrap(),
                0
            );

            let bob_token_id = sh34.owners_token_by_index(accounts.bob, 0);
            assert_eq!(
                sh34.owner_of(bob_token_id.ok().unwrap()),
                Some(accounts.bob)
            );
            assert_eq!(sh34.balance_of(accounts.bob), 1);

            assert_eq!(1, ink::env::test::recorded_events().count());
        }

        #[ink::test]
        fn mint_presale_works() {
            let mut sh34 = init();
            let accounts = default_accounts();
            assert_eq!(sh34.owner(), accounts.alice);

            set_sender(accounts.alice);
            assert!(sh34.set_minting_status(Some(2)).is_ok()); // preesale
            assert!(sh34.add_account_to_presale(accounts.bob, 1).is_ok());

            set_sender(accounts.bob);

            assert_eq!(sh34.total_supply(), 0);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(PRESALE_PRICE);
            assert!(sh34.mint_next().is_ok());
            assert_eq!(sh34.total_supply(), 1);
            assert_eq!(
                sh34.get_account_presale_minting_amount(accounts.bob)
                    .unwrap(),
                0
            );

            let bob_token_id = sh34.owners_token_by_index(accounts.bob, 0);
            assert_eq!(
                sh34.owner_of(bob_token_id.ok().unwrap()),
                Some(accounts.bob)
            );
            assert_eq!(sh34.balance_of(accounts.bob), 1);

            assert_eq!(1, ink::env::test::recorded_events().count());
        }

        #[ink::test]
        fn withdraw_launchpad_works() {
            let mut sh34 = init();
            let accounts = default_accounts();
            assert_eq!(sh34.owner(), accounts.alice);

            set_sender(accounts.alice);
            assert!(sh34.set_minting_status(Some(2)).is_ok()); // presale
            assert!(sh34.add_account_to_presale(accounts.bob, 1).is_ok());

            set_sender(accounts.bob);

            test::set_account_balance::<ink::env::DefaultEnvironment>(accounts.bob, PRESALE_PRICE);
            assert!(pay_with_call!(sh34.mint_next(), PRESALE_PRICE).is_ok());

            set_sender(accounts.alice);
            assert!(sh34.set_minting_status(Some(4)).is_ok());

            test::set_account_balance::<ink::env::DefaultEnvironment>(accounts.django, 0);

            // after mint  withdraw
            test::set_block_timestamp::<ink::env::DefaultEnvironment>(PUBLIC_SALE_END_AT + 1);

            set_sender(accounts.django);
            assert!(sh34.withdraw_launchpad().is_ok());

            assert_eq!(
                test::get_account_balance::<ink::env::DefaultEnvironment>(accounts.django)
                    .ok()
                    .unwrap(),
                (PRESALE_PRICE * 10) / 100
            );

            assert_eq!(1, ink::env::test::recorded_events().count());
        }

        #[ink::test]
        fn withdraw_project_works() {
            let mut sh34 = init();
            let accounts = default_accounts();
            assert_eq!(sh34.owner(), accounts.alice);

            set_sender(accounts.alice);
            assert!(sh34.set_minting_status(Some(2)).is_ok()); // presale
            assert!(sh34.add_account_to_presale(accounts.bob, 1).is_ok());

            set_sender(accounts.bob);

            test::set_account_balance::<ink::env::DefaultEnvironment>(accounts.bob, PRESALE_PRICE);
            assert!(pay_with_call!(sh34.mint_next(), PRESALE_PRICE).is_ok());

            set_sender(accounts.alice);
            assert!(sh34.set_minting_status(Some(4)).is_ok());

            test::set_account_balance::<ink::env::DefaultEnvironment>(accounts.charlie, 0);

            // after mint withdraw
            test::set_block_timestamp::<ink::env::DefaultEnvironment>(PUBLIC_SALE_END_AT + 1);

            set_sender(accounts.charlie);
            assert!(sh34.withdraw_project().is_ok());

            assert_eq!(
                test::get_account_balance::<ink::env::DefaultEnvironment>(accounts.charlie)
                    .ok()
                    .unwrap(),
                (PRESALE_PRICE * 90) / 100
            );

            assert_eq!(1, ink::env::test::recorded_events().count());
        }

        #[ink::test]
        fn mint_multiple_works() {
            let mut sh34 = init();
            let accounts = default_accounts();

            set_sender(accounts.alice);
            let num_of_mints: u64 = 5;
            // Set max limit to 'num_of_mints', fails to mint 'num_of_mints + 1'. Caller is contract owner
            assert!(sh34.set_max_mint_amount(num_of_mints).is_ok());
            assert!(sh34.set_minting_status(Some(3)).is_ok());

            assert_eq!(
                sh34.mint(accounts.bob, num_of_mints + 1),
                Err(PSP34Error::Custom(
                    Shiden34Error::TooManyTokensToMint.as_str()
                ))
            );

            assert_eq!(sh34.total_supply(), 0);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(
                PRICE * num_of_mints as u128,
            );
            assert!(sh34.mint(accounts.bob, num_of_mints).is_ok());
            assert_eq!(sh34.total_supply(), num_of_mints as u128);
            assert_eq!(sh34.balance_of(accounts.bob), 5);
            assert_eq!(5, ink::env::test::recorded_events().count());
        }

        #[ink::test]
        fn mint_above_limit_fails() {
            let mut sh34 = init();
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let num_of_mints: u64 = MAX_SUPPLY + 1;

            assert_eq!(sh34.total_supply(), 0);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(
                PRICE * num_of_mints as u128,
            );
            assert!(sh34.set_max_mint_amount(num_of_mints).is_ok());
            assert_eq!(
                sh34.mint(accounts.bob, num_of_mints),
                Err(PSP34Error::Custom(Shiden34Error::CollectionIsFull.as_str()))
            );
        }

        #[ink::test]
        fn mint_low_value_fails() {
            let mut sh34 = init();
            let accounts = default_accounts();
            set_sender(accounts.alice);
            assert!(sh34.set_minting_status(Some(3)).is_ok());

            set_sender(accounts.bob);
            let num_of_mints = 1;

            assert_eq!(sh34.total_supply(), 0);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(
                PRICE * num_of_mints as u128 - 1,
            );
            assert_eq!(
                sh34.mint(accounts.bob, num_of_mints),
                Err(PSP34Error::Custom(Shiden34Error::BadMintValue.as_str()))
            );
            test::set_value_transferred::<ink::env::DefaultEnvironment>(
                PRICE * num_of_mints as u128 - 1,
            );
            assert_eq!(
                sh34.mint_next(),
                Err(PSP34Error::Custom(Shiden34Error::BadMintValue.as_str()))
            );
            assert_eq!(sh34.total_supply(), 0);
        }

        #[ink::test]
        fn withdrawal_works() {
            let mut sh34 = init();
            let accounts = default_accounts();

            set_sender(accounts.alice);
            assert!(sh34.set_minting_status(Some(3)).is_ok());

            set_balance(accounts.bob, PRICE);
            set_sender(accounts.bob);

            assert!(pay_with_call!(sh34.mint_next(), PRICE).is_ok());
            let expected_contract_balance = PRICE + sh34.env().minimum_balance();
            assert_eq!(sh34.env().balance(), expected_contract_balance);

            // Bob fails to withdraw
            set_sender(accounts.bob);
            assert!(sh34.withdraw_launchpad().is_err());
            assert_eq!(sh34.env().balance(), expected_contract_balance);

            // Django (launchpad treasrury) withdraws. Existential minimum is still set
            set_sender(accounts.django);
            assert!(sh34.withdraw_launchpad().is_ok());
        }

        #[ink::test]
        fn token_uri_works() {
            use crate::paras_launchpad::Id::U64;

            let mut sh34 = init();
            let accounts = default_accounts();
            set_sender(accounts.alice);

            assert!(sh34.set_minting_status(Some(3)).is_ok());
            test::set_value_transferred::<ink::env::DefaultEnvironment>(PRICE);
            let mint_result = sh34.mint_next();
            assert!(mint_result.is_ok());
            // return error if request is for not yet minted token

            let alice_token_id: u64 =
                match sh34.owners_token_by_index(accounts.alice, 0).ok().unwrap() {
                    U64(value) => value,
                    _ => 0,
                };
            assert_eq!(
                sh34.token_uri(alice_token_id),
                PreludeString::from(
                    BASE_URI.to_owned() + format!("{}.json", alice_token_id).as_str()
                )
            );

            // verify token_uri when baseUri is empty
            set_sender(accounts.alice);
            assert!(sh34.set_base_uri(PreludeString::from("")).is_ok());
            assert_eq!(
                sh34.token_uri(alice_token_id),
                PreludeString::from("".to_owned() + format!("{}.json", alice_token_id).as_str())
            );
        }

        #[ink::test]
        fn owner_is_set() {
            let accounts = default_accounts();
            let sh34 = init();
            assert_eq!(sh34.owner(), accounts.alice);
        }

        #[ink::test]
        fn set_base_uri_works() {
            let accounts = default_accounts();
            const NEW_BASE_URI: &str = "new_uri/";
            let mut sh34 = init();

            set_sender(accounts.alice);
            let collection_id = sh34.collection_id();
            assert!(sh34.set_base_uri(NEW_BASE_URI.into()).is_ok());
            assert_eq!(
                sh34.get_attribute(collection_id, String::from("baseUri")),
                Some(String::from(NEW_BASE_URI))
            );

            set_sender(accounts.charlie);
            let collection_id = sh34.collection_id();
            assert!(sh34.set_base_uri(NEW_BASE_URI.into()).is_ok());

            set_sender(accounts.bob);
            assert_eq!(
                sh34.set_base_uri(NEW_BASE_URI.into()),
                Err(PSP34Error::Custom(String::from("Unauthorized")))
            );
        }

        #[ink::test]
        fn check_supply_overflow_ok() {
            let max_supply = u64::MAX - 1;
            let accounts = default_accounts();
            let mut sh34 = ParasLaunchpadContract::new(
                String::from("Shiden34"), // name: String,
                String::from("SH34"),     // symbol: String,
                String::from(BASE_URI),   // base_uri: String,
                max_supply,               // max_supply: u64
                PREPRESALE_PRICE,
                PRESALE_PRICE,
                PRICE, // price_per_mint: Balance,
                0,     // prepresale_start_at: u64,
                0,     // presale_start_at: u64,
                0,     // public_sale_start_at: u64,
                0,     // public_sale_end_at: u64,
                10,
                accounts.charlie, // project_treasury: AccountId,
                accounts.django,  // launchpad_treasury: AccountId,
            );

            // check case when last_token_id.add(mint_amount) if more than u64::MAX
            // assert!(sh34.set_max_mint_amount(u64::MAX).is_ok());
            // assert_eq!(
            //     sh34.check_amount(3),
            //     Err(PSP34Error::Custom(Shiden34Error::CollectionIsFull.as_str()))
            // );

            // // check case when mint_amount is 0
            // assert_eq!(
            //     sh34.check_amount(0),
            //     Err(PSP34Error::Custom(
            //         Shiden34Error::CannotMintZeroTokens.as_str()
            //     ))
            // );
        }

        #[ink::test]
        fn check_value_overflow_ok() {
            let max_supply = u64::MAX;
            let price = u128::MAX as u128;
            let accounts = default_accounts();
            let sh34 = ParasLaunchpadContract::new(
                String::from("Shiden34"), // name: String,
                String::from("SH34"),     // symbol: String,
                String::from(BASE_URI),   // base_uri: String,
                max_supply,               // max_supply: u64,
                PREPRESALE_PRICE,
                PRESALE_PRICE,
                price,           // price_per_mint: Balance,
                0,               // prepresale_start_at: u64,
                0,               // presale_start_at: u64,
                0,               // public_sale_start_at: u64,
                100000000000000, // public_sale_end_at: u64,
                10,
                accounts.charlie, // project_treasury: AccountId,
                accounts.django,
            );
            let transferred_value = u128::MAX;
            let mint_amount = u64::MAX;
            assert_eq!(
                sh34.check_value(transferred_value, mint_amount, &MintingStatus::Public),
                Err(PSP34Error::Custom(Shiden34Error::BadMintValue.as_str()))
            );
        }

        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<Environment>()
        }

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<Environment>(sender);
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(account_id, balance)
        }
    }
}
