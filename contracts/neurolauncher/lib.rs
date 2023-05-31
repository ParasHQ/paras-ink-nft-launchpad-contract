#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod neurolauncher {
    use ink::codegen::{EmitEvent, Env};
    use openbrush::{
        contracts::{
            ownable::*,
            psp34::extensions::{enumerable::*, metadata::*},
        },
        traits::{Storage, String},
    };

    use ink::prelude::vec::Vec;

    use psp34_extension_pkg::{impls::launchpad::*, traits::launchpad::*, traits::psp34_traits::*};

    // Shiden34Contract contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Neurolauncher {
        #[storage_field]
        psp34: psp34::Data<enumerable::Balances>,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        metadata: metadata::Data,
        #[storage_field]
        launchpad: types::Data,
    }

    impl PSP34 for Neurolauncher {}
    impl PSP34Enumerable for Neurolauncher {}
    impl PSP34Metadata for Neurolauncher {}
    impl Ownable for Neurolauncher {}

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

    impl Neurolauncher
    where
        T: Storage<Data> + Storage<psp34::Data<enumerable::Balances>>,
    {
        #[ink(constructor)]
        pub fn new(
            base_uri: String,
            presale_price_per_mint: Balance, // presale price in astar
            price_per_mint: Balance,         // public prce in astar
            prepresale_start_at: u64,        // set to 1685538000000
            presale_start_at: u64,           // set to 1685538000000
            public_sale_start_at: u64, // 1 day after presale_start_at (in milliseconds) 1685628000000
            project_treasury: AccountId, // WAbEH87bbRgSobQUaXfKnG5Dqb9EYgGUnewoHSVjYaLcLMA
        ) -> Self {
            let mut instance = Self::default();

            let caller = instance.env().caller();
            instance._init_with_owner(caller);
            let collection_id = instance.collection_id();

            // constructor set
            instance._set_attribute(
                collection_id.clone(),
                String::from("name"),
                "Neurolauncher".as_bytes().to_vec(),
            );
            instance._set_attribute(
                collection_id.clone(),
                String::from("symbol"),
                "NRL".as_bytes().to_vec(),
            );

            instance.launchpad.max_supply = 1000;

            instance._set_attribute(collection_id, String::from("baseUri"), base_uri);

            // pricing
            instance.launchpad.prepresale_price_per_mint = None;
            instance.launchpad.presale_price_per_mint = Some(presale_price_per_mint);
            instance.launchpad.price_per_mint = Some(price_per_mint);

            instance.launchpad.max_amount = 3;

            instance.launchpad.project_treasury = Some(project_treasury);
            instance.launchpad.prepresale_start_at = prepresale_start_at;
            instance.launchpad.presale_start_at = presale_start_at;
            instance.launchpad.public_sale_start_at = public_sale_start_at;
            instance.launchpad.public_sale_end_at = None;

            instance.launchpad.launchpad_fee = 0;
            instance.launchpad.launchpad_treasury = None;

            let mint_to_owner = 100;

            // preinstantiate (dont change)
            instance.launchpad.total_sales = 0;
            instance.launchpad.withdrawn_sales_launchpad = 0;
            instance.launchpad.withdrawn_sales_project = 0;
            instance.launchpad.token_set = ((mint_to_owner + 1)
                ..(instance.launchpad.max_supply) + 1)
                .map(u64::from)
                .collect::<Vec<u64>>();
            instance.launchpad.pseudo_random_salt = 0;

            // mint the first 100 to owner wallet
            for i in 1..(mint_to_owner + 1) {
                let _ = instance.psp34._mint_to(project_treasury, Id::U64(i));
            }

            instance
        }
    }

    // Override event emission methods
    impl psp34::Internal for Neurolauncher {
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

    impl Launchpad for Neurolauncher {}
    impl Psp34Traits for Neurolauncher {}

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
        const MAX_SUPPLY: u64 = 1000;
        const SUPPLY_MINTED_TO_OWNER: u128 = 100;
        const NAME: &str = "Neurolauncher";
        const SYMBOL: &str = "NRL";

        const PUBLIC_SALE_END_AT: u64 = 1682899200000;
        const ONE_MONTH_IN_MILLIS: u64 = 2592000000;

        #[ink::test]
        fn init_works() {
            let sh34 = init();
            let collection_id = sh34.collection_id();
            assert_eq!(
                sh34.get_attribute(collection_id.clone(), String::from("name")),
                Some(String::from(NAME))
            );
            assert_eq!(
                sh34.get_attribute(collection_id.clone(), String::from("symbol")),
                Some(String::from(SYMBOL))
            );
            assert_eq!(
                sh34.get_attribute(collection_id, String::from("baseUri")),
                Some(String::from(BASE_URI))
            );
            assert_eq!(sh34.max_supply(), MAX_SUPPLY);
            assert_eq!(sh34.price(), PRICE);
        }

        fn init() -> Neurolauncher {
            let accounts = default_accounts();
            Neurolauncher::new(
                String::from(BASE_URI), // base_uri: String,
                PRESALE_PRICE,          // presale_price_per_mint: Balance
                PRICE,                  // price_per_mint: Balance,
                0,                      // prepresale_start_at: u64,
                0,                      // presale_start_at: u64,
                0,                      // public_sale_start_at: u64,
                accounts.charlie,       // project_treasury: AccountId,
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

            assert_eq!(sh34.total_supply(), SUPPLY_MINTED_TO_OWNER);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(PRICE);
            assert!(sh34.mint_next().is_ok());
            assert_eq!(sh34.total_supply(), SUPPLY_MINTED_TO_OWNER + 1);

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

            assert_eq!(sh34.total_supply(), SUPPLY_MINTED_TO_OWNER);
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

            assert_eq!(sh34.total_supply(), SUPPLY_MINTED_TO_OWNER);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(PRESALE_PRICE);
            assert!(sh34.mint_next().is_ok());
            assert_eq!(sh34.total_supply(), SUPPLY_MINTED_TO_OWNER + 1);
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
            assert!(sh34.withdraw_launchpad().is_err());
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
                (PRESALE_PRICE * 100) / 100
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

            assert_eq!(sh34.total_supply(), SUPPLY_MINTED_TO_OWNER);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(
                PRICE * num_of_mints as u128,
            );
            assert!(sh34.mint(accounts.bob, num_of_mints).is_ok());
            assert_eq!(
                sh34.total_supply(),
                SUPPLY_MINTED_TO_OWNER + num_of_mints as u128
            );
            assert_eq!(sh34.balance_of(accounts.bob), 5);
            assert_eq!(5, ink::env::test::recorded_events().count());
        }

        #[ink::test]
        fn mint_above_limit_fails() {
            let mut sh34 = init();
            let accounts = default_accounts();
            set_sender(accounts.alice);
            let num_of_mints: u64 = MAX_SUPPLY + 1;

            assert_eq!(sh34.total_supply(), SUPPLY_MINTED_TO_OWNER);
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

            assert_eq!(sh34.total_supply(), SUPPLY_MINTED_TO_OWNER);
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
            assert_eq!(sh34.total_supply(), SUPPLY_MINTED_TO_OWNER);
        }

        #[ink::test]
        fn token_uri_works() {
            use crate::neurolauncher::Id::U64;

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
            let mut sh34 = Neurolauncher::new(
                String::from(BASE_URI), // base_uri: String,
                PRESALE_PRICE,
                PRICE,            // price_per_mint: Balance,
                0,                // prepresale_start_at: u64,
                0,                // presale_start_at: u64,
                0,                // public_sale_start_at: u64,
                accounts.charlie, // project_treasury: AccountId,
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
            let sh34 = Neurolauncher::new(
                String::from(BASE_URI), // base_uri: String,
                PRESALE_PRICE,
                price,            // price_per_mint: Balance,
                0,                // prepresale_start_at: u64,
                0,                // presale_start_at: u64,
                0,                // public_sale_start_at: u64,
                accounts.charlie, // project_treasury: AccountId,
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
