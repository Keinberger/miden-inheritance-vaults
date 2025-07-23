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
    let note_code = fs::read_to_string(Path::new("masm /inheritance_vault_note.masm")).unwrap();
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