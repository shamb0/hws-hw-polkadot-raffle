#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod raffletkt {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_prelude::format;
    use ink_prelude::vec::Vec as PreludeVec;

    use ink_storage::{
        collections::{
            HashMap as StorageHashMap,
            Vec as StorageVec,
        }
    };

    /// Minimum player bet is 0.01 unit
    const BET_VALUE_MIN: u128 = 10_000_000_000_000;
    /// Maximum player bet is 0.1 unit
    const BET_VALUE_MAX: u128 = 100_000_000_000_000;

    #[ink(storage)]
    pub struct RaffleTkt {

        /// Player Pool :: list of players purchaed the raffle ticket
        player_pool: StorageVec<AccountId>,

        /// Player Pool :: list of players purchaed the raffle ticket
        winners_pool: StorageVec<AccountId>,

        /// Player Status in the pool :: active or not
        player_status: StorageHashMap<AccountId, bool>,

        /// beneficiary
        fund_beneficiary: AccountId,

        /// minimum raffle duration
        minimum_raffle_lock_duration: u64,

        /// time stamp at which to start the draw
        raffle_draw_time_stamp: u64,

        /// Number of registered players
        num_players: u32,

        /// Minimum Number of players needed to start the draw
        min_num_players: u32,

        /// Number of registered players
        total_balance: Balance,

        /// raffle contract owner
        raffle_owner: AccountId,

    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// The user has already played for this raffle
        UserHasAlreadyPlayed,
        /// Bet should be between 0.1 and 0.01 Unit
        IncorrectBet,
        /// Raffle cannot be drawn yet
        RaffleNotDrawable,
        /// Raffle is closed
        RaffleClosed,
        /// Caller not in player list
        InvalidPlayer,
        /// The transfer has failed
        TransferFailed,
        /// Invalid Contract Owner
        InvalidOwner
    }
    pub type Result<T> = core::result::Result<T, Error>;


    #[ink(event)]
    /// New player enters the raffle
    pub struct NewPlayer {
        #[ink(topic)]
        /// The player account
        player: AccountId,
    }

    #[ink(event)]
    /// The raffle is started, meaning enough players are playing
    pub struct RaffleStarted {
        #[ink(topic)]
        /// The start date
        start_date: Timestamp,
    }

    #[ink(event)]
    /// A winner has been picked
    pub struct WinnerPicked {
        #[ink(topic)]
        /// The winner account
        winner: AccountId,
    }

    #[ink(event)]
    /// A invalid player index
    pub struct PlayerInvalidIndex {
        #[ink(topic)]
        /// invalid index
        index: u32,
    }

    #[ink(event)]
    pub struct TransferFailed {
    }

    impl RaffleTkt {

        #[ink(constructor)]
        pub fn default( raffle_beneficiary: AccountId, minimum_players: u32, minimum_raffle_duration: u64) -> Self {

            let caller = Self::env().caller();

            let raffle_obj = Self {
                player_pool: StorageVec::new(),
                winners_pool: StorageVec::new(),
                player_status: StorageHashMap::default(),
                fund_beneficiary: raffle_beneficiary,
                minimum_raffle_lock_duration: minimum_raffle_duration,
                raffle_draw_time_stamp: 0,
                min_num_players: minimum_players,
                num_players: 0,
                total_balance: 0,
                raffle_owner: caller,
            };

            raffle_obj
        }

        #[ink(message, payable)]
        pub fn raffle_play(&mut self) -> Result<()> {

            let deposit_min = BET_VALUE_MIN;
            let deposit_max = BET_VALUE_MAX;
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
                return Err(Error::UserHasAlreadyPlayed)
            }

            // Check for transfered balance range
            if value < deposit_min || value > deposit_max {
                return Err(Error::IncorrectBet)
            }

            self.player_status.insert( caller, true );
            self.player_pool.push( caller );

            self.num_players = self.player_pool.len();

            self.total_balance += value;

            Self::env().emit_event( NewPlayer {
                player: caller,
            });

            if self.num_players >= self.min_num_players {

                self.raffle_draw_time_stamp = Self::env().block_timestamp() + self.minimum_raffle_lock_duration;

                Self::env().emit_event( RaffleStarted {
                    start_date: self.raffle_draw_time_stamp,
                });
            }

            if cfg!(test) {
                let dbg_msg = format!( "Cont Curr Balance { }", self.env().balance() );
                ink_env::debug_println( &dbg_msg );
            }

            if cfg!(test) {
                let dbg_msg = format!( "player_pool len { }", self.num_players );
                ink_env::debug_println( &dbg_msg );
            }

            return Ok(())
        }

        #[ink(message)]
        pub fn raffle_draw(&mut self) -> Result<()> {

            let caller = self.env().caller();

            // Check if max winner is selecetd ...
            if self.raffle_is_game_closed() == true {
                return Err(Error::RaffleClosed)
            }

            if self.raffle_is_draw_open() == false {
                return Err(Error::RaffleNotDrawable)
            }

            // Check player is in player pool
            let player_status = self.player_status.contains_key( &caller );
            // assert!( player_status == true );
            if player_status == false {
                return Err(Error::InvalidPlayer)
            }


            if cfg!(test) {
                let dbg_msg = format!( "player_pool len {:#?}", self.player_pool.len() );
                ink_env::debug_println( &dbg_msg );
            }

            let rand_indx = Self::get_random() % self.player_pool.len( );

            if cfg!(test) {
                let dbg_msg = format!( "random indx { }", rand_indx );
                ink_env::debug_println( &dbg_msg );
            }

            if self.player_pool.get(rand_indx).is_some() {

                let winner = ( self.player_pool.swap_remove( rand_indx ) ).unwrap();

                self.winners_pool.push( winner );

                Self::env().emit_event(WinnerPicked {
                    winner
                });

            } else {

                Self::env().emit_event(PlayerInvalidIndex {
                    index: rand_indx
                });

            }

            // Check if max winner is selecetd ...
            if self.raffle_is_game_closed() == true {

                return self.raffle_transfer_fund_to_beneficiary()

            }

            return Ok(())
        }

        #[ink(message)]
        pub fn raffle_getwinnerid(&self) -> PreludeVec<AccountId> {

            let mut winners: PreludeVec<AccountId> = PreludeVec::default();

            for win_item in self.winners_pool.iter() {
                winners.push( *win_item );
            }

            winners
        }

        #[ink(message)]
        pub fn raffle_getplayersid(&self) -> PreludeVec<AccountId> {

            let mut players: PreludeVec<AccountId> = PreludeVec::new();

            for player_item in self.player_pool.iter() {
                players.push( *player_item );
            }

            players
        }


        #[ink(message)]
        pub fn raffle_is_draw_open(&self) -> bool {

            Self::env().block_timestamp() >= self.raffle_draw_time_stamp

        }

        #[ink(message)]
        pub fn raffle_is_game_closed(&self) -> bool {

            self.winners_pool.len() == 2
        }


        #[ink(message)]
        pub fn raffle_get_fund_beneficiary_id(&self) -> AccountId {

            self.fund_beneficiary

        }

        #[ink(message)]
        pub fn raffle_getdonation_balance(&self) -> Balance {

            self.total_balance

        }

        #[ink(message)]
        /// Terminate the raffle contract
        pub fn raffle_terminate(&mut self) -> Result<()> {

            let caller = self.env().caller();

            if caller == self.raffle_owner {

                if self.raffle_is_game_closed() == true {

                    self.env().terminate_contract(self.raffle_owner);
                }

            }
            else {
                return Err(Error::InvalidOwner)
            }

            return Ok(())
        }

        fn raffle_transfer_fund_to_beneficiary(&mut self) -> Result<()> {

            if self.winners_pool.len() == 2 && self.total_balance > 0 {

                if cfg!(test) {
                    let dbg_msg = format!( "benefit val { }", self.total_balance );
                    ink_env::debug_println( &dbg_msg );
                }

                let transfer_result = self.env().transfer( self.fund_beneficiary, self.total_balance );

                if transfer_result.is_err() {
                    Self::env().emit_event(TransferFailed {});
                    return Err(Error::TransferFailed)
                }

                self.total_balance = 0;
            }

            return Ok(())
        }

        fn get_random() -> u32 {
            let seed: [u8; 8] = [70, 93, 3, 03, 15, 124, 148, 18];
            let random_hash = Self::env().random(&seed);
            Self::as_u32_be(&random_hash.as_ref())
        }

        fn as_u32_be(array: &[u8]) -> u32 {
            ((array[0] as u32) << 24) +
                ((array[1] as u32) << 16) +
                ((array[2] as u32) << 8) +
                ((array[3] as u32) << 0)
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

            let accounts = default_accounts();

            let mut raffletkt = RaffleTkt::default( accounts.alice, 4, 4 * 1000);

            let dbg_msg = format!( "Raffle Start Donate" );
            ink_env::debug_println( &dbg_msg );

            let tst_players_list = ink_prelude::vec![ ( accounts.bob, 15_000_000_000_000 ),
                                                    ( accounts.charlie, 15_000_000_000_000 ),
                                                    ( accounts.django, 15_000_000_000_000 ),
                                                    ( accounts.eve, 15_000_000_000_000 ),
                                                    ( accounts.frank, 15_000_000_000_000 ) ];

            for ( tst_player_inx, tst_player_val )  in tst_players_list.iter() {
                set_sender( *tst_player_inx , *tst_player_val );
                let play_stat = raffletkt.raffle_play();

                match play_stat {
                    Ok(_) => {
                        let dbg_msg = format!( "Raffle Play Status Ok");
                        ink_env::debug_println( &dbg_msg );
                    }
                    Err(msg) => {
                        let dbg_msg = format!( "Raffle Play Status Err{:?}", msg );
                        ink_env::debug_println( &dbg_msg );
                    }
                }
            }

            let dbg_msg = format!( "Raffle Start Draw" );
            ink_env::debug_println( &dbg_msg );

            let mut tst_break_loop = false;
            while tst_break_loop == false {

                for ( tst_player_inx, tst_player_val )  in tst_players_list.iter() {
                    set_sender( *tst_player_inx , *tst_player_val );

                    let draw_stat = raffletkt.raffle_draw();

                    match draw_stat {
                        Ok(_) => {
                            let dbg_msg = format!( "Raffle Draw Status Ok");
                            ink_env::debug_println( &dbg_msg );
                        }
                        Err(msg) => {
                            let dbg_msg = format!( "Raffle Draw Status Err{:?}", msg );
                            ink_env::debug_println( &dbg_msg );
                        }
                    }

                    if raffletkt.raffle_is_game_closed() == true {
                        let dbg_msg = format!( "Two Winners Selected Game Over !!!" );
                        ink_env::debug_println( &dbg_msg );
                        tst_break_loop = true;

                        raffletkt.raffle_transfer_fund_to_beneficiary();

                        let trans_balan_stat = raffletkt.raffle_draw();

                        match trans_balan_stat {
                            Ok(_) => {
                                let dbg_msg = format!( "Raffle Draw Status Ok");
                                ink_env::debug_println( &dbg_msg );
                            }
                            Err(msg) => {
                                let dbg_msg = format!( "Raffle Draw Status Err{:?}", msg );
                                ink_env::debug_println( &dbg_msg );
                            }
                        }

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
