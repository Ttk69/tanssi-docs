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

The implementation of those transactions also uses storage, emits events, defines custom errors, and relies on other modules to handle currency (to charge for the tickets and transfer the total amount to the winner) and randomize the winner selection. This implementation is not intended for production

In this article, the following steps, necessary to build and add the example module to the runtime, will be covered: 

1. Create the lottery module files (package)
2. Configure the module's dependencies
3. Add the code with the custom logic
4. Configure the runtime with the new module

It is important to note that none of the code presented in this article is intended for production use.

## Creating the Lottery Module Files {: #creating-lottery-module-files } 

Before starting to write code, the files containing the logic need to be created. As modules in Substrate are inherently abstract and can be reused in many different runtimes with different customizations, it is Cargo, the Rust language package manager, the command that creates the module, in the format of a new package.

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

Now the module is created and ready to contain the custom logic.

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

The full example of the `Cargo.toml` file sets, besides the attributes, the dependencies required by Substrate:

??? code "View the complete Cargo.toml file"

    ```rust
    --8<-- 'code/basic-substrate/lottery-example-cargo.toml'
    ```

## Adding Custom Logic {: #adding-custom-logic}

As presented in the [custom-made module](/learn/framework/modules/#custom-modules){target=_blank} section of the modularity article, creating a module involves implementing the following attribute macros, where the first three are mandatory:

1. `#[frame_support::pallet]` - this attribute is the entry point that marks the module as usable in the runtime
2. `#[pallet::pallet]` - applied to a structure that is used to retrieve module information easily
3. `#[pallet::config]` - is a required attribute to define the configuration for the data types of the module
4. `#[pallet::call]` -  this macro is used to define functions that will be exposed as transactions, allowing them to be dispatched to the runtime. It is here that the developers add their custom transactions and logic
5. `#[pallet::error]` - as transactions may not be successful (insufficient funds, as an error example) and for security reasons, a custom module can never end up throwing an exception, all the possible errors are to be identified and listed in an enum to be returned upon an unsuccessful execution
6. `#[pallet::event]` - events can be defined and used as a means to provide more information to the user
7. `#[pallet::storage]` - this macro is used to define elements that will be persisted in storage. As resources are scarce in a blockchain, it should be used wisely to store only sensible information

### Implementing the Module Basic Structure {: #implementing-basic-structure }

The first two mandatory macros, `#[frame_support::pallet]` and `#[pallet::pallet]`, provide the basic structure of the module and are required to enable the module to be used in the runtime.

The following snippet shows the general structure of a custom Substrate module.

```rust
#[frame_support::pallet(dev_mode)]
pub mod pallet {
    ...
    #[pallet::pallet]
    pub struct Pallet<T>(_);
    
    // All the logic goes here
}
```

### Implementing the Module Configuration {: #implementing-module-configuration }

To make the modules highly adaptable, their configuration is abstract enough to allow them to be adapted to the specific requirements of the use case the runtime implements.

The implementation of the `#[pallet::config]` macro is mandatory and sets the module's dependency on other modules and the types and values specified by the runtime-specific settings. More about module dependency in the [Substrate documentation](https://docs.substrate.io/build/pallet-coupling/){target=_blank}.


In this example, the lottery module depends on other modules to manage the currency and the random function to select the winner and also reads and uses the ticket price and the maximum number of participants directly from the runtime settings. The configurations for this module are:

1. Events: the module depends on the runtime's definition of an event to be able to emit them.
2. Currency: this module needs to transfer funds, hence, it needs the definition of the currency system from the runtime
3. Randomness: to select the winner of the prize, from the list of participants 
4. Ticket cost: the price to charge the buyers that participate in the lottery
5. Maximum number of participants: the top limit of participants allowed in each lottery round
6. Module Id: every module has an identifier, this is required to access the module account to hold the participant's funds until transferred to the winner

```rust
#[pallet::config]
pub trait Config: frame_system::Config {

    // 1. Event definition
    type RuntimeEvent: From<Event<Self>> 
        + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    // 2. Currency 
    type Currency: Currency<Self::AccountId> 
        + ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId>;

    // 3. Randomness
    type MyRandomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

    // 4. Ticket cost
    #[pallet::constant]
    type TicketCost: Get<BalanceOf<Self>>;

    // 5. Maximum number of participants
    #[pallet::constant]
    type MaxParticipants: Get<u32>;

    // 6. Module Id
    #[pallet::constant]
    type PalletId: Get<PalletId>;
}
```

This abstract definition of dependencies is crucial to avoid coupling to a specific use case, and to enable the modules to serve as basic building blocks to Substrate Appchains.

### Implementing Transactions {: #implementing-transactions } 

Calls represent the behavior a runtime exposes, in the form of transactions that can be dispatched for processing. By implementing the macro `#[pallet::call]`, the custom logic can be added to the module.

This is the general structure of the macro implementation, and the calls definition:

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

### Implementing Custom Errors {: #implementing-custom-errors}

The `#[pallet::error]` macro is applied to an enumeration of errors that might occur during the execution. It is important for security reasons to handle all error cases gracefully and never crash in the runtime.

```rust
// Errors inform users that something went wrong.
#[pallet::error]
pub enum Error<T> {
    NotEnoughCurrency,
    AccountAlreadyParticipating,
    CanNotAddParticipant,
}
```

### Implementing Events {: #implementing-events }

The `#[pallet::event]` macro is applied to an enumeration of events to inform the user of any changes in the state or important actions that happened during the execution in the runtime.

The following snippet shows the events defined in the lottery example:

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

### Implementing Storage for State Persistence {: #implementing-storage }

The `#[pallet::storage]` macro initializes a runtime storage structure. In this example, a basic value storage structure is used to persist the list of participants in a bounded capacity vector ([BoundedVec](https://crates.parity.io/frame_support/storage/bounded_vec/struct.BoundedVec.html){target=_blank}).

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

With everything set, the Apchain has now support for a basic implementation of a lottery.

--8<-- 'text/disclaimers/third-party-content.md'