---
title: Custom-Made Module
description: Substrate is a modular blockchain framework that makes it easy to build unique and innovative Appchains composing built-in modules with custom-made ones.
---

# Adding a Custom Made Module {: #adding-custom-made-module } 

## Introduction {: #introduction }

By providing a comprehensive library of pre-built modules addressing many common requirements, the framework greatly simplifies the process of building an Appchain and accelerates the deployment and evolution into a ContainerChain, nevertheless, addressing an innovative use case usually requires a development effort to fully meet the requirements, and, in Substrate, adding custom logic translates into writing and integrating runtime modules. 

Continuing the example presented in [Modularity](/learn/framework/modules/#custom-module-example), the step-by-step process of how to create the module will be covered in this article.

## Creating the Lottery Module {: #creating-lottery-module } 

As presented in the [custom-made module](/learn/framework/modules/#custom-modules) section of the modularity article, creating a module involves implementing the following attribute macros, where the first three are mandatory:

1. `#[frame_support::pallet]`
2. `#[pallet::pallet]`
3. `#[pallet::config]`
4. `#[pallet::call]`
5. `#[pallet::error]`
6. `#[pallet::event]`
7. `#[pallet::storage]`

But before starting to write code, the files containing the logic need to be created. From the root folder of the repository, navigate to the folder `pallets`, where the module will be created.

```bash
cd container-chains/pallets
```

Create the module with cargo, which creates a new package:

```bash
cargo new lottery-example
```

By default, the package creates a file `main.rs` located in a folder `src`, but to respect the naming convention used in Substrate, rename the file to `lib.rs`:

```bash
mv lottery-example/src/main.rs lottery-example/src/lib.rs
```

Now the module file `lib.rs` is created and ready to contain the custom logic.

### Implementing `#[frame_support::pallet]` and `#[pallet::pallet]`

The implementation of these macros and the code structure are mandatory to enable the module to be used in the runtime.

```rust
#[frame_support::pallet(dev_mode)]
pub mod pallet {
    ...
    #[pallet::pallet]
    pub struct Pallet<T>(_);
    ...
}
```

### Implementing `#[pallet::config]`

To make the modules highly adaptable, they can be configured to the specific requirements of the use case the runtime implements.

The implementation of the config macro is mandatory and sets the module's dependency on other modules and the types and values specified by the runtime settings.

In this example, the lottery module depends on other modules to manage the currency and the random function to select the winner. More about module dependency in the [Substrate documentation](https://docs.substrate.io/build/pallet-coupling/){target=_blank}.

This module also reads and uses the ticket price and the maximum number of participants directly from the runtime settings.

```rust
/// Configure the module by specifying the parameters 
/// and types on which it depends.
#[pallet::config]
pub trait Config: frame_system::Config {

    /// Because this pallet emits events, it depends on the 
    /// runtime's definition of an event.
    type RuntimeEvent: From<Event<Self>> 
        + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// This module depends on other modules, such as balances and randomness
    type Currency: Currency<Self::AccountId> 
        + ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId>;

    type MyRandomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

    /// Some values that must be configured when adding the module 
    /// to the runtime
    #[pallet::constant]
    type TicketCost: Get<BalanceOf<Self>>;

    #[pallet::constant]
    type MaxParticipants: Get<u32>;

    #[pallet::constant]
    type PalletId: Get<PalletId>;
}
```

### Implementing `#[pallet::call]`

The following code snippet shows the two transactions that this module exposes: buy_ticket and award_prize.


```rust
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
```

### Implementing `#[pallet::error]`

This macro is applied to an enumeration of errors that might occur during the execution. It is important for security reasons to handle all error cases gracefully (and never crash in the runtime).

```rust
// Errors inform users that something went wrong.
#[pallet::error]
pub enum Error<T> {
    NotEnoughCurrency,
    AccountAlreadyParticipating,
    CanNotAddParticipant,
}
```

### Implementing `#[pallet::event]`

This macro is applied to an enumeration of events to inform the user of any changes in the state or important actions that happened during the execution in the runtime.

```rust
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    /// Event emitted when a ticket is bought
    TicketBought { who: T::AccountId },
    /// Event emitted when the prize is awarded
    PrizeAwarded { winner: T::AccountId },
    /// Event emitted when there are no participants
    ThereAreNoParticipants,
}
```

### Implementing `#[pallet::storage]`

This macro initializes a runtime storage structure. In this example, a basic value storage structure is used to persist the list of participants.

In the heavily constrained environment of an AppChain, deciding what to store and which structure to use can be critical in terms of performance. More on this topic is covered in the [Substrate documentation](https://docs.substrate.io/build/runtime-storage/){target=_blank}.

```rust
#[pallet::storage]
#[pallet::getter(fn get_participants)]
pub(super) type Participants<T: Config> = StorageValue<
    _,
    BoundedVec<T::AccountId, T::MaxParticipants>,
    OptionQuery
>;
```
