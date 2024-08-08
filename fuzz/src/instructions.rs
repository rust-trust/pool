
    system_instruction::create_account,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use {
    arbitrary::{Arbitrary, Result as ArbResult, Unstructured},
    honggfuzz::fuzz,
};

use arrayvec::ArrayVec;
/// Use u8 as an account id to simplify the address space and re-use accounts
/// more often.
type AccountId = u8;

type AmountT = u64;
type DecT = DecimalU64;

pub struct PoolInfo<const TOKEN_COUNT: usize> {
    pub pool_keypair: Keypair,
    pub nonce: u8,
    pub authority: Pubkey,
    pub lp_mint_keypair: Keypair,
    pub token_mint_keypairs: [Keypair; TOKEN_COUNT],
    pub token_account_keypairs: [Keypair; TOKEN_COUNT],
    pub governance_keypair: Keypair,
    pub governance_fee_keypair: Keypair,
}

impl<const TOKEN_COUNT: usize> PoolInfo<TOKEN_COUNT> {
    pub fn new() -> Self {
        let pool_keypair = Keypair::new();
        let lp_mint_keypair = Keypair::new();
        let (authority, nonce) = Pubkey::find_program_address(&[&pool_keypair.pubkey().to_bytes()[..32]], &pool::id());
        let mut token_mint_arrayvec = ArrayVec::<_, TOKEN_COUNT>::new();
        let mut token_account_arrayvec = ArrayVec::<_, TOKEN_COUNT>::new();
        for _i in 0..TOKEN_COUNT {
            token_mint_arrayvec.push(Keypair::new());
            token_account_arrayvec.push(Keypair::new());
        }
        let token_mint_keypairs: [Keypair; TOKEN_COUNT] = token_mint_arrayvec.into_inner().unwrap();
        let token_account_keypairs: [Keypair; TOKEN_COUNT] = token_account_arrayvec.into_inner().unwrap();
        let governance_keypair = Keypair::new();
        let governance_fee_keypair = Keypair::new();

        Self {
            pool_keypair,
            nonce,
            authority,
            lp_mint_keypair,
            token_mint_keypairs,
            token_account_keypairs,
            governance_keypair,
            governance_fee_keypair,
        }
    }

    pub fn get_token_mint_pubkeys(&self) -> [Pubkey; TOKEN_COUNT] {
        Self::to_key_array(&self.token_mint_keypairs)
    }

    pub fn get_token_account_pubkeys(&self) -> [Pubkey; TOKEN_COUNT] {
        Self::to_key_array(&self.token_account_keypairs)
    }

    pub async fn get_token_account_balances(&self, banks_client: &mut BanksClient) -> [AmountT; TOKEN_COUNT] {
        let token_account_pubkeys = self.get_token_account_pubkeys();
        get_token_balances(banks_client, token_account_pubkeys).await
    }

    fn to_key_array(account_slice: &[Keypair; TOKEN_COUNT]) -> [Pubkey; TOKEN_COUNT] {
        account_slice
            .iter()
            .map(|account| account.pubkey())
            .collect::<ArrayVec<_, TOKEN_COUNT>>()
            .into_inner()
            .unwrap()
    }

    /// Creates pool's token mint accounts and token accounts
    /// for all tokens and LP token
    pub async fn init_pool(
        &self,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        user_accounts_owner: &Keypair,
        lp_fee: DecT,
        governance_fee: DecT,
    ) {
        let rent = banks_client.get_rent().await.unwrap();

        let token_mint_pubkeys = *(&self.get_token_mint_pubkeys());
        let token_account_pubkeys = *(&self.get_token_account_pubkeys());

        let pool_len = solana_program::borsh::get_packed_len::<pool::state::PoolState<TOKEN_COUNT>>();
        let mut ixs_vec = vec![
            create_account(
                &payer.pubkey(),
                &self.pool_keypair.pubkey(),
                rent.minimum_balance(pool_len),
                pool_len as u64,
                &pool::id(),
            ),
            // Create LP Mint account
            create_account(
                &payer.pubkey(),
                &self.lp_mint_keypair.pubkey(),
                rent.minimum_balance(Mint::LEN),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &self.lp_mint_keypair.pubkey(),
                &self.authority,
                None,
            )
            .unwrap(),
        ];
        // create token mints and Token accounts
        for i in 0..TOKEN_COUNT {
            println!("adding create_account & initialize_mint ix for {}", i);
            ixs_vec.push(create_account(
                &payer.pubkey(),
                &token_mint_pubkeys[i],
                //&token_mint_keypairs[i],
                rent.minimum_balance(Mint::LEN),
                Mint::LEN as u64,
                &spl_token::id(),
            ));
            ixs_vec.push(
                spl_token::instruction::initialize_mint(
                    &spl_token::id(),
                    &token_mint_pubkeys[i],
                    &user_accounts_owner.pubkey(),
                    None,
                )
                .unwrap(),
            );
        }
        for i in 0..TOKEN_COUNT {
            println!("adding create_account & initialize_account ix for {}", i);
            ixs_vec.push(create_account(
                &payer.pubkey(),
                &token_account_pubkeys[i],
                //&token_account_keypairs[i],
                rent.minimum_balance(Token::LEN),
                Token::LEN as u64,
                &spl_token::id(),
            ));
            ixs_vec.push(
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &token_account_pubkeys[i],
                    &token_mint_pubkeys[i],
                    &self.authority,
                )
                .unwrap(),
            );
        }

        ixs_vec.push(create_account(
            &payer.pubkey(),
            &self.governance_keypair.pubkey(),
            rent.minimum_balance(Token::LEN), //TODO: not sure what the len of this should be? data would just be empty?
            Token::LEN as u64,
            &user_accounts_owner.pubkey(), //TODO: randomly assigned owner to the user account owner
        ));
        ixs_vec.push(create_account(
            &payer.pubkey(),
            &self.governance_fee_keypair.pubkey(),
            rent.minimum_balance(Token::LEN),
            Token::LEN as u64,
            &spl_token::id(),
        ));
        ixs_vec.push(
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &self.governance_fee_keypair.pubkey(),
                &self.lp_mint_keypair.pubkey(),
                &user_accounts_owner.pubkey(), //TODO: randomly assigned governance_fee token account owner to the user account owner,
            )
            .unwrap(),
        );
        ixs_vec.push(
            create_init_ix::<TOKEN_COUNT>(
                &pool::id(),
                &self.pool_keypair.pubkey(),
                &self.lp_mint_keypair.pubkey(),
                &self.governance_keypair.pubkey(),
                &self.governance_fee_keypair.pubkey(),
                self.nonce,
                amp_factor,
                lp_fee,
                governance_fee,
            )
            .unwrap(),
        );

        let mut transaction = Transaction::new_with_payer(&ixs_vec, Some(&payer.pubkey()));
        let recent_blockhash = banks_client.get_recent_blockhash().await.unwrap();
        let mut signatures = vec![
            payer,
            &self.pool_keypair,
            //user_accounts_owner,
            &self.lp_mint_keypair,
        ];

        for i in 0..TOKEN_COUNT {
            signatures.push(&self.token_mint_keypairs[i]);
        }
        for i in 0..TOKEN_COUNT {
            signatures.push(&self.token_account_keypairs[i]);
        }

        signatures.push(&self.governance_keypair);
        signatures.push(&self.governance_fee_keypair);

        transaction.sign(&signatures, recent_blockhash);

        banks_client.process_transaction(transaction).await.unwrap();
    }

    pub async fn execute_add(
        &self,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        user_accounts_owner: &Keypair,
        user_transfer_authority: &Keypair,
        user_token_accounts: [Pubkey; TOKEN_COUNT],
        token_program_account: &Pubkey,
        user_lp_token_account: &Pubkey,
        deposit_amounts: [AmountT; TOKEN_COUNT],
        minimum_amount: AmountT,
    ) {
            )
            .unwrap()],
            Some(&payer.pubkey()),
        );
        let recent_blockhash = banks_client.get_recent_blockhash().await.unwrap();
        transaction.sign(&[payer, user_transfer_authority], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }
}
pub struct FuzzInstruction<const TOKEN_COUNT: usize> {
    instruction: DeFiInstruction<TOKEN_COUNT>,
    user_acct_id: AccountId,
}
        })
    }
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    loop {
            let mut program_test =
                ProgramTest::new("pool", pool::id(), processor!(Processor::<{ TOKEN_COUNT }>::process));

            program_test.set_bpf_compute_max_units(200_000);

            let mut test_state = rt.block_on(program_test.start_with_context());
            rt.block_on(run_fuzz_instructions(
                &mut test_state.banks_client,
                test_state.payer,
                test_state.last_blockhash,
async fn run_fuzz_instructions<const TOKEN_COUNT: usize>(
    banks_client: &mut BanksClient,
    correct_payer: Keypair,
    recent_blockhash: Hash,
    let governance_fee = DecimalU64::new(1000, 5).unwrap();
    let user_accounts_owner = Keypair::new();
    let user_transfer_authority = Keypair::new();
    let pool = PoolInfo::<{ TOKEN_COUNT }>::new();

    //creates pool's token mints & token accounts
    pool.init_pool(
        banks_client,
        &correct_payer,
        &user_accounts_owner,
        amp_factor,
        lp_fee,
        governance_fee,
    )
    .await;

    // need to do initial add from a user's token accounts
    // TODO: focus on just executing the fuzz_ixs then worry about how to handle validations

    let mut init_prep_add_ixs = vec![];
    // create user token accounts that will do initial add
    for token_idx in 0..TOKEN_COUNT {
        let token_mint_keypair = &pool.token_mint_keypairs[token_idx];
        init_prep_add_ixs.push(create_associated_token_account(
            &correct_payer.pubkey(),
            &user_accounts_owner.pubkey(),
            &token_mint_keypair.pubkey(),
        ));
    }
    init_prep_add_ixs.push(create_associated_token_account(
        &correct_payer.pubkey(),
        &user_accounts_owner.pubkey(),
        &pool.lp_mint_keypair.pubkey(),
    ));
    println!("[DEV] finished setting up ixs for user ATAs");
    let mut transaction = Transaction::new_with_payer(&init_prep_add_ixs, Some(&correct_payer.pubkey()));
    transaction.sign(&[&correct_payer], recent_blockhash);
    let result = banks_client.process_transaction(transaction).await;
    println!("[DEV] finished creating ATA. Result: {:?}", result);
    //mint inital token amounts to user token accounts
    let mut init_user_token_accounts: [Pubkey; TOKEN_COUNT] = [Pubkey::new_unique(); TOKEN_COUNT];
    for token_idx in 0..TOKEN_COUNT {
        let token_mint_keypair = &pool.token_mint_keypairs[token_idx];
        let user_token_pubkey =
            get_associated_token_address(&user_accounts_owner.pubkey(), &token_mint_keypair.pubkey());
        init_user_token_accounts[token_idx] = user_token_pubkey;
        mint_tokens_to(
            banks_client,
            &correct_payer,
            &recent_blockhash,
            &token_mint_keypair.pubkey(),
            &user_token_pubkey,
            &user_accounts_owner,
        )
        .await
        .unwrap();

        approve_delegate(
            banks_client,
            &correct_payer,
            &recent_blockhash,
            &user_token_pubkey,
            &user_transfer_authority.pubkey(),
            &user_accounts_owner,
    pool.execute_add(
        banks_client,
        &correct_payer,
        &user_accounts_owner,
        &user_transfer_authority,
        init_user_token_accounts,
        &spl_token::id(),
        &user_lp_token_account,
        deposit_amounts,
        0,
    )
    .await;

    let pool_token_account_balances = pool.get_token_account_balances(banks_client).await;
    println!("[DEV] pool_token_account_balances: {:?}", pool_token_account_balances);
    //Map<user_wallet_key>, associated_token_account_pubkey
    let mut user_token_accounts: HashMap<usize, HashMap<AccountId, Pubkey>> = HashMap::new();
    let mut user_lp_token_accounts: HashMap<AccountId, Pubkey> = HashMap::new();
    for token_idx in 0..TOKEN_COUNT {
        user_token_accounts.insert(token_idx, HashMap::new());
    }
    //[HashMap<AccountId, Pubkey>; TOKEN_COUNT] = [HashMap::new(); TOKEN_COUNT];

        let user_wallet_keypair = user_wallets.get(&user_id).unwrap();
        for token_idx in 0..TOKEN_COUNT {
            let token_mint_keypair = &pool.token_mint_keypairs[token_idx];
            if !user_token_accounts[&token_idx].contains_key(&user_id) {
                let user_ata_pubkey = create_assoc_token_acct_and_mint(
                    banks_client,
                    &correct_payer,
                    recent_blockhash,
                    &user_accounts_owner,
                    &user_wallet_keypair.pubkey(),
                    &token_mint_keypair.pubkey(),
                )
                .await
                .unwrap();
                user_token_accounts
                    .get_mut(&token_idx)
                    .unwrap()
                    .insert(user_id, user_ata_pubkey);
            }
        }

        // create user ATA for LP Token
        .iter()
        .map(|&v| v) // deref &Keypair
        .chain(global_signer_keys.iter())
        .collect::<Vec<&Keypair>>();
        DeFiInstruction::Add {
            input_amounts,
            minimum_mint_amount,
        } => {
            let mut ix_vec = vec![];
            (ix_vec, kp_vec)
        }
        DeFiInstruction::SwapExactInput {
            exact_input_amounts,
            output_token_index,
            minimum_output_amount,
        } => {
            (ix_vec, kp_vec)
        }
        DeFiInstruction::SwapExactOutput {
            maximum_input_amount,
            input_token_index,
            exact_output_amounts,
        } => {
            (ix_vec, kp_vec)
        }
        DeFiInstruction::RemoveUniform {
            exact_burn_amount,
            minimum_output_amounts,
        } => {
            (ix_vec, kp_vec)
        }
        DeFiInstruction::RemoveExactBurn {
            exact_burn_amount,
            output_token_index,
            minimum_output_amount,
        } => {
            (ix_vec, kp_vec)
        }
        DeFiInstruction::RemoveExactOutput {
            maximum_burn_amount,
            exact_output_amounts,
        } => {
            let ix_vec = vec![];
            let kp_vec = vec![];
            (ix_vec, kp_vec)
        }
    }
}

/** Helper fns  **/
pub fn get_user_token_accounts<const TOKEN_COUNT: usize>(
    user_acct_id: AccountId,
    user_token_accounts: &HashMap<usize, HashMap<AccountId, Pubkey>>,
) -> [Pubkey; TOKEN_COUNT] {
    let mut user_token_accts_arrvec = ArrayVec::<_, TOKEN_COUNT>::new();
    for token_idx in 0..TOKEN_COUNT {
        let user_token_account = user_token_accounts.get(&token_idx).unwrap().get(&user_acct_id).unwrap();
        user_token_accts_arrvec.push(*user_token_account);
    }
    user_token_accts_arrvec.into_inner().unwrap()
}

/// Creates an associated token account and mints
/// `amount` for a user
pub async fn create_assoc_token_acct_and_mint(
    banks_client: &mut BanksClient,
    correct_payer: &Keypair,
    recent_blockhash: Hash,
    mint_authority: &Keypair,
    user_wallet_pubkey: &Pubkey,
    token_mint: &Pubkey,
    amount: u64,
) -> Result<Pubkey, TransportError> {
    let create_ix = create_associated_token_account(&correct_payer.pubkey(), user_wallet_pubkey, token_mint);
    let ixs = vec![create_ix];
    let mut transaction = Transaction::new_with_payer(&ixs, Some(&correct_payer.pubkey()));
    transaction.sign(&[correct_payer], recent_blockhash);

    let user_token_pubkey = get_associated_token_address(user_wallet_pubkey, token_mint);
    if amount > 0 {
        mint_tokens_to(
            banks_client,
            &correct_payer,
            &recent_blockhash,
            token_mint,
            &user_token_pubkey,
            mint_authority,
            amount,
        )
        .await
        .unwrap();
    }
    Ok(user_token_pubkey)
}
/// Creates and initializes a token account
pub async fn create_token_account(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    account: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(spl_token::state::Account::LEN);

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &account.pubkey(),
                account_rent,
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(&spl_token::id(), &account.pubkey(), mint, owner).unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, account], *recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

pub async fn mint_tokens_to(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Keypair,
    amount: u64,
) -> Result<(), TransportError> {
    let mut transaction = Transaction::new_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            destination,
            &authority.pubkey(),
            &[&authority.pubkey()],
            amount,
        )
        .unwrap()],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, authority], *recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

pub async fn approve_delegate(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    source: &Pubkey,
    delegate: &Pubkey,
    source_owner: &Keypair,
    amount: u64,
) -> Result<(), TransportError> {
    let mut transaction = Transaction::new_with_payer(
        &[spl_token::instruction::approve(
            &spl_token::id(),
            source,
            delegate,
            &source_owner.pubkey(),
            &[&source_owner.pubkey()],
            amount,
        )
        .unwrap()],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, source_owner], *recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    Ok(())
}

pub async fn get_account(banks_client: &mut BanksClient, pubkey: &Pubkey) -> Account {
    banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}

pub async fn get_mint_state(banks_client: &mut BanksClient, pubkey: &Pubkey) -> Mint {
    let acct = get_account(banks_client, pubkey).await;
    Mint::unpack_from_slice(acct.data.as_slice()).unwrap()
}

pub async fn get_token_balance(banks_client: &mut BanksClient, token_account_pubkey: Pubkey) -> u64 {
    let token_account = get_account(banks_client, &token_account_pubkey).await;
    let account_info = Token::unpack_from_slice(token_account.data.as_slice()).unwrap();
    account_info.amount
}

pub async fn get_token_balances<const TOKEN_COUNT: usize>(
    banks_client: &mut BanksClient,
    token_accounts: [Pubkey; TOKEN_COUNT],
) -> [AmountT; TOKEN_COUNT] {
    let mut token_accounts_arrvec = ArrayVec::<_, TOKEN_COUNT>::new();
    for i in 0..TOKEN_COUNT {
        token_accounts_arrvec.push(get_token_balance(banks_client, token_accounts[i]).await);
    }
    token_accounts_arrvec.into_inner().unwrap()
}

pub async fn get_token_balances_map<const TOKEN_COUNT: usize>(
    banks_client: &mut BanksClient,
    token_accounts: [Pubkey; TOKEN_COUNT],
) -> BTreeMap<Pubkey, u64> {
    let mut btree = BTreeMap::<Pubkey, u64>::new();
    for i in 0..TOKEN_COUNT {
        let token_account = get_account(banks_client, &token_accounts[i]).await;
        let account_info = Token::unpack_from_slice(token_account.data.as_slice()).unwrap();
        btree.insert(token_accounts[i], account_info.amount);
    }
    btree
}

pub async fn print_user_token_account_owners<const TOKEN_COUNT: usize>(
    banks_client: &mut BanksClient,
    token_accounts: [Pubkey; TOKEN_COUNT],
) {
    for i in 0..TOKEN_COUNT {
        let token_account = get_account(banks_client, &token_accounts[i]).await;
        let spl_token_account_info = Token::unpack_from_slice(token_account.data.as_slice()).unwrap();
        println!(
            "token_account.key: {} token_account.owner: {} spl_token_account_info.owner: {}",
            &token_accounts[i], token_account.owner, spl_token_account_info.owner
        );
    }
}

fn clone_keypair(keypair: &Keypair) -> Keypair {
    return Keypair::from_bytes(&keypair.to_bytes().clone()).unwrap();
}
