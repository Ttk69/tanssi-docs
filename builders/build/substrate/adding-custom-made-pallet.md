---
title: Adding a Custom-Made Module
description: Substrate is a modular blockchain framework that makes it easy to build unique and innovative Appchains composing built-in modules with custom-made ones.
---

# Adding a Custom Made Module {: #adding-custom-made-module } 

## Introduction {: #introduction }

By providing a comprehensive library of pre-built modules addressing many common requirements, the framework greatly simplifies the process of building an Appchain and accelerates the deployment and evolution into a ContainerChain, nevertheless, addressing an innovative use case usually requires a development effort to fully meet the requirements, and, in Substrate, adding custom logic translates into writing and integrating runtime modules. 

The example presented in the [Modularity](/learn/framework/modules/#custom-module-example) article shows a simple lottery module exposing two transactions:

- **Buy tickets**
- **Award prize**

The implementation of those transactions also uses storage, emits events, defines custom errors, and relies on other modules to handle currency (to charge for the tickets and transfer the total amount to the winner) and randomize the winner selection.

To build and add the module to the runtime, at least the following steps are necessary: 

1. Create the lottery module files (package)
2. Configure the module's dependencies
3. Add the code with the custom logic
4. Configure the runtime with the new module

In this article, the steps to build the module are covered.

## Creating the Lottery Module Files {: #creating-lottery-module-files } 

Before starting to write code, the files containing the logic need to be created. As modules in Substrate are inherently abstract and can be reused in many different runtimes with different customizations, it is Cargo, the Rust language package manager, the command that creates the module, in the format of an new package.

From the root folder of the repository, navigate to the folder `pallets`, where the module will be created.

```bash
cd container-chains/pallets
```

Create the module package with cargo:

```bash
cargo new lottery-example
```

By default, Cargo creates the new package in a folder with the provided name (lottery-example, in this case) containing a manifest file, `Cargo.toml`, and a `src` folder, with a `main.rs` file. To respect the naming convention used in Substrate, the `main.rs` file is renamed to `lib.rs`:

```bash
mv lottery-example/src/main.rs lottery-example/src/lib.rs
```

Now the module is created an ready to contain the custom logic.

## Configure the Module's Dependencies {: #configure-module-dependencies}

Being the module an independent package, it has its own `Cargo.toml` where the module attributes and dependencies must be defined.

As an example of the attributes, the name of the module, version, authors, and other relevant information can be set.

```toml
[package]
name = "module-lottery-example"
version = "4.0.0-dev"
description = "Simple module example"
authors = [""]
homepage = ""
...
```

This file also defines the module's dependencies, such as the core functionality that allows seamless integration with the runtime and other modules, access to storage, events emission, and more. 

The full example of the file sets, besides the attributes, the dependencies required by Substrate.

??? code "View the complete toml file"

    ```rust
    --8<-- 'code/basic-substrate/lottery-example-cargo.toml'
    ```

## Adding Custom Logic {: #adding-custom-logic}

As presented in the [custom-made module](/learn/framework/modules/#custom-modules) section of the modularity article, creating a module involves implementing the following attribute macros, where the first three are mandatory:

1. `#[frame_support::pallet]`
2. `#[pallet::pallet]`
3. `#[pallet::config]`
4. `#[pallet::call]`
5. `#[pallet::error]`
6. `#[pallet::event]`
7. `#[pallet::storage]`

### Implementing `#[frame_support::pallet]` and `#[pallet::pallet]`

The implementation of these macros and the code structure are mandatory to enable the module to be used in the runtime.

The following snippet shows the general structure of a custom Substrate module.

```rust
#[frame_support::pallet(dev_mode)]
pub mod pallet {
    ...
    #[pallet::pallet]
    pub struct Pallet<T>(_);
    
    // All the logic here
}
```

### Implementing `#[pallet::config]`

To make the modules highly adaptable, they can be adapted to the specific requirements of the use case the runtime implements.

The implementation of the config macro is mandatory and sets the module's dependency on other modules and the types and values specified by the runtime settings. In this example, the lottery module depends on other modules to manage the currency and the random function to select the winner. More about module dependency in the [Substrate documentation](https://docs.substrate.io/build/pallet-coupling/){target=_blank}.

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

    /// This module depends on balances module to handle currency
    type Currency: Currency<Self::AccountId> 
        + ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId>;

    /// This module depends on randomness to fairly select the winner
    type MyRandomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

    /// The ticket price is configured in the runtime
    #[pallet::constant]
    type TicketCost: Get<BalanceOf<Self>>;

    /// The participants limit is configured in the runtime
    #[pallet::constant]
    type MaxParticipants: Get<u32>;

    /// The module id if configured in the runtime
    #[pallet::constant]
    type PalletId: Get<PalletId>;
}
```

This abstract definition of dependencies is crucial to avoid coupling to a specific use case, and to enable the modules to serve as basic building blocks to Substrate Appchains.

### Implementing `#[pallet::call]`

Calls represent the behavior a runtime exposes, in the form of transactions that can be dispatched for processing.

This is the general structure of a call:

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    
    #[pallet::call_index(0)]
    #[pallet::weight(0)]
    pub fn one_call(origin: OriginFor<T>) -> DispatchResult { }

    #[pallet::call_index(1)]
    #[pallet::weight(0)]
    pub fn another_call(origin: OriginFor<T>) -> DispatchResult { }

    // Other calls
}
```

Every call is enclosed within the `#[pallet::call]` macro, and present the following elements: 

- **Call Index** - is a mandatory unique identifier for every dispatchable call
- **Weight** - is a measure of computational effort an extrinsic takes when being processed
- **Origin** - identifies the signing account making the call
- **Result** - the return value of the call, which might be an Error if anything goes wrong

In this lottery example, we defined two calls with the following logic:

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    
    #[pallet::call_index(0)]
    #[pallet::weight(0)]
    pub fn buy_ticket(origin: OriginFor<T>) -> DispatchResult {

        // 1. Validates the origin signature
        // 2. Checks that the user has enough balance to afford the ticket price
        // 3. Checks that the user is not already participating
        // 4. Adds the user as a new participant for the prize
        // 5. Transfers the ticket cost to the module's account, to hold until transferred to the winner
    
    }

    #[pallet::call_index(1)]
    #[pallet::weight(0)]
    pub fn award_prize(origin: OriginFor<T>) -> DispatchResult {

        // 1. Validates the origin signature
        // 2. Gets a random number from the randomness module
        // 3. Selects the winner from the participants lit
        // 4. Transfers the total prize to the winner's account
        // 5. Resets the participants list, and gets ready for another lottery round

    }
}
```

These calls also emit events, to keep the user informed and can return errors, should any of the validations go wrong.

Here is the complete implementation of the calls, with the custom lottery logic:

??? code "View the complete calls code"

    ```rust
    --8<-- 'code/basic-substrate/lottery-example-calls.rs'
    ```

### Implementing `#[pallet::error]`

This macro is applied to an enumeration of errors that might occur during the execution. It is important for security reasons to handle all error cases gracefully and never crash in the runtime.

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

This macro initializes a runtime storage structure. In this example, a basic value storage structure is used to persist the list of participants in a bounded vector ([BoundedVec](https://crates.parity.io/frame_support/storage/bounded_vec/struct.BoundedVec.html){target=_blank}).

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

## Configure the Runtime {: #configure-runtime }

Finally, with the module finished, it can be included in the runtime. By doing so, the transactions buy_tickets and award_prize will be callable by the users.

First, configure the modules:

```rust

// Add the configuration for randomness module
impl pallet_insecure_randomness_collective_flip::Config for Runtime {
}

// Custom module id
parameter_types! {
	pub const PalletId: PalletId = PalletId(*b"loex5678");
}

// Add configuration for the lottery module
impl pallet_lottery_example::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type TicketCost = ConstU128<1000000000000000>;
	type PalletId = PalletId;
	type MaxParticipants = ConstU32<500>;
	type MyRandomness = RandomCollectiveFlip;
}
```

And in the construction of the runtime, add the randomness and lottery module.

```rust
construct_runtime!(
	pub struct Runtime {
        ...
        // Include the custom logic from the pallet-template in the runtime.
        RandomCollectiveFlip: pallet_insecure_randomness_collective_flip,
        Lottery: pallet_lottery_example,
        ...
    }
)
```

With everything set, the Apchain has now support for a basic implementatios of a lottery.