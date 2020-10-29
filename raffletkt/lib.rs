#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod raffletkt {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_prelude::format;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        collections::{
            HashMap as StorageHashMap,
            Stash as StorageStash,
        }
    };

    use ink_env::{
        hash::{
            Keccak256,
        }
    };

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct RaffleTkt {

        /// Player Pool :: list of players purchaed the raffle ticket
        player_pool: StorageStash<AccountId>,

        /// Player Pool :: list of players purchaed the raffle ticket
        winners_pool: StorageStash<AccountId>,

        /// Player Status in the pool :: active or not
        player_status: StorageHashMap<AccountId, bool>,

        /// beneficiary
        fund_beneficiary: AccountId,

        /// Validity time-stamp
        validity_timestamp: u64,

        /// Number of registered players
        num_players: u32,

        /// Number of registered players
        total_balance: Balance

    }

    impl RaffleTkt {

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                player_pool: StorageStash::default(),
                winners_pool: StorageStash::default(),
                player_status: StorageHashMap::default(),
                fund_beneficiary: Default::default(),
                validity_timestamp: Default::default(),
                num_players: 0,
                total_balance: 0,
            }
        }

        #[ink(message)]
        pub fn update_raffle_beneficiary(&mut self, raffle_beneficiary: AccountId ) {
            self.fund_beneficiary = raffle_beneficiary;
            self.validity_timestamp = self.env().block_timestamp() + ( 5 * 60 * 1000 );

            if cfg!(test) {
                let dbg_msg = format!( "validity_timestamp {:#?}", self.validity_timestamp );
                ink_env::debug_println( &dbg_msg );
            }

        }

        #[ink(message)]
        #[ink(payable)]
        pub fn raffle_play(&mut self) -> bool {
            let mut rval = false;
            let deposit_min = 10000000000;
            let deposit_max = 100000000000;
            let caller = self.env().caller();
            let value = self.env().transferred_balance();

            if cfg!(test) {
                let dbg_msg = format!( "raffle_play value {:#?}", value );
                ink_env::debug_println( &dbg_msg );
            }

            if cfg!(test) {
                let dbg_msg = format!( "raffle_play bts {:#?}", self.env().block_timestamp() );
                ink_env::debug_println( &dbg_msg );
            }

            // Check player is new to player pool
            let player_status = self.player_status.contains_key( &caller );
            // assert!( player_status == false );
            if player_status == true {
                return rval
            }

            // Check for transfered balance range
            if value < deposit_min || value > deposit_max {
                return rval
            }

            self.player_status.insert( caller, true );
            self.player_pool.put( caller );

            self.num_players = self.player_pool.len();

            // match self.env().transfer( self.env().account_id(), value ) {
            //     _ok => ()
            // }

            self.total_balance += value;

            if cfg!(test) {
                let dbg_msg = format!( "Cont Curr Balance { }", self.env().balance() );
                ink_env::debug_println( &dbg_msg );
            }

            if cfg!(test) {
                let dbg_msg = format!( "player_pool len { }", self.num_players );
                ink_env::debug_println( &dbg_msg );
            }

            rval = true;
            rval

        }

        #[ink(message)]
        pub fn raffle_draw(&mut self) -> bool {
            let mut rval = false;

            let caller = self.env().caller();

            // Check for minimum of 5 players to start the draw
            // assert!( self.num_players >= 5 );
            if self.num_players < 5 {
                return rval
            }

            // Check player is in player pool
            let player_status = self.player_status.contains_key( &caller );
            // assert!( player_status == true );
            if player_status == false {
                return rval
            }

            // Check winners in the pool is less than 2 ...
            // assert!( self.winners_pool.len() < 2 );
            if self.winners_pool.len() >= 2 {
                return rval
            }

            // assert!( self.env().block_timestamp() > self.validity_timestamp );

            if cfg!(test) {
                let dbg_msg = format!( "player_pool len {:#?}", self.player_pool.len() );
                ink_env::debug_println( &dbg_msg );
            }

            let rand_indx = self.get_random() % self.player_pool.len( );

            if cfg!(test) {
                let dbg_msg = format!( "random indx { }", rand_indx );
                ink_env::debug_println( &dbg_msg );
            }

            if self.player_pool.get(rand_indx).is_some() {

                // let rand_id = ( self.player_pool.get(rand_indx) ).unwrap();

                // if *rand_id == caller {
                    let winner_id = ( self.player_pool.take( rand_indx ) ).unwrap();

                    self.winners_pool.put( winner_id );
                // }
            }

            rval = true;
            rval
        }

        #[ink(message)]
        pub fn raffle_transfer_fund_to_beneficiary(&mut self) {

            if self.winners_pool.len() == 2 && self.total_balance > 0 {

                if cfg!(test) {
                    let dbg_msg = format!( "benefit val { }", self.total_balance );
                    ink_env::debug_println( &dbg_msg );
                }

                match self.env().transfer( self.fund_beneficiary, self.total_balance ) {
                    _ok => ()
                }

                self.total_balance = 0;
            }
        }

        #[ink(message)]
        pub fn raffle_isgamedone(&self) -> bool {

            self.winners_pool.len() == 2

        }

        #[ink(message)]
        pub fn raffle_getwinnerid(&self) -> Vec<AccountId> {

            let mut winners: Vec<AccountId> = Default::default();

            for win_item in self.winners_pool.iter() {
                winners.push( *win_item );
            }

            winners
        }

        #[ink(message)]
        pub fn raffle_getplayersid(&self) -> Vec<AccountId> {

            let mut players: Vec<AccountId> = Default::default();

            for player_item in self.player_pool.iter() {
                players.push( *player_item );
            }

            players
        }

        #[ink(message)]
        pub fn raffle_getcontract_balance(&self) -> Balance {

            self.env().balance()

        }

        #[ink(message)]
        pub fn raffle_getdonation_balance(&self) -> Balance {

            self.total_balance

        }

        #[ink(message)]
        pub fn raffle_fund_beneficiary(&self) -> AccountId {

            self.fund_beneficiary

        }

        fn get_random() -> u32 {
            let encodable = ( Self::env().caller(), Self::env().block_timestamp() , Self::env().block_number() );
            let keccak256_output = Self::env().hash_encoded::<Keccak256, _>( &encodable );
            let mut rand_hash = Self::env().random(&keccak256_output);
            let rand_num = rand_hash.as_mut();
            let mut rval: u32 = 0;
            for val in rand_num.iter() {
                rval += *val as u32;
            }
            rval
        }

    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_env::{
            call,
            test,
        };
        use ink_lang as ink;

        type Accounts = test::DefaultAccounts<Environment>;
        const WALLET: [u8; 32] = [7; 32];

        fn default_accounts() -> Accounts {
            test::default_accounts()
                .expect("Test environment is expected to be initialized.")
        }

        fn set_sender(sender: AccountId, endowment: Balance ) {
            test::push_execution_context::<Environment>(
                sender,
                WALLET.into(),
                1000000,
                endowment,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
        }

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let mut raffletkt = RaffleTkt::default();
            let accounts = default_accounts();

            let dbg_msg = format!( "Raffle Add Beneficiary" );
            ink_env::debug_println( &dbg_msg );

            // let ben_bal: Result<T::Balance, E::Err> = ink_env::test::get_account_balance( accounts.alice );
            // match ben_bal {
            //     Err(why) => panic!("{:?}", why),
            //     Ok(ben_bal) => {
            //         let dbg_msg = format!( "Beneficiery balance ... {}", ben_bal );
            //         ink_env::debug_println( &dbg_msg );
            //     },
            // }

            raffletkt.update_raffle_beneficiary( accounts.alice );

            let dbg_msg = format!( "Raffle Start Donate" );
            ink_env::debug_println( &dbg_msg );

            let tst_players_list = ink_prelude::vec![ ( accounts.bob, 15000000000 ),
                                                    ( accounts.charlie, 15000000000 ),
                                                    ( accounts.django, 15000000000 ),
                                                    ( accounts.eve, 15000000000 ),
                                                    ( accounts.frank, 15000000000 ) ];

            for ( tst_player_inx, tst_player_val )  in tst_players_list.iter() {
                set_sender( *tst_player_inx , *tst_player_val );
                let play_stat = raffletkt.raffle_play();
                let dbg_msg = format!( "Raffle Play Status {}", play_stat );
                ink_env::debug_println( &dbg_msg );
            }

            let dbg_msg = format!( "Raffle Start Draw" );
            ink_env::debug_println( &dbg_msg );

            let mut tst_break_loop = false;
            while tst_break_loop == false {

                for ( tst_player_inx, tst_player_val )  in tst_players_list.iter() {
                    set_sender( *tst_player_inx , *tst_player_val );

                    let draw_stat = raffletkt.raffle_draw();
                    let dbg_msg = format!( "Raffle Draw Status {}", draw_stat );
                    ink_env::debug_println( &dbg_msg );

                    if raffletkt.raffle_isgamedone() == true {
                        let dbg_msg = format!( "Two Winners Selected Game Over !!!" );
                        ink_env::debug_println( &dbg_msg );
                        tst_break_loop = true;
                        break;
                    }
                }
            }

            let dbg_msg = format!( "Raffle Winners list ..." );
            ink_env::debug_println( &dbg_msg );

            let win_list = raffletkt.raffle_getwinnerid();

            for win_item in win_list.iter() {
                let dbg_msg = format!( "{:#?}", win_item );
                ink_env::debug_println( &dbg_msg );
            }


            // match ink_env::test::get_account_balance( accounts.alice ){
            //     Err(why) => panic!("{:?}", why),
            //     Ok(ben_bal) => {
            //         let dbg_msg = format!( "Beneficiery balance ... {}", ben_bal );
            //         ink_env::debug_println( &dbg_msg );
            //     },
            // }

        }
    }
}
