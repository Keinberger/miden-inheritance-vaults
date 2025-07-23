# Building Inheritance Vaults with Miden Notes: Solving Crypto's Billion-Dollar Problem

## Overview

Over time, billions of dollars in cryptocurrency have disappeared forever—not from hacks or market crashes, but from something far more certain: death. When crypto holders pass away without proper inheritance mechanisms, their digital assets become permanently inaccessible, contributing to a growing graveyard of lost wealth. Traditional finance has established inheritance mechanisms: bank accounts have beneficiaries, investment accounts transfer to heirs, and courts can force access when necessary. Cryptocurrency, designed to be permissionless and decentralized, lacks these safeguards by design. But what if we could create a completely trustless inheritance system using pure mathematics and cryptography? With Miden Notes, we can build inheritance vaults that solve this critical problem without relying on any intermediaries.

**What You'll Learn In This Guide:**

- Key concepts like Miden Accounts and Miden Notes
- Creating time-locked cryptographic vaults for a trustless inheritance mechanism
- Working with Miden Assembly

## Prerequisites

1. Ensure you have Rust installed locally on your machine. To install Rust, follow the instructions here: [Rust installation guide](https://www.rust-lang.org/tools/install)
2. Ensure you have a local Miden node running. This is needed for interacting with the Miden network. To get the node running locally, follow the instructions on the [Miden Node Setup page](https://0xmiden.github.io/miden-docs/imported/miden-tutorials/src/miden_node_setup.html).

### What Are Miden Notes?

Before building our inheritance vault, let's understand Miden's fundamental architecture. Unlike Ethereum's account-based model, Miden uses a **note-based system** similar to Bitcoin's UTXO model, but with programmable smart contract capabilities.

#### Key Concepts:

**Miden Notes** are cryptographic objects that:

- Contain assets (tokens, NFTs, or other digital value)
- Include executable code that defines spending conditions
- May only be consumed (spent) when specific conditions are met (defined by its executable code)
- Generate cryptographic proofs when consumed

Think of a Miden Note as a **programmable safe deposit box**. The box contains your assets, has a sophisticated lock with custom rules, and can only be opened when those rules are satisfied.

If you want to learn even more about Miden Notes, you can have a look here: [Miden Notes Docs](https://0xmiden.github.io/miden-docs/imported/miden-base/src/note.html).

**Miden Accounts** are persistent entities that:

- Own and manage data or assets
- Execute transactions to create and consume notes
- Maintain state across multiple transactions

Think of Miden Accounts like fancy Ethereum EOA's, with the additional ability of natively being able to store code. The last aspect is not relevant for this tutorial but useful to know!

## Architecture Overview: How Our Inheritance Vault Works

Our inheritance vault system operates on a simple but powerful principle:

1. **Vault Creation**: The owner creates a note containing inheritance funds (tokens) with a built-in deadline
2. **Inheritance Claim**: After the deadline passes, the designated beneficiary can claim the inheritance funds
3. **Deadline Extension**: The owner can consume the existing note and create a new one at any time. This has the basic effect of "extending the deadline", and also serves the purpose of proving that the owner is still alive.

Here's the conceptual flow:

```
Owner Creates Vault → Vault Active → (Optional): Owner Extends Deadline By Claiming Assets & Creating New Vault
                                   ↓
                      Deadline Passes → Beneficiary Claims Assets
```

The beauty of this system is its **trustless nature** with no intermediaries, no courts, no third parties. Just mathematics ensuring that either the owner maintains control or the beneficiary gains access.

## Step-by-Step Implementation

### Step 1: Initializing the repository

Create a new Rust repository for the project using the following commands:

```bash
cargo new miden-rust-client
cd miden-rust-client
```

Make sure to add the following dependencies to your Cargo.toml file:

```toml
[dependencies]
miden-client = { version = "0.9.2", features = ["testing", "concurrent", "tonic", "sqlite"] }
miden-lib = { version = "0.9.4", default-features = false }
miden-objects = { version = "0.9.4", default-features = false }
miden-crypto = { version = "0.14.0", features = ["executable"] }
miden-assembly = "0.14.0"
rand = { version = "0.9" }
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1.0", features = ["raw_value"] }
tokio = { version = "1.40", features = ["rt-multi-thread", "net", "macros", "fs"] }
rand_chacha = "0.9.0"
miden-client-tools = "0.2.3"

[[bin]]
name = "inheritance_vault"
path = "src/main.rs"
```

### Step 2: Building the core vault script

Now let's dive into the Miden Assembly code for creating our inheritance vault. Think of it as the "Smart Contract Code" of our Vault.

To get started, let's create a new directory and file for our Note code:

```bash
mkdir masm
touch masm/inheritance_vault_note.masm
```

#### Disclaimer about Miden Assembly

Miden Assembly is a low-level, stack-based language which works primarily with operand instructions and manipulating stack. If you are unfamiliar with working with low-level languages, you can read our comprehensive documentation on Miden Assembly and learn how it works.

[Miden Assembly Documentation](https://0xmiden.github.io/miden-docs/glossary.html?highlight=assembly#miden-assembly)

Open this file to implement our first lines of code.

First, we are going to build the main skeleton of our Note code. We are going to define two functions, `verify_inheritance_claim` and `add_note_assets_to_account`. The latter will be used for releasing funds to the consumer of the Note if all checks pass. Also, we are going to add the "main" function, which gets executed automatically when consuming the Note. Finally, we are also going to add the libraries that our Note code will require and constant error statement for any checks that fail.

Add this snippet to the `masm/inheritance_vault_note.masm` file:

```masm
use.miden::account
use.miden::note
use.miden::tx
use.miden::contracts::wallets::basic->wallet

const.ERR_WRONG_NUMBER_OF_INPUTS="Time-locked note expects exactly 3 note inputs"
const.ERR_WRONG_BENEFICIARY="Only designated beneficiary can claim this inheritance"
const.ERR_TOO_EARLY="Inheritance deadline has not passed yet"

proc.add_note_assets_to_account
    # Our logic for transferring funds after will be here
end

# Input stack: [beneficiary_suffix, beneficiary_prefix, deadline]
proc.verify_inheritance_claim
    # Our logic for verifying an inheritance claim will be here
end

begin
    # Our main logic will be here
end
```

### Step 3: Implementing the asset transfer logic

The following code will use the Miden Note library to obtain which assets are contained inside of this Note. For each asset found, the function executes a for loop to transfer all funds to the caller of the Note.

Modify the `add_note_assets_to_account` function to implement the following code:

```masm
# Input Stack: []
proc.add_note_assets_to_account
    push.0 exec.note::get_assets
    # => [num_of_assets, 0 = ptr, ...]

    # compute the pointer at which we should stop iterating
    mul.4 dup.1 add
    # => [end_ptr, ptr, ...]

    # pad the stack and move the pointer to the top
    padw movup.5
    # => [ptr, 0, 0, 0, 0, end_ptr, ...]

    # compute the loop latch
    dup dup.6 neq
    # => [latch, ptr, 0, 0, 0, 0, end_ptr, ...]

    while.true
      # => [ptr, 0, 0, 0, 0, end_ptr, ...]

      # save the pointer so that we can use it later
      dup movdn.5
      # => [ptr, 0, 0, 0, 0, ptr, end_ptr, ...]

      # load the asset
      mem_loadw
      # => [ASSET, ptr, end_ptr, ...]

      # pad the stack before call
      padw swapw padw padw swapdw
      # => [ASSET, pad(12), ptr, end_ptr, ...]

      # add asset to the account
      call.wallet::receive_asset
      # => [pad(16), ptr, end_ptr, ...]

      # clean the stack after call
      dropw dropw dropw
      # => [0, 0, 0, 0, ptr, end_ptr, ...]

      # increment the pointer and compare it to the end_ptr
      movup.4 add.4 dup dup.6 neq
      # => [latch, ptr+4, ASSET, end_ptr, ...]
    end

    # clear the stack
    drop dropw drop
end
```

### Step 4: Implementing the owner verification logic

The following code will use the Miden Account and Sender libraries to check if the caller is the original author (sender) of the Note. If that is the case, the main function will automatically initiate the asset transfer to the sender, i.e. owner of the inheritance funds. If not the case, the main logic will execute the `verify_inheritance_claim` function.

This function accepts inputs, which you can think of as the "deployment parameters" of the Note. These consist of the deadline block number, as well as the beneficiary ID split into its suffix and prefix.

Make sure to read each line of code and respective comment to understand how the Stack is being manipulated and what the individual operands do.

Modify the main function function to implement the following code:

```masm
begin
    # Push inputs to stack
    push.0 exec.note::get_inputs
    # Stack: [num_inputs, inputs_ptr]
    eq.3 assert.err=ERR_WRONG_NUMBER_OF_INPUTS
    # Stack: [inputs_ptr]
    padw movup.4 mem_loadw drop
    # Stack: [beneficiary_suffix, beneficiary_prefix, deadline]

    # Push sender id to stack
    exec.note::get_sender
    # Stack: [sender_suffix, sender_prefix, beneficiary_suffix, beneficiary_prefix, deadline]

    # Push current id to the stack
    exec.account::get_id
    # Stack: [account_suffix, account_prefix, sender_suffix, sender_prefix, beneficiary_suffix, beneficiary_prefix, deadline]

    # Do what it used to do before (sender address verification, if not the same => verify inheritance claim with loaded stack)
    exec.account::is_id_equal
    # Stack: [is_equal, beneficiary_suffix, beneficiary_prefix, deadline]

    if.true
      # Clear all stack elements and ensure clean state
      drop drop drop
      exec.add_note_assets_to_account
    else
      exec.verify_inheritance_claim
    end
end
```

### Step 5: Implementing the claim verification logic

Finally, we are going to implement the logic of the `verify_inheritance_claim` function. Here we are going to check that the caller is indeed the beneficiary, and secondly, that the deadline has passed. If both checks succeed, the function is going to initiate the release of the funds.

Again, it's a good idea to read each line of code and respective comment to thoroughly understand the Assembly code.

Modify the `verify_inheritance_claim` function to implement the following code:

```masm
# Input stack: [beneficiary_suffix, beneficiary_prefix, deadline]
proc.verify_inheritance_claim
    # Push current account id to the stack
    exec.account::get_id
    # Stack: [account_suffix, account_prefix, beneficiary_suffix, beneficiary_prefix, deadline]

    # Verify if the current account id is the same as the beneficiary id
    exec.account::is_id_equal assert.err=ERR_WRONG_BENEFICIARY
    # Stack: [deadline]

    # Push current block number to the stack
    exec.tx::get_block_number
    # Stack: [block_number, deadline]


    # Verify the deadline has passed
    gte assert.err=ERR_TOO_EARLY
    # Stack: []

    # Execute release of funds
    exec.add_note_assets_to_account
end
```

### Step 6: Final Note Code

After following the previous steps, the `masm/inheritance_vault_note.masm` should look like the following:

```masm
use.miden::account
use.miden::note
use.miden::tx
use.miden::contracts::wallets::basic->wallet

const.ERR_WRONG_NUMBER_OF_INPUTS="Time-locked note expects exactly 3 note inputs"
const.ERR_WRONG_BENEFICIARY="Only designated beneficiary can claim this inheritance"
const.ERR_TOO_EARLY="Inheritance deadline has not passed yet"

# Input Stack: []
proc.add_note_assets_to_account
    push.0 exec.note::get_assets
    # => [num_of_assets, 0 = ptr, ...]

    # compute the pointer at which we should stop iterating
    mul.4 dup.1 add
    # => [end_ptr, ptr, ...]

    # pad the stack and move the pointer to the top
    padw movup.5
    # => [ptr, 0, 0, 0, 0, end_ptr, ...]

    # compute the loop latch
    dup dup.6 neq
    # => [latch, ptr, 0, 0, 0, 0, end_ptr, ...]

    while.true
      # => [ptr, 0, 0, 0, 0, end_ptr, ...]

      # save the pointer so that we can use it later
      dup movdn.5
      # => [ptr, 0, 0, 0, 0, ptr, end_ptr, ...]

      # load the asset
      mem_loadw
      # => [ASSET, ptr, end_ptr, ...]

      # pad the stack before call
      padw swapw padw padw swapdw
      # => [ASSET, pad(12), ptr, end_ptr, ...]

      # add asset to the account
      call.wallet::receive_asset
      # => [pad(16), ptr, end_ptr, ...]

      # clean the stack after call
      dropw dropw dropw
      # => [0, 0, 0, 0, ptr, end_ptr, ...]

      # increment the pointer and compare it to the end_ptr
      movup.4 add.4 dup dup.6 neq
      # => [latch, ptr+4, ASSET, end_ptr, ...]
    end

    # clear the stack
    drop dropw drop
end

# Input stack: [beneficiary_suffix, beneficiary_prefix, deadline]
proc.verify_inheritance_claim
    # Push current account id to the stack
    exec.account::get_id
    # Stack: [account_suffix, account_prefix, beneficiary_suffix, beneficiary_prefix, deadline]

    # Verify if the current account id is the same as the beneficiary id
    exec.account::is_id_equal assert.err=ERR_WRONG_BENEFICIARY
    # Stack: [deadline]

    # Push current block number to the stack
    exec.tx::get_block_number
    # Stack: [block_number, deadline]


    # Verify the deadline has passed
    gte assert.err=ERR_TOO_EARLY
    # Stack: []

    # Execute release of funds
    exec.add_note_assets_to_account
end

begin
    # Push inputs to stack
    push.0 exec.note::get_inputs
    # Stack: [num_inputs, inputs_ptr]
    eq.3 assert.err=ERR_WRONG_NUMBER_OF_INPUTS
    # Stack: [inputs_ptr]
    padw movup.4 mem_loadw drop
    # Stack: [beneficiary_suffix, beneficiary_prefix, deadline]

    # Push sender id to stack
    exec.note::get_sender
    # Stack: [sender_suffix, sender_prefix, beneficiary_suffix, beneficiary_prefix, deadline]

    # Push current id to the stack
    exec.account::get_id
    # Stack: [account_suffix, account_prefix, sender_suffix, sender_prefix, beneficiary_suffix, beneficiary_prefix, deadline]

    # Do what it used to do before (sender address verification, if not the same => verify inheritance claim with loaded stack)
    exec.account::is_id_equal
    # Stack: [is_equal, beneficiary_suffix, beneficiary_prefix, deadline]

    if.true
      # Clear all stack elements and ensure clean state
      drop drop drop
      exec.add_note_assets_to_account
    else
      exec.verify_inheritance_claim
    end
end
```

### Step 7: Initializing the Rust client

Before interacting with the Miden network to publish our inheritance vault, we must instantiate the Miden client. It will be our gateway to the Miden network. Also, we will instantiate a local keystore, which is used to store private keys for accounts. Finally, we will also add a helper function to create and deploy a token faucet. A faucet account on Miden mints fungible tokens.

Add the following code to the `src/main.rs` file:

```rust
use rand::{rngs::StdRng, RngCore};
use std::{fs, path::Path, sync::Arc};
use tokio::time::{sleep, Duration};

use miden_client::{
    account::{
        component::{BasicFungibleFaucet, RpoFalcon512},
        AccountBuilder, AccountStorageMode, AccountType,
    },
    asset::{Asset, FungibleAsset, TokenSymbol},
    auth::AuthSecretKey,
    builder::ClientBuilder,
    crypto::{FeltRng, SecretKey},
    keystore::FilesystemKeyStore,
    note::{
        Note, NoteAssets, NoteExecutionHint, NoteExecutionMode, NoteInputs, NoteMetadata,
        NoteRecipient, NoteScript, NoteTag, NoteType,
    },
    rpc::{Endpoint, TonicRpcClient},
    transaction::{OutputNote, TransactionKernel, TransactionRequestBuilder},
    Client, ClientError, Felt,
};
use miden_objects::account::NetworkId;
use miden_client_tools::{
    create_basic_account, mint_from_faucet_for_account
};

async fn create_basic_faucet(
    client: &mut Client,
    keystore: FilesystemKeyStore<StdRng>,
) -> Result<miden_client::account::Account, ClientError> {
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);
    let key_pair = SecretKey::with_rng(client.rng());
    let anchor_block = client.get_latest_epoch_block().await.unwrap();
    let symbol = TokenSymbol::new("INH").unwrap();
    let decimals = 8;
    let max_supply = Felt::new(1_000_000);
    let builder = AccountBuilder::new(init_seed)
        .anchor((&anchor_block).try_into().unwrap())
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(AccountStorageMode::Public)
        .with_component(RpoFalcon512::new(key_pair.public_key()))
        .with_component(BasicFungibleFaucet::new(symbol, decimals, max_supply).unwrap());
    let (account, seed) = builder.build().unwrap();
    client.add_account(&account, Some(seed), false).await?;
    keystore
        .add_key(&AuthSecretKey::RpoFalcon512(key_pair))
        .unwrap();
    Ok(account)
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // Initialize client & keystore
    let endpoint = Endpoint::new("http".to_string(), "localhost".to_string(), Some(57291));
    let timeout_ms = 10_000;
    let rpc_api = Arc::new(TonicRpcClient::new(&endpoint, timeout_ms));

    let mut client = ClientBuilder::new()
        .with_rpc(rpc_api)
        .with_filesystem_keystore("./keystore")
        .in_debug_mode(true)
        .build()
        .await?;

    let sync_summary = client.sync_state().await.unwrap();
    println!("Connected to network, at block: {}", sync_summary.block_num);

    let keystore: FilesystemKeyStore<rand::prelude::StdRng> =
        FilesystemKeyStore::new("./keystore".into()).unwrap();

    Ok(())
}
```

### Step 8: Create accounts and deploy faucet

In this step, we will generate two Miden Accounts, the owner and the beneficiary. We will also deploy a faucet, which we are going to use to mint new "IHT" tokens. These tokens will be put into the vault for the inheritance claim.

Add the following code to the end of the `main()` function:

```rust
    // -------------------------------------------------------------------------
    // STEP 1: Create accounts and deploy faucet
    // -------------------------------------------------------------------------
    println!("\nCreating new accounts");
    let (owner_account, _) = create_basic_account(&mut client, keystore.clone()).await.unwrap();
    println!(
        "Owner's account ID: {:?}",
        owner_account.id().to_bech32(NetworkId::Testnet)
    );
    let (beneficiary_account, _) = create_basic_account(&mut client, keystore.clone()).await.unwrap();
    println!(
        "Beneficiary's account ID: {:?}",
        beneficiary_account.id().to_bech32(NetworkId::Testnet)
    );

    // -------------------------------------------------------------------------
    // STEP 2: Deploy faucet and mint IHT tokens for owner
    // -------------------------------------------------------------------------

    println!("\nDeploying a new fungible faucet.");
    let faucet = create_basic_faucet(&mut client, keystore.clone()).await.unwrap();
    println!(
        "Faucet account ID: {:?}",
        faucet.id().to_bech32(NetworkId::Testnet)
    );
    client.sync_state().await?;

    let mint_amount: u64 = 1000000;
    let _ = mint_from_faucet_for_account(&mut client, &owner_account, &faucet, mint_amount, None)
        .await
        .unwrap();
    println!("Minted {} tokens to owner using faucet", mint_amount);

    let sync_summary = client.sync_state().await.unwrap();
```

### Step 9: Deploy the vault

This step will be our most extensive one. Make sure to read the explanations carefully to understand the individual components properly.

First, we will use an assembler to compile our Note code. The assembler is a core component of the Miden VM that translates human-readable Miden Assembly code into executable bytecode that the VM can understand. For our inheritance vault, the assembler takes our Note's MASM code (which defines the inheritance logic) and converts it into a format that can be executed by Miden's virtual machine when the Note is consumed.

After that, we need to specify and compose the following to instantiate the Miden Note:

- Assets: Are the assets that will be locked inside of the Note, i.e. the "INH" tokens
- Metadata: Contains sender (owner), note type (public or private) and its reference Tag.
- Recipient: The recipient is not an account address, instead it is a value that describes when a note can be consumed. Because not all notes have predetermined consumer addresses, e.g. swap notes can be consumed by anyone, the recipient is defined as the code and its inputs, that when successfully executed results in the note's consumption. The address is derived by creating a digest of the following values:
  - Inputs: These are the input arguments for the Note, i.e. the beneficiary address (prefix & suffix) and the deadline block number
  - Serial Number: This is a randomly generated word. Each Note must have a unique serial number to prevent double spending.
  - Script: The compiled Note Script

After having composed all of these values using the Miden library helper functions, we are able to build and submit the transaction request to publish the note to the Miden network.

Add the following code to the end of the `main()` function:

```rust
    // set deadline to 5 blocks from current
    let deadline = sync_summary.block_num.as_u64() + 3;
    println!("Deadline: {}", deadline);

    // compile script
    let assembler = TransactionKernel::assembler().with_debug_mode(true);
    let note_code = fs::read_to_string(Path::new("masm/inheritance_vault_note.masm")).unwrap();
    let note_script = NoteScript::compile(note_code, assembler).unwrap();

    println!("Compiled note script!");

    let note_inputs = NoteInputs::new(vec![Felt::new(deadline), beneficiary_account.id().suffix(), beneficiary_account.id().prefix().as_felt()]).unwrap();
    let serial_num = client.rng().draw_word();
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);
    let tag = NoteTag::for_public_use_case(0, 0, NoteExecutionMode::Local).unwrap();
    let metadata = NoteMetadata::new(
        owner_account.id(),
        NoteType::Public,
        tag,
        NoteExecutionHint::always(),
        Felt::new(0),
    )?;
    let assets = NoteAssets::new(vec![Asset::Fungible(FungibleAsset::new(faucet.id(), 10).unwrap())]).unwrap();
    let inheritance_note = Note::new(assets, metadata, recipient);

    println!("Note ID: {:?}", inheritance_note.id().to_hex());

    // build and submit transaction
    let note_request = TransactionRequestBuilder::new()
        .with_own_output_notes(vec![OutputNote::Full(inheritance_note.clone())])
        .build()
        .unwrap();
    let tx_result = client
        .new_transaction(owner_account.id(), note_request)
        .await
        .unwrap();
    let _ = client.submit_transaction(tx_result.clone()).await;
    client.sync_state().await?;

    println!("Note submitted successfully! {:?} \n", tx_result.executed_transaction().id());
```

### Step 10: Consume the Note as the beneficiary

As our last step, we will consume and claim the inheritance funds as the beneficiary.

First we will wait 10 seconds to ensure that the deadline has passed. After that, we build and submit a transaction request to claim the inheritance funds (i.e. consuming the Note).

Add the following code to the end of the `main()` function:

```rust
    // wait 10 seconds to ensure deadline has passed
    sleep(Duration::from_secs(10)).await;

    println!("Consuming note as beneficiary");
    let consume_custom_request = TransactionRequestBuilder::new()
        .with_unauthenticated_input_notes([(inheritance_note, None)])
        .build()
        .unwrap();
    let tx_result = client
        .new_transaction(beneficiary_account.id(), consume_custom_request)
        .await
        .unwrap();
    let _ = client.submit_transaction(tx_result.clone()).await;
    client.sync_state().await?;

    println!(
        "Consumed Note Tx: {:?} \n",
        tx_result.executed_transaction().id()
    );
```

### Step 11: Run the final code

The final `src/main.rs` file should look like this:

```rust
use rand::{rngs::StdRng, RngCore};
use std::{fs, path::Path, sync::Arc};
use tokio::time::{sleep, Duration};

use miden_client::{
    account::{
        component::{BasicFungibleFaucet, RpoFalcon512},
        AccountBuilder, AccountStorageMode, AccountType,
    },
    asset::{Asset, FungibleAsset, TokenSymbol},
    auth::AuthSecretKey,
    builder::ClientBuilder,
    crypto::{FeltRng, SecretKey},
    keystore::FilesystemKeyStore,
    note::{
        Note, NoteAssets, NoteExecutionHint, NoteExecutionMode, NoteInputs, NoteMetadata,
        NoteRecipient, NoteScript, NoteTag, NoteType,
    },
    rpc::{Endpoint, TonicRpcClient},
    transaction::{OutputNote, TransactionKernel, TransactionRequestBuilder},
    Client, ClientError, Felt,
};
use miden_objects::account::NetworkId;
use miden_client_tools::{
    create_basic_account, mint_from_faucet_for_account
};

async fn create_basic_faucet(
    client: &mut Client,
    keystore: FilesystemKeyStore<StdRng>,
) -> Result<miden_client::account::Account, ClientError> {
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);
    let key_pair = SecretKey::with_rng(client.rng());
    let anchor_block = client.get_latest_epoch_block().await.unwrap();
    let symbol = TokenSymbol::new("INH").unwrap();
    let decimals = 8;
    let max_supply = Felt::new(1_000_000);
    let builder = AccountBuilder::new(init_seed)
        .anchor((&anchor_block).try_into().unwrap())
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(AccountStorageMode::Public)
        .with_component(RpoFalcon512::new(key_pair.public_key()))
        .with_component(BasicFungibleFaucet::new(symbol, decimals, max_supply).unwrap());
    let (account, seed) = builder.build().unwrap();
    client.add_account(&account, Some(seed), false).await?;
    keystore
        .add_key(&AuthSecretKey::RpoFalcon512(key_pair))
        .unwrap();
    Ok(account)
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // Initialize client & keystore
    let endpoint = Endpoint::new("http".to_string(), "localhost".to_string(), Some(57291));
    let timeout_ms = 10_000;
    let rpc_api = Arc::new(TonicRpcClient::new(&endpoint, timeout_ms));

    let mut client = ClientBuilder::new()
        .with_rpc(rpc_api)
        .with_filesystem_keystore("./keystore")
        .in_debug_mode(true)
        .build()
        .await?;

    let sync_summary = client.sync_state().await.unwrap();
    println!("Connected to network, at block: {}", sync_summary.block_num);

    let keystore = FilesystemKeyStore::new("./keystore".into()).unwrap();

    // -------------------------------------------------------------------------
    // STEP 1: Create accounts and deploy faucet
    // -------------------------------------------------------------------------
    println!("\nCreating new accounts");
    let (owner_account, _) = create_basic_account(&mut client, keystore.clone()).await.unwrap();
    println!(
        "Owner's account ID: {:?}",
        owner_account.id().to_bech32(NetworkId::Testnet)
    );
    let (beneficiary_account, _) = create_basic_account(&mut client, keystore.clone()).await.unwrap();
    println!(
        "Beneficiary's account ID: {:?}",
        beneficiary_account.id().to_bech32(NetworkId::Testnet)
    );

    // -------------------------------------------------------------------------
    // STEP 2: Deploy faucet and mint IHT tokens for owner
    // -------------------------------------------------------------------------

    println!("\nDeploying a new fungible faucet.");
    let faucet = create_basic_faucet(&mut client, keystore.clone()).await.unwrap();
    println!(
        "Faucet account ID: {:?}",
        faucet.id().to_bech32(NetworkId::Testnet)
    );
    client.sync_state().await?;

    let mint_amount: u64 = 1000000;
    let _ = mint_from_faucet_for_account(&mut client, &owner_account, &faucet, mint_amount, None)
        .await
        .unwrap();
    println!("Minted {} tokens to owner using faucet", mint_amount);

    let sync_summary = client.sync_state().await.unwrap();

    // -------------------------------------------------------------------------
    // STEP 3: Create custom note
    // -------------------------------------------------------------------------

    // set deadline to 5 blocks from current
    let deadline = sync_summary.block_num.as_u64() + 3;
    println!("Deadline: {}", deadline);

    // compile script
    let assembler = TransactionKernel::assembler().with_debug_mode(true);
    let note_code = fs::read_to_string(Path::new("masm/inheritance_vault_note.masm")).unwrap();
    let note_script = NoteScript::compile(note_code, assembler).unwrap();

    println!("Compiled note script!");

    let note_inputs = NoteInputs::new(vec![Felt::new(deadline), beneficiary_account.id().suffix(), beneficiary_account.id().prefix().as_felt()]).unwrap();
    let serial_num = client.rng().draw_word();
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);
    let tag = NoteTag::for_public_use_case(0, 0, NoteExecutionMode::Local).unwrap();
    let metadata = NoteMetadata::new(
        owner_account.id(),
        NoteType::Public,
        tag,
        NoteExecutionHint::always(),
        Felt::new(0),
    )?;
    let assets = NoteAssets::new(vec![Asset::Fungible(FungibleAsset::new(faucet.id(), 10).unwrap())]).unwrap();
    let inheritance_note = Note::new(assets, metadata, recipient);

    println!("Note ID: {:?}", inheritance_note.id().to_hex());

    // build and submit transaction
    let note_request = TransactionRequestBuilder::new()
        .with_own_output_notes(vec![OutputNote::Full(inheritance_note.clone())])
        .build()
        .unwrap();
    let tx_result = client
        .new_transaction(owner_account.id(), note_request)
        .await
        .unwrap();
    let _ = client.submit_transaction(tx_result.clone()).await;
    client.sync_state().await?;

    println!("Note submitted successfully! {:?} \n", tx_result.executed_transaction().id());

    // -------------------------------------------------------------------------
    // STEP 4: Consume the Custom Note (as beneficiary)
    // -------------------------------------------------------------------------

    // wait 10 seconds to ensure deadline has passed
    sleep(Duration::from_secs(10)).await;

    println!("Consuming note as beneficiary");
    let consume_custom_request = TransactionRequestBuilder::new()
        .with_unauthenticated_input_notes([(inheritance_note, None)])
        .build()
        .unwrap();
    let tx_result = client
        .new_transaction(beneficiary_account.id(), consume_custom_request)
        .await
        .unwrap();
    let _ = client.submit_transaction(tx_result.clone()).await;
    client.sync_state().await?;

    println!(
        "Consumed Note Tx: {:?} \n",
        tx_result.executed_transaction().id()
    );

    Ok(())
}
```

You can now run the code by using the following command:

```bash
cargo run --release
```

The output will look like this:

```bash
Connected to network, at block: 1955

Creating new accounts
Owner's account ID: "mtst1qzlnuyurz242yyqqqr924ezqkshs5p4w"
Beneficiary's account ID: "mtst1qq0g9xz36a3v5yqqqz2jrjm4rqf789js"

Deploying a new fungible faucet.
Faucet account ID: "mtst1qrduu0cyhv33sgqqqzjfxus5tvaxlru5"
Minted 1000000 tokens to owner using faucet
Deadline: 1959
Compiled note script!
Note ID: "0x442cd829d18e21532752bbbe4ddd7cb5f0d7e21bbbc9dfec7341d1b8db032aa6"
Note submitted successfully! 0x8d7fbef1ce8a5279a13f2a0c1e93a808fe1b6ffb081c236e91bd3b48f427fd7a

Consuming note as beneficiary
Consumed Note Tx: 0x19a8e6ce3800a00da904acdcbda59d7d448c1219b3ca7dcdd9482e9add8a79e8
```

## Conclusion

You've now built a complete trustless inheritance system using Miden Notes—solving one of cryptocurrency's most pressing problems without relying on any intermediaries. Your inheritance vault demonstrates the power of programmable money: assets that can be locked, unlocked, and transferred based on pure mathematical conditions.

You can be proud of yourself! 🎉
