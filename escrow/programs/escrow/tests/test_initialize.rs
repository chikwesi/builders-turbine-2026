use {
    anchor_lang::{
        system_program::ID as SYSTEM_PROGRAM_ID, AccountDeserialize, InstructionData,
        ToAccountMetas,
    },
    anchor_spl::{
        associated_token::{self, ID as ASSOCIATED_TOKEN_ACCOUNT_ID},
        token::spl_token,
    },
    escrow::{msg, program_pack::Pack},
    litesvm::LiteSVM,
    litesvm_token::{
        spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
    },
    solana_keypair::Keypair,
    solana_message::{Instruction, Message},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

fn setup() -> (LiteSVM, Keypair) {
    let program_id = escrow::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/escrow.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    (svm, payer)
}

#[test]
fn test_make_and_refund() {
    let (mut svm, payer) = setup();
    let maker = payer.pubkey();

    let mint_a = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint A: {}\n", mint_a);

    let mint_b = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint B: {}\n", mint_b);

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();
    msg!("Maker ATA A: {}\n", maker_ata_a);

    let escrow = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &123u64.to_le_bytes()],
        &escrow::id(),
    )
    .0;
    msg!("Escrow PDA: {}\n", escrow);

    let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
    msg!("Vault PDA: {}\n", vault);

    MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1_000_000_000)
        .send()
        .unwrap();

    let make_ix = Instruction {
        program_id: escrow::id(),
        accounts: escrow::accounts::Make {
            maker: maker,
            mint_a: mint_a,
            mint_b: mint_b,
            maker_ata_a: maker_ata_a,
            escrow: escrow,
            vault: vault,
            associated_token_program: ASSOCIATED_TOKEN_ACCOUNT_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrow::instruction::Make {
            deposit: 10_000_000,
            seed: 123u64,
            receive: 10_000_000,
        }
        .data(),
    };

    // Create and send the tx containing the Make instruction
    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    let tx = svm.send_transaction(transaction).unwrap();

    msg!("\n\nMake Transaction successful");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);
    msg!("Tx Signature: {}", tx.signature);

    let vault_account = svm.get_account(&vault).unwrap();
    let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();

    assert_eq!(vault_data.amount, 10_000_000);
    assert_eq!(vault_data.owner, escrow);
    assert_eq!(vault_data.mint, mint_a);

    let escrow_account = svm.get_account(&escrow).unwrap();
    let escrow_data =
        escrow::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
    assert_eq!(escrow_data.seed, 123u64);
    assert_eq!(escrow_data.maker, maker);
    assert_eq!(escrow_data.mint_a, mint_a);
    assert_eq!(escrow_data.mint_b, mint_b);
    assert_eq!(escrow_data.receive, 10_000_000);

    // Create the "Refund" instruction to refund tokens back to the maker

    let refund_ix = Instruction {
        program_id: escrow::id(),
        accounts: escrow::accounts::Refund {
            maker: maker,
            mint_a: mint_a,
            maker_ata_a: maker_ata_a,
            escrow: escrow,
            vault: vault,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrow::instruction::Refund {}.data(),
    };

    // Create and send the tx containing the Make instruction
    let message = Message::new(&[refund_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    let tx = svm.send_transaction(transaction).unwrap();

    msg!("\n\nRefund Transaction successful");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);
    msg!("Tx Signature: {}", tx.signature);
    assert!(svm.get_account(&escrow).is_none());
    assert!(svm.get_account(&vault).is_none());
}

#[test]
fn test_make_and_take() {
    let (mut svm, payer) = setup();
    let maker = payer.pubkey();
    let taker = Keypair::new();
    svm.airdrop(&taker.pubkey(), 1_000_000_000).unwrap();

    let mint_a = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint A: {}\n", mint_a);

    let mint_b = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint B: {}\n", mint_b);

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();
    msg!("Maker ATA A: {}\n", maker_ata_a);

    let maker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_b)
        .owner(&maker)
        .send()
        .unwrap();
    msg!("Maker ATA B: {}\n", maker_ata_b);

    let taker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &taker, &mint_a)
        .owner(&taker.pubkey())
        .send()
        .unwrap();
    msg!("Taker ATA A: {}\n", taker_ata_a);

    let taker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &taker, &mint_b)
        .owner(&taker.pubkey())
        .send()
        .unwrap();
    msg!("Taker ATA B: {}\n", taker_ata_b);

    let escrow = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &123u64.to_le_bytes()],
        &escrow::id(),
    )
    .0;
    msg!("Escrow PDA: {}\n", escrow);

    let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
    msg!("Vault PDA: {}\n", vault);

    MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1_000_000_000)
        .send()
        .unwrap();

    MintTo::new(&mut svm, &payer, &mint_b, &taker_ata_b, 1_000_000_000)
        .send()
        .unwrap();

    let make_ix = Instruction {
        program_id: escrow::id(),
        accounts: escrow::accounts::Make {
            maker: maker,
            mint_a: mint_a,
            mint_b: mint_b,
            maker_ata_a: maker_ata_a,
            escrow: escrow,
            vault: vault,
            associated_token_program: ASSOCIATED_TOKEN_ACCOUNT_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrow::instruction::Make {
            deposit: 10_000_000,
            seed: 123u64,
            receive: 10_000_000,
        }
        .data(),
    };

    // Create and send the tx containing the Make instruction
    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    let tx = svm.send_transaction(transaction).unwrap();

    msg!("\n\nMake Transaction successful");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);
    msg!("Tx Signature: {}", tx.signature);

    let vault_account = svm.get_account(&vault).unwrap();
    let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
    assert_eq!(vault_data.amount, 10_000_000);
    assert_eq!(vault_data.owner, escrow);
    assert_eq!(vault_data.mint, mint_a);

    let escrow_account = svm.get_account(&escrow).unwrap();
    let escrow_data =
        escrow::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
    assert_eq!(escrow_data.seed, 123u64);
    assert_eq!(escrow_data.maker, maker);
    assert_eq!(escrow_data.mint_a, mint_a);
    assert_eq!(escrow_data.mint_b, mint_b);
    assert_eq!(escrow_data.receive, 10_000_000);

    // Create the take instruction to execute the trade and close escrow
    let take_ix = Instruction {
        program_id: escrow::id(),
        accounts: escrow::accounts::Take {
            taker: taker.pubkey(),
            maker: maker,
            mint_a: mint_a,
            mint_b: mint_b,
            taker_ata_a: taker_ata_a,
            taker_ata_b: taker_ata_b,
            maker_ata_b: maker_ata_b,
            escrow: escrow,
            vault: vault,
            associated_token_program: ASSOCIATED_TOKEN_ACCOUNT_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrow::instruction::Take {}.data(),
    };

    // Create and send the tx containing the Make instruction
    let message = Message::new(&[take_ix], Some(&taker.pubkey()));
    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new(&[&taker], message, recent_blockhash);

    let tx = svm.send_transaction(transaction).unwrap();

    msg!("\n\nTake Transaction successful");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);
    msg!("Tx Signature: {}", tx.signature);
    assert!(svm.get_account(&escrow).is_none());
    assert!(svm.get_account(&vault).is_none());
}
