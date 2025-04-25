use bdk::{bitcoin, FeeRate, SyncOptions, Wallet};
use bdk::database::MemoryDatabase;
use bdk::blockchain::ElectrumBlockchain;
use bdk::electrum_client::Client;
use bdk::wallet::AddressIndex;

fn main() -> Result<(), bdk::Error> {
    println!("Creating a simple BDK wallet application...");
    
    let client = Client::new("ssl://electrum.blockstream.info:60002")?;
    let blockchain = ElectrumBlockchain::from(client);
    
    // create wallet using descriptor strings
    // a test descriptor is used
    let wallet = Wallet::new(
        "wpkh([c258d2e4/84h/1h/0h]tpubDDYkZojQFQjht8Tm4jsS3iuEmKjTiEGjG6KnuFNKKJb5A6ZUCUZKdvLdSDWofKi4ToRCwb9poe1XdqfUnP4jaJjCB2Zwv11ZLgSbnZSNecE/0/*)",
        Some("wpkh([c258d2e4/84h/1h/0h]tpubDDYkZojQFQjht8Tm4jsS3iuEmKjTiEGjG6KnuFNKKJb5A6ZUCUZKdvLdSDWofKi4ToRCwb9poe1XdqfUnP4jaJjCB2Zwv11ZLgSbnZSNecE/1/*)"),
        bitcoin::Network::Testnet,
        MemoryDatabase::default(),
    )?;
    
    // sync wallet with blockchain
    println!("Syncing wallet with blockchain...");
    wallet.sync(&blockchain, SyncOptions::default())?;
    
    // check wallet balance
    let balance = wallet.get_balance()?;
    println!("Wallet balance: {} SAT (confirmed: {}, unconfirmed: {})", 
             balance.get_total(), balance.confirmed, balance.untrusted_pending);
    
    // generate new addresses
    println!("\nGenerating new addresses:");
    println!("Address #0: {}", wallet.get_address(AddressIndex::New)?);
    println!("Address #1: {}", wallet.get_address(AddressIndex::New)?);
    println!("Address #2: {}", wallet.get_address(AddressIndex::New)?);
    
    // list unspent UTXOs
    let utxos = wallet.list_unspent()?;
    println!("\nUTXO count: {}", utxos.len());
    for (i, utxo) in utxos.iter().enumerate() {
        println!("UTXO #{}: {} SAT, txid: {}", i, utxo.txout.value, utxo.outpoint.txid);
    }
    
    // create transaction
    if balance.confirmed > 0 {
        println!("\nCreating a transaction:");
        let send_to = wallet.get_address(AddressIndex::New)?;
        let (psbt, details) = {
            let mut builder = wallet.build_tx();
            builder
                .add_recipient(send_to.script_pubkey(), 10_000)
                .enable_rbf()
                .do_not_spend_change()
                .fee_rate(FeeRate::from_sat_per_vb(5.0));
            builder.finish()?
        };

        println!("Transaction recipient: {}", send_to);
        println!("Transaction details: {:#?}", details);
        println!("Unsigned PSBT: {}", &psbt);
        
        // Sign transaction
        println!("\nSigning transaction...");
        let mut psbt_to_sign = psbt.clone();
        let finalized = wallet.sign(&mut psbt_to_sign, Default::default())?;
        
        if finalized {
            println!("Transaction fully signed and ready to broadcast!");
        } else {
            println!("Transaction partially signed.");
        }
        
        // broadcast transaction
        // blockchain.broadcast(&psbt_to_sign.extract_tx())?;
    } else {
        println!("\nWallet has no confirmed balance. Cannot create a transaction.");
    }
    
    Ok(())
}