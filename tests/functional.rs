#![cfg(feature = "test-bpf")]

mod helpers;

use helpers::*;

use solana_program_test::*;
use solana_sdk::signature::{Keypair, Signer};
use std::time::{SystemTime, UNIX_EPOCH};

struct Parameters {
    amp_factor: DecT,
    lp_fee: DecT,
    governance_fee: DecT,
    lp_decimals: u8,
    stable_decimals: [u8; TOKEN_COUNT],
    pool_balances: [AmountT; TOKEN_COUNT],
    user_funds: [AmountT; TOKEN_COUNT],
}
    }
}

struct User {
    lp: TokenAccount,
    stables: [TokenAccount; TOKEN_COUNT],
}

    }

    fn stable_approve(&self, amounts: &[AmountT; TOKEN_COUNT], solnode: &mut SolanaNode) {
        for i in 0..TOKEN_COUNT {
            self.stables[i].approve(amounts[i], solnode);
        }
    }

    fn stable_balances(&self, solnode: &mut SolanaNode) -> [AmountT; TOKEN_COUNT] {
        let mut balances = [0; TOKEN_COUNT];
        for i in 0..TOKEN_COUNT {
            balances[i] = self.stables[i].balance(solnode);
        }
        balances
    }
}
fn setup_standard_testcase(params: &Parameters) -> (SolanaNode, DeployedPool, User, User) {
    let mut solnode = SolanaNode::new();
    let stable_mints: [_; TOKEN_COUNT] = create_array(|i| MintAccount::new(params.stable_decimals[i], &mut solnode));
    solnode.execute_transaction().expect("transaction failed unexpectedly");

    let pool = DeployedPool::new(
        params.lp_decimals,
        &stable_mints,
        params.amp_factor,
        params.lp_fee,
        params.governance_fee,
        &mut solnode,
    )
    .unwrap();
    let user = User::new(&params.user_funds, &stable_mints, &pool, &mut solnode);
    let lp_collective = User::new(&params.pool_balances, &stable_mints, &pool, &mut solnode);
    lp_collective.stable_approve(&params.pool_balances, &mut solnode);
    let defi_ix = DeFiInstruction::<TOKEN_COUNT>::Add {
        input_amounts: params.pool_balances,
        minimum_mint_amount: 0 as AmountT,
    };
    pool.execute_defi_instruction(defi_ix, &lp_collective.stables, Some(&lp_collective.lp), &mut solnode)
        .unwrap();

    (solnode, pool, user, lp_collective)
}

        )
        .unwrap();

        assert_eq!(pool.balances(&mut solnode), [0; TOKEN_COUNT]);
    }

        println!(
            "lp_collective stable balance: {:?}",
            lp_collective.stable_balances(&mut solnode)
        );
        println!("user lp balance: {}", user.lp.balance(&mut solnode));
        println!("user stable balance: {:?}", user.stable_balances(&mut solnode));
    }

    #[test]
    fn test_pool_swap_exact_input() {
        let params = default_params();
        let (mut solnode, pool, user, _) = setup_standard_testcase(&params);
        let exact_input_amounts = create_array(|i| i as u64 * params.user_funds[i] / 10);

        user.stable_approve(&exact_input_amounts, &mut solnode);
        let defi_ix = DeFiInstruction::SwapExactInput {
            exact_input_amounts,
            output_token_index: 0,
            minimum_output_amount: 0 as AmountT,
        };

        let lp_supply_before = pool.lp_total_supply(&mut solnode);
        let depth_before = pool.state(&mut solnode).previous_depth;
        println!("> user balance before: {:?}", user.stable_balances(&mut solnode));
        pool.execute_defi_instruction(defi_ix, &user.stables, None, &mut solnode)
            .unwrap();

        let depth_after = pool.state(&mut solnode).previous_depth;
        let lp_supply_after = pool.lp_total_supply(&mut solnode);
        // lp_share/lp_supply_before * depth_before <= lp_share/lp_supply_after * depth_after
        //  a. "your share of the depth of the pool must never decrease"
        //  b. if lp_fee == 0 then your share should be the same otherwise it should increase
        if params.lp_fee + params.governance_fee == 0 {
            assert_eq!(
                depth_before,
                (depth_after * lp_supply_before as u128) / lp_supply_after as u128
            );
        } else {
            assert!(depth_before <= (depth_after * lp_supply_before as u128) / lp_supply_after as u128);
        }

        println!(">  user balance after: {:?}", user.stable_balances(&mut solnode));
    }

    }

    }

    #[test]
    fn test_expensive_add() {
        let scale_factor = (10 as AmountT).pow(9);
        let initial_balances: [AmountT; TOKEN_COUNT] =
            [5_590_413, 6_341_331, 4_947_048, 3_226_825, 2_560_56724, 3_339_50641];

        let initial_balances: [_; TOKEN_COUNT] = create_array(|i| initial_balances[i] * scale_factor);

        let user_add: [AmountT; TOKEN_COUNT] = [
            10_000_000,
            9_000_000,
            11_000_000,
            12_000_000,
            13_000_00000,
            12_000_00000,
        ];

        let user_add: [_; TOKEN_COUNT] = create_array(|i| user_add[i] * scale_factor);

        let params = Parameters {
            amp_factor: DecT::new(1000, 0).unwrap(),
            lp_fee: DecT::new(3, 6).unwrap(),
            governance_fee: DecT::new(1, 6).unwrap(),
            lp_decimals: 6,
            stable_decimals: create_array(|i| if i < 4 { 6 } else { 8 }),
            pool_balances: create_array(|i| initial_balances[i]),
            user_funds: create_array(|i| user_add[i]),
        };

        let (mut solnode, pool, user, _) = setup_standard_testcase(&params);

        user.stable_approve(&params.user_funds, &mut solnode);
        let defi_ix = DeFiInstruction::Add {
            input_amounts: params.user_funds,
            minimum_mint_amount: 0 as AmountT,
        };
        println!("> user balance before: {:?}", user.stable_balances(&mut solnode));
        pool.execute_defi_instruction(defi_ix, &user.stables, Some(&user.lp), &mut solnode)
            .unwrap();
        println!(">       user lp after: {:?}", user.lp.balance(&mut solnode));
    }
    }

    #[test]
    fn test_change_governance_fee_account() {
        let initial_balances: [AmountT; TOKEN_COUNT] =
            [5_590_413, 6_341_331, 4_947_048, 3_226_825, 2_560_56724, 3_339_50641];

        let user_add: [AmountT; TOKEN_COUNT] = [
            10_000_000,
            9_000_000,
            11_000_000,
            12_000_000,
            13_000_00000,
            12_000_00000,
        ];

        let params = Parameters {
            amp_factor: DecT::new(1000, 0).unwrap(),
            lp_fee: DecT::new(3, 6).unwrap(),
            governance_fee: DecT::new(1, 6).unwrap(),
            lp_decimals: 6,
            stable_decimals: create_array(|i| if i < 4 { 6 } else { 8 }),
            pool_balances: create_array(|i| initial_balances[i]),
            user_funds: create_array(|i| user_add[i]),
        };

        let (mut solnode, pool, ..) = setup_standard_testcase(&params);

        let new_gov_fee_token_account = pool.create_lp_account(&mut solnode);

        let gov_ix = GovernanceInstruction::ChangeGovernanceFeeAccount {
            governance_fee_key: *new_gov_fee_token_account.pubkey(),
        };
        pool.execute_governance_instruction(gov_ix, Some(new_gov_fee_token_account.pubkey()), &mut solnode)
            .unwrap();

        let updated_state = pool.state(&mut solnode);
        assert_eq!(updated_state.governance_fee_key, *new_gov_fee_token_account.pubkey());
    }

    #[test]
    fn test_adjust_amp_factor() {
        let initial_balances: [AmountT; TOKEN_COUNT] =
            [5_590_413, 6_341_331, 4_947_048, 3_226_825, 2_560_56724, 3_339_50641];

        let user_add: [AmountT; TOKEN_COUNT] = [
            10_000_000,
            9_000_000,
            11_000_000,
            12_000_000,
            13_000_00000,
            12_000_00000,
        ];

        let params = Parameters {
            amp_factor: DecT::new(1000, 0).unwrap(),
            lp_fee: DecT::new(3, 6).unwrap(),
            governance_fee: DecT::new(1, 6).unwrap(),
            lp_decimals: 6,
            stable_decimals: create_array(|i| if i < 4 { 6 } else { 8 }),
            pool_balances: create_array(|i| initial_balances[i]),
            user_funds: create_array(|i| user_add[i]),
        };

        let (mut solnode, pool, ..) = setup_standard_testcase(&params);

        let curr_ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let target_ts = curr_ts + (10 * pool::amp_factor::MIN_ADJUSTMENT_WINDOW);
        let target_value = DecT::new(1010, 0).unwrap();
        let gov_ix = GovernanceInstruction::AdjustAmpFactor {
            target_ts,
            target_value,
        };
        pool.execute_governance_instruction(gov_ix, None, &mut solnode).unwrap();

        let updated_state = pool.state(&mut solnode);
        assert_eq!(updated_state.amp_factor.get(target_ts + 100), target_value);
    }

    #[test]
    fn test_pause() {
        let initial_balances: [AmountT; TOKEN_COUNT] =
            [5_590_413, 6_341_331, 4_947_048, 3_226_825, 2_560_56724, 3_339_50641];

        let user_add: [AmountT; TOKEN_COUNT] = [
            10_000_000,
            9_000_000,
            11_000_000,
            12_000_000,
            13_000_00000,
            12_000_00000,
        ];

        let params = Parameters {
            amp_factor: DecT::new(1000, 0).unwrap(),
            lp_fee: DecT::new(3, 6).unwrap(),
            governance_fee: DecT::new(1, 6).unwrap(),
            lp_decimals: 6,
            stable_decimals: create_array(|i| if i < 4 { 6 } else { 8 }),
            pool_balances: create_array(|i| initial_balances[i]),
            user_funds: create_array(|i| user_add[i]),
        };

        let (mut solnode, pool, user, _) = setup_standard_testcase(&params);

        let gov_ix = GovernanceInstruction::SetPaused { paused: true };
        pool.execute_governance_instruction(gov_ix, None, &mut solnode).unwrap();
        assert!(pool.state(&mut solnode).is_paused);

        user.stable_approve(&params.user_funds, &mut solnode);
        let defi_ix = DeFiInstruction::Add {
            input_amounts: params.user_funds,
            minimum_mint_amount: 0 as AmountT,
        };
        //TODO: check this. after changing pool, this shouldn't be passing since i'm not throwing an error anymore?
        // println!("\n\nSHOULD FAIL THIS EXECUTE_DEFI_IX\n\n");
        pool.execute_defi_instruction(defi_ix, &user.stables, Some(&user.lp), &mut solnode)
            .expect_err("Should not be able to execute defi_ix when paused");

        let gov_ix = GovernanceInstruction::SetPaused { paused: false };
        pool.execute_governance_instruction(gov_ix, None, &mut solnode).unwrap();

        assert!(!pool.state(&mut solnode).is_paused);

        user.stable_approve(&params.user_funds, &mut solnode);
        let defi_ix = DeFiInstruction::Add {
            input_amounts: params.user_funds,
            minimum_mint_amount: 0 as AmountT,
        };
        pool.execute_defi_instruction(defi_ix, &user.stables, Some(&user.lp), &mut solnode)
            .unwrap();
    }
}

#[tokio::test]
async fn test_pool_swap_exact_output() {
    let mut test = ProgramTest::new(
        "pool",
        pool::id(),
        processor!(pool::processor::Processor::<{ TOKEN_COUNT }>::process),
    );

    // limit to track compute unit increase.
    // Mainnet compute budget as of 08/25/2021 is 200_000
    test.set_bpf_compute_max_units(200_000);

    //TODO: not sure if needed
    let user_accounts_owner = Keypair::new();

    let (mut banks_client, payer, _recent_blockhash) = test.start().await;

    const RESERVE_AMOUNT: u64 = 42;

    let amp_factor = DecimalU64::new(1000, 0).unwrap();
    let lp_fee = DecimalU64::new(1000, 4).unwrap();
    let governance_fee = DecimalU64::new(1000, 5).unwrap();
    let pool = TestPoolAccountInfo::<{ TOKEN_COUNT }>::new();
    pool.init_pool(
        &mut banks_client,
        &payer,
        &user_accounts_owner,
        amp_factor,
        lp_fee,
        governance_fee,
    )
    .await;

    let mut deposit_tokens_to_mint_arrayvec = ArrayVec::<_, TOKEN_COUNT>::new();
    let mut deposit_tokens_for_approval_arrayvec = ArrayVec::<_, TOKEN_COUNT>::new();
    let mut inc: u64 = 1;
    for i in 0..TOKEN_COUNT {
        let approval_amount: u64 = inc * 100;
        let mint_amount: u64 = approval_amount * 2;
        deposit_tokens_to_mint_arrayvec.push(mint_amount);
        deposit_tokens_for_approval_arrayvec.push(approval_amount);
        inc += 1;
    }
    let deposit_tokens_to_mint: [AmountT; TOKEN_COUNT] = deposit_tokens_to_mint_arrayvec.into_inner().unwrap();
    let deposit_tokens_for_approval: [AmountT; TOKEN_COUNT] =
        deposit_tokens_for_approval_arrayvec.into_inner().unwrap();
    let user_transfer_authority = Keypair::new();
    let (user_token_accounts, user_lp_token_account) = pool
        .prepare_accounts_for_add(
            &mut banks_client,
            &payer,
            &user_accounts_owner,
            &user_transfer_authority.pubkey(),
            deposit_tokens_to_mint,
            deposit_tokens_for_approval,
        )
        .await;
    for i in 0..TOKEN_COUNT {
        let user_token_acct_acct = get_account(&mut banks_client, &user_token_accounts[i].pubkey()).await;
        let user_token_acct = Token::unpack(&user_token_acct_acct.data).unwrap();
        println!(
            "user_token_accounts[{}].amount is {}. delegated_amount: {}",
            i, user_token_acct.amount, user_token_acct.delegated_amount
        );
    }

    let mut user_token_keypairs_arrvec = ArrayVec::<_, TOKEN_COUNT>::new();
    for i in 0..TOKEN_COUNT {
        user_token_keypairs_arrvec.push(user_token_accounts[i].pubkey());
    }
    let user_token_pubkeys = user_token_keypairs_arrvec.into_inner().unwrap();
    let user_token_balances_before = get_token_balances(&mut banks_client, user_token_pubkeys).await;
    let user_lp_token_balances_before =
        get_token_balances::<{ 1 }>(&mut banks_client, [user_lp_token_account.pubkey()]).await;
    assert_eq!(deposit_tokens_to_mint, user_token_balances_before);
    assert_eq!(0, user_lp_token_balances_before[0]);
    println!("[DEV] Executing add");
    pool.execute_add(
        &mut banks_client,
        &payer,
        &user_accounts_owner,
        &user_transfer_authority,
        &user_token_accounts,
        &spl_token::id(),
        &user_lp_token_account.pubkey(),
        deposit_tokens_for_approval,
        0,
    )
    .await;

    print!("user_account_owner: {}, user_transfer_authority: {}", user_accounts_owner.pubkey(), user_transfer_authority.pubkey());
    print_user_token_account_owners(&mut banks_client, user_token_pubkeys).await;
    let user_token_balances_after = get_token_balances(&mut banks_client, user_token_pubkeys).await;
    let user_token_balances_after_tree = get_token_balances2(&mut banks_client, user_token_pubkeys).await;
    let mut expected_user_token_balances_arrvec = ArrayVec::<_, TOKEN_COUNT>::new();
    for i in 0..TOKEN_COUNT {
        expected_user_token_balances_arrvec.push(deposit_tokens_to_mint[i] - deposit_tokens_for_approval[i]);
    }
    let expected_user_token_balances = expected_user_token_balances_arrvec.into_inner().unwrap();
    println!("expected_user_token_balances: {:?}", expected_user_token_balances);
    println!("user_token_balances_after: {:?}", user_token_balances_after_tree);
    //assert_eq!(expected_user_token_balances, user_token_balances_after);
    let user_lp_token_balance_after =
        get_token_balances::<{ 1 }>(&mut banks_client, [user_lp_token_account.pubkey()]).await;
    println!("user_lp_token_balance_after: {:?}", user_lp_token_balance_after);
    let governance_fee_balance =
        get_token_balances::<{ 1 }>(&mut banks_client, [pool.governance_fee_keypair.pubkey()]).await;
    println!("governance_fee_balance: {:?}", governance_fee_balance);
    let mut exact_output_amounts_arrayvec = ArrayVec::<_, TOKEN_COUNT>::new();
    let mut inc: u64 = 1;
    for i in 0..TOKEN_COUNT - 1 {
        let mint_amount: u64 = inc;
        exact_output_amounts_arrayvec.push(mint_amount);
        inc += 1;
    }
    exact_output_amounts_arrayvec.push(0);
    let exact_output_amounts: [AmountT; TOKEN_COUNT] = exact_output_amounts_arrayvec.into_inner().unwrap();
    println!("[DEV] exact_output_amounts: {:?}", exact_output_amounts);
    let input_token_index = 3;
    let maximum_input_amount = 10;
    //TODO: do i need to revoke afterwards?
    println!("[DEV] preparing accounts for swap");
    pool.prepare_accounts_for_swap_exact_output(
        &mut banks_client,
        &payer,
        &user_accounts_owner,
        &user_transfer_authority.pubkey(),
        &user_token_pubkeys,
        maximum_input_amount,
        input_token_index,
        
    ).await;

    pool.execute_swap_exact_output(
        &mut banks_client,
        &payer,
        &user_accounts_owner,
        &user_transfer_authority,
        &user_token_accounts,
        &spl_token::id(),
        maximum_input_amount,
        input_token_index,
        exact_output_amounts,
    ).await;

    

    let user_token_balances_after_swap = get_token_balances(&mut banks_client, user_token_pubkeys).await;
    println!("user_token_balances_after_swap: {:?}", user_token_balances_after_swap);
    // for i in 0..TOKEN_COUNT - 1 {
    //     assert_eq!(user_token_balances_after[i] - exact_input_amounts[i], user_token_balances_after_swap[i]);
    // }

    let governance_fee_balance = get_token_balances::<{ 1 }>(&mut banks_client, [pool.governance_fee_keypair.pubkey()]).await;
    println!("governance_fee_balance: {:?}", governance_fee_balance);


}
