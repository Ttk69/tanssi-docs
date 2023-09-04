#[pallet::call]
impl<T: Config> Pallet<T> {
    
    #[pallet::call_index(0)]
    #[pallet::weight(0)]
    pub fn buy_ticket(origin: OriginFor<T>) -> DispatchResult {
        let buyer = ensure_signed(origin)?;

        // Checks that the user has enough balance to afford the ticket price
        ensure!(
            T::Currency::free_balance(&buyer) >= T::TicketCost::get(),
            Error::<T>::NotEnoughCurrency
        );

        // Checks that the user do not have a ticket yet
        if let Some(participants) = Self::get_participants() {
            ensure!(
                !participants.contains(&buyer),
                Error::<T>::AccountAlreadyParticipating
            );
        }

        // Stores the user to participate in the lottery
        match Self::get_participants() {
            Some(mut participants) => { 
                ensure!(
                    participants.try_push(buyer.clone()).is_ok(), 
                    Error::<T>::CanNotAddParticipant
                );
                Participants::<T>::set(Some(participants));
            }, 
            None => {
                let mut participants = BoundedVec::new();
                ensure!(
                    participants.try_push(buyer.clone()).is_ok(), 
                    Error::<T>::CanNotAddParticipant
                );
                Participants::<T>::set(Some(participants));
            }
        };

        // Transfer the ticket cost to the module's account
        T::Currency::transfer(&buyer, &Self::get_pallet_account(), T::TicketCost::get(), ExistenceRequirement::KeepAlive)?;
        
        // Notify the event
        Self::deposit_event(Event::TicketBought { who: buyer });
        Ok(())
    }

    #[pallet::call_index(1)]
    #[pallet::weight(0)]
    pub fn award_prize(origin: OriginFor<T>) -> DispatchResult {
        let _who = ensure_root(origin)?;

        match Self::get_participants() {
            Some(participants) => { 
                
                // Gets a random number, using randomness module
                let nonce = Self::get_and_increment_nonce();
                let (random_seed, _) = T::MyRandomness::random(&nonce);
                let random_number = <u32>::decode(&mut random_seed.as_ref())
                    .expect("secure hashes should always be bigger than u32; qed");
                
                // Selects the winner 
                let winner_index = random_number as usize % participants.len();
                let winner = participants.as_slice().get(winner_index).unwrap();

                // Transfers the total prize to the winner's account
                let prize = T::Currency::free_balance(&Self::get_pallet_account());
                T::Currency::transfer(&Self::get_pallet_account(), &winner, prize, ExistenceRequirement::AllowDeath)?;

                // Resets the storage, and gets ready for another lottery round
                Participants::<T>::kill();

                Self::deposit_event(Event::PrizeAwarded { winner: winner.clone() } );
            }, 
            None => {
                Self::deposit_event(Event::ThereAreNoParticipants);
            }
        };

        Ok(())
    }
}
