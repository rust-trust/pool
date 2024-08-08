use crate::{
    common::create_array,
    decimal::{self, DecimalU64, U128},
    error::PoolError,
};

use std::{
    ops::{Add, Sub},
    vec::Vec,
};

use uint::construct_uint;
construct_uint! {
    pub struct U192(3);
}

use rust_decimal::{prelude::*, Decimal};
type InvariantResult<T> = Result<T, PoolError>;

pub type AmountT = U128;
type AmpT = Decimal;
type FeeT = Decimal;
type DecT = DecimalU64;

}



    // const ONE_HALF: Decimal = Decimal::from_parts(5, 0, 0, false, 1);
    // AmountT::from((decimal + ONE_HALF).trunc().to_u128().unwrap())

    //due to rounding errors we can get negative values here, hence the unwrap_or
    decimal.to_u128().unwrap_or(0).into()
}


        (self + denominator / 2) / denominator
    }
}


    }
}


fn exclude_index<const TOKEN_COUNT: usize>(index: usize, array: &[AmountT; TOKEN_COUNT]) -> Vec<AmountT> {
    array
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != index)
        .map(|(_, v)| *v)
        .collect::<Vec<AmountT>>()
}

fn sum_balances<const TOKEN_COUNT: usize>(balances: &[AmountT; TOKEN_COUNT]) -> AmountT {
    balances.iter().fold(AmountT::zero(), |acc, &balance| acc + balance)
}

fn binary_op_balances<const TOKEN_COUNT: usize>(
    op: impl Fn(AmountT, AmountT) -> AmountT,
    balances1: &[AmountT; TOKEN_COUNT],
    balances2: &[AmountT; TOKEN_COUNT],
) -> [AmountT; TOKEN_COUNT] {
}

fn unary_op_balances<const TOKEN_COUNT: usize>(
    op: impl Fn(AmountT) -> AmountT,
    balances: &[AmountT; TOKEN_COUNT],
) -> [AmountT; TOKEN_COUNT] {
    create_array(|i| op(balances[i]))
}

pub struct Invariant<const TOKEN_COUNT: usize>;
impl<const TOKEN_COUNT: usize> Invariant<TOKEN_COUNT> {
    pub fn add(
        input_amounts: &[AmountT; TOKEN_COUNT],
        pool_balances: &[AmountT; TOKEN_COUNT],
        amp_factor: DecT,
        lp_fee: DecT,
        governance_fee: DecT,
        lp_total_supply: AmountT,
        previous_depth: AmountT,
    ) -> InvariantResult<(AmountT, AmountT, AmountT)> {
        let amp_factor: Decimal = amp_factor.into();
        } else {
            let lp_fee: FeeT = lp_fee.into();
            let governance_fee: FeeT = governance_fee.into();
            let total_fee = lp_fee + governance_fee;
            Self::add_remove(
                true,
                &input_amounts,
                &pool_balances,
                amp_factor,
                total_fee,
                governance_fee,
                lp_total_supply,
            )
        }
    }

    pub fn swap_exact_input(
        input_amounts: &[AmountT; TOKEN_COUNT],
        output_index: usize,
        pool_balances: &[AmountT; TOKEN_COUNT],
        amp_factor: DecT,
        lp_fee: DecT,
        governance_fee: DecT,
        lp_total_supply: AmountT,
        previous_depth: AmountT,
    ) -> InvariantResult<(AmountT, AmountT, AmountT)> {
        let amp_factor: Decimal = amp_factor.into();
        let lp_fee: FeeT = lp_fee.into();
        let governance_fee: FeeT = governance_fee.into();
        let total_fee = lp_fee + governance_fee;
        Self::swap(
            true,
            &input_amounts,
            output_index,
            &pool_balances,
            amp_factor,
            total_fee,
            governance_fee,
            lp_total_supply,
        )
    }

    pub fn swap_exact_output(
        input_index: usize,
        output_amounts: &[AmountT; TOKEN_COUNT],
        pool_balances: &[AmountT; TOKEN_COUNT],
        amp_factor: DecT,
        lp_fee: DecT,
        governance_fee: DecT,
        lp_total_supply: AmountT,
        previous_depth: AmountT,
    ) -> InvariantResult<(AmountT, AmountT, AmountT)> {
        let amp_factor: Decimal = amp_factor.into();
        let lp_fee: FeeT = lp_fee.into();
        let governance_fee: FeeT = governance_fee.into();
        let total_fee = lp_fee + governance_fee;
        Self::swap(
            false,
            &output_amounts,
            input_index,
            &pool_balances,
            amp_factor,
            total_fee,
            governance_fee,
            lp_total_supply,
        )
    }

    pub fn remove_exact_burn(
        burn_amount: AmountT,
        output_index: usize,
        pool_balances: &[AmountT; TOKEN_COUNT],
        amp_factor: DecT,
        lp_fee: DecT,
        governance_fee: DecT,
        lp_total_supply: AmountT,
        previous_depth: AmountT,
    ) -> InvariantResult<(AmountT, AmountT, AmountT)> {
        let amp_factor: Decimal = amp_factor.into();
        let lp_fee: FeeT = lp_fee.into();
        let governance_fee: FeeT = governance_fee.into();
        let total_fee = lp_fee + governance_fee;
            burn_amount,
            output_index,
            &pool_balances,
            amp_factor,
            total_fee,
            governance_fee,
            lp_total_supply,
        )
    }

    pub fn remove_exact_output(
        output_amounts: &[AmountT; TOKEN_COUNT],
        pool_balances: &[AmountT; TOKEN_COUNT],
        amp_factor: DecT,
        lp_fee: DecT,
        governance_fee: DecT,
        lp_total_supply: AmountT,
        previous_depth: AmountT,
    ) -> InvariantResult<(AmountT, AmountT, AmountT)> {
        let amp_factor: Decimal = amp_factor.into();
        let lp_fee: FeeT = lp_fee.into();
        let governance_fee: FeeT = governance_fee.into();
        let total_fee = lp_fee + governance_fee;
        Self::add_remove(
            false,
            &output_amounts,
            &pool_balances,
            amp_factor,
            total_fee,
            governance_fee,
            lp_total_supply,
    }

    fn swap(
        is_exact_input: bool, //false => exact output
        amounts: &[AmountT; TOKEN_COUNT],
        index: usize,
        pool_balances: &[AmountT; TOKEN_COUNT],
        amp_factor: AmpT,
        total_fee: FeeT,
        governance_fee: FeeT,
        lp_total_supply: AmountT,
        let mut updated_balances = binary_op_balances(
            if is_exact_input { AmountT::add } else { AmountT::sub },
            &pool_balances,
            &amounts,
        );
            binary_op_balances(AmountT::sub, &updated_balances, &input_fee_amounts)
        } else {
            updated_balances
        });

        // println!("SWAP  swap_base_balances: {:?}", swap_base_balances);
        let known_balances = exclude_index(index, swap_base_balances);
        let final_amount = if !is_exact_input && !total_fee.is_zero() {
            fast_round(Decimal::from(intermediate_amount) / (Decimal::one() - total_fee))
        } else {
            intermediate_amount
        };
    }

    fn add_remove(
        is_add: bool, //false => remove
        amounts: &[AmountT; TOKEN_COUNT],
        pool_balances: &[AmountT; TOKEN_COUNT],
        amp_factor: AmpT,
        total_fee: FeeT,
        governance_fee: FeeT,
        lp_total_supply: AmountT,
        let updated_balances = binary_op_balances(
            if is_add { AmountT::add } else { AmountT::sub },
            &pool_balances,
            &amounts,
        );
                pool_balances,
            );
            let taxbase = binary_op_balances(
                |updated, scaled| match (is_add, updated > scaled) {
                    (true, true) => updated - scaled,
                    (false, false) => scaled - updated,
                    _ => 0.into(),
                },
                &updated_balances,
                &scaled_balances,
            );

            };
            let fee_amounts = unary_op_balances(|balance| fast_round(fee * Decimal::from(balance)), &taxbase);
            if updated_balances
                .iter()
                .zip(fee_amounts.iter())
                .any(|(&updated_balance, &fee_amount)| updated_balance <= fee_amount)
            {
                //This error is an artifact of the approximative, simplified way in which fees are calculated.
                //Fees are calculated using amounts rather than depth. This should be fine in real world situations
                //but (like all linear approximations) it leads to impossible demands in extreme situations:
                //The fee math implementation assumes that e.g. a fee of 25 % on inputs can be offset by charging an
                //extra 33 % to the token balance that is being withdrawn. This is only marginally true however
                //because the marginal price of each additional token withdrawn increases (tending towards infinity
                //as the balance of that particular token approaches zero), while the marginal price of each additional
                //token added decreases (tending towards zero as its token balance approaches infinity).
                //
                //Another easy intuition pump to see the issue with this approach is:
                //When withdrawing essentially the entire balance of one token, there is no way to withdraw an
                //additional (say) 10 % in fees of that token, since those extra 10 % simply don't exist in the pool.
                //
                //Overall, this issue should be of little practical concern however since any remove that would run
                //into it is economically trumped by a proportional remove that avoids fees altogether and would
                //essentially withdraw all token balances, including the requested one.
                return Err(PoolError::ImpossibleRemove);
            }
            let fee_adjusted_balances = binary_op_balances(AmountT::sub, &updated_balances, &fee_amounts);
            //solana_program::msg!("ADD/REMOVE 5");
            let fee_adjusted_depth = Self::calculate_depth(&fee_adjusted_balances, amp_factor, updated_depth)?;
            //solana_program::msg!("ADD/REMOVE 6");
            let total_fee_depth = updated_depth - fee_adjusted_depth;
    }

    fn remove_exact_burn_impl(
        burn_amount: AmountT,
        output_index: usize,
        pool_balances: &[AmountT; TOKEN_COUNT],
        amp_factor: AmpT,
        total_fee: FeeT,
        governance_fee: FeeT,
        lp_total_supply: AmountT,
        let updated_depth =
            initial_depth * (Decimal::from(lp_total_supply - burn_amount) / Decimal::from(lp_total_supply));
        debug_assert!(initial_depth > updated_depth);
        let known_balances = exclude_index(output_index, &pool_balances);
        let unknown_balance =
            Self::calculate_unknown_balance(&known_balances, updated_depth, amp_factor, pool_balances[output_index])?;
        let base_amount = pool_balances[output_index] - unknown_balance;
        let (output_amount, governance_mint_amount) = if !total_fee.is_zero() {
            let sum_pool_balances = sum_balances(&pool_balances);
            let output_amount = base_amount - fee_amount;
            let mut updated_balances = *pool_balances;
            updated_balances[output_index] -= output_amount;
        } else {
            (base_amount, 0.into())
        };
        Ok((output_amount, governance_mint_amount, fast_round(updated_depth)))
    }

    fn calculate_depth(
        pool_balances: &[AmountT; TOKEN_COUNT],
        amp_factor: AmpT,
        initial_guess: Decimal,
    ) -> InvariantResult<Decimal> {
        let pool_balances_times_n: [_; TOKEN_COUNT] =
            create_array(|i| U128::from(pool_balances[i]) * AmountT::from(TOKEN_COUNT));
        let pool_balances_sum = sum_balances(pool_balances);

        // use f64 to calculate either the exact result (if there's sufficient precision) or an updated initial guess
        let mut depth = {
            let amp_factor = dec_to_f64(amp_factor);
            //numeric range considerations for reciprocal_decay_precomp:
            // https://en.wikipedia.org/wiki/Double-precision_floating-point_format
            //f64: 1 bit mantissa sign, 52 bit mantissa, 11 bit exponent (offset encoding)
            // https://en.wikipedia.org/wiki/Double-precision_floating-point_format#Exponent_encoding
            // => max exponent: 2^1023 ~= 10^308
            //scaled pool balances are at most 19 decimals (u64) + the decimal shift to
            // make them uniform in regards to token decimals (see MAX_DECIMAL_DIFFERENCE
            // in processor.rs - currently 8)
            //overall, even if the entire U128 range were to be used, this would still only
            // give 38 decimals per token and hence a product less than 10^240 for a pool
            // with 6 tokens, which is well within the range of f64 (i.e. < 10^308)
            let reciprocal_decay_precomp = pool_balances_times_n
                .iter()
                .fold(1f64, |acc, &pool_balance| acc * pool_balance.as_u128() as f64)
                .recip();
            let pool_balances_sum = pool_balances_sum.as_u128() as f64;
            let amp_times_sum = pool_balances_sum * amp_factor;
            let denominator_fixed = amp_factor - 1f64;

            let mut previous_depth = 0f64;
            let mut depth = if initial_guess.is_zero() {
                pool_balances_sum
            } else {
                dec_to_f64(initial_guess)
            };

            //sample values for f64 and its underlying representation:
            // f64       .to_bits() decimal    .to_bits() binary
            //  1     :  4607182418800017408 = 0_01111111111_0000000000000000000000000000000000000000000000000000
            //  2     :  4611686018427387904 = 0_10000000000_0000000000000000000000000000000000000000000000000000
            //  4     :  4616189618054758400 = 0_10000000001_0000000000000000000000000000000000000000000000000000
            //  5     :  4617315517961601024 = 0_10000000001_0100000000000000000000000000000000000000000000000000
            // -1     : 13830554455654793216 = 1_01111111111_0000000000000000000000000000000000000000000000000000
            // -2     : 13835058055282163712 = 1_10000000000_0000000000000000000000000000000000000000000000000000
            // -4     : 13839561654909534208 = 1_10000000001_0000000000000000000000000000000000000000000000000000
            // -5     : 13840687554816376832 = 1_10000000001_0100000000000000000000000000000000000000000000000000
            //  0.5   :  4602678819172646912 = 0_01111111110_0000000000000000000000000000000000000000000000000000
            //  0.25  :  4598175219545276416 = 0_01111111101_0000000000000000000000000000000000000000000000000000
            //  0.625 :  4603804719079489536 = 0_01111111110_0100000000000000000000000000000000000000000000000000
            // -0.5   : 13826050856027422720 = 1_01111111110_0000000000000000000000000000000000000000000000000000
            // -0.25  : 13821547256400052224 = 1_01111111101_0000000000000000000000000000000000000000000000000000
            // -0.625 : 13827176755934265344 = 1_01111111110_0100000000000000000000000000000000000000000000000000
            //                mantissa sign bit | exponent  | mantissa

            //terminates if we've converged to the correct value or exhausted the precision of f64
            loop {
                if depth.abs_diff(previous_depth) <= 0.5f64 {
                    return Ok(Decimal::from(depth as u128));
                }
                //AbsDiff::abs_diff(. , .) syntax to get rid of compiler warning
                if AbsDiff::abs_diff(depth.to_bits(), previous_depth.to_bits()) <= 2 {
                    break;
                }
                previous_depth = depth;

                //similar consideration as above:
                //depth.powi(TOKEN_COUNT+1) will always be less than 10^308 for 6 tokens
                let reciprocal_decay = depth.powi(TOKEN_COUNT as i32) * reciprocal_decay_precomp;
                let n_times_depth_times_decay = depth * reciprocal_decay * TOKEN_COUNT as f64;
                let numerator = amp_times_sum + n_times_depth_times_decay;
                let denominator = denominator_fixed + reciprocal_decay * (TOKEN_COUNT + 1) as f64;

                depth = numerator / denominator;
            }

            Decimal::from(depth as u128)
        };

        let pool_balances_times_n: [_; TOKEN_COUNT] = create_array(|i| Decimal::from(pool_balances_times_n[i]));
        let amp_times_sum = Decimal::from(pool_balances_sum) * amp_factor;
        let denominator_fixed = amp_factor - Decimal::one();

        let mut previous_depth = Decimal::zero();
            previous_depth = depth;

            let reciprocal_decay = pool_balances_times_n
                .iter()
                .fold(Decimal::one(), |acc, &pool_balance_times_n| {
                    acc * (depth / pool_balance_times_n)
                });
            let n_times_depth_times_decay = depth * reciprocal_decay * Decimal::from(TOKEN_COUNT);
            let numerator = amp_times_sum + n_times_depth_times_decay;
            let denominator = denominator_fixed + reciprocal_decay * Decimal::from(TOKEN_COUNT + 1);

            depth = numerator / denominator;
        }

        Ok(depth)
    }

    fn calculate_unknown_balance(
        // this should have type &[AmountT; TOKEN_COUNT-1] but Rust currently does not support const operations
        // on const generics and hence TOKEN_COUNT-1 is illegal and so it has to be a Vec instead...
        known_balances: &Vec<AmountT>,
        depth: Decimal,
        amp_factor: AmpT,
        initial_guess: AmountT,
    ) -> InvariantResult<AmountT> {
        let n = AmountT::from(TOKEN_COUNT);
        let known_balance_sum = known_balances
            .iter()
            previous_unknown_balance = unknown_balance;
            let numerator = numerator_fixed + unknown_balance * unknown_balance;
            let denominator = (denominator_fixed + unknown_balance * 2) - depth;

            unknown_balance = numerator.rounded_div(denominator);
            // println!(".         numerator: {}", numerator);
            // println!(".       denominator: {}", denominator);
            // println!(".   unknown_balance: {}", unknown_balance);
        }

        Ok(AmountT::from(unknown_balance.as_u128()))
    }
}

#[cfg(all(test, not(feature = "test-bpf")))]
mod tests {
    use super::*;
    use crate::decimal::DecimalU128;

    fn assert_close_enough(v1: AmountT, v2: AmountT, max_diff: AmountT) {
        let diff = if v1 > v2 { v1 - v2 } else { v2 - v1 };
        assert!(
            diff <= max_diff,
            "not sufficiently close: {} {}, max_diff: {}",
            v1,
            v2,
            max_diff
        );
    }

    #[test]
    fn basic_invariant_tests() {
        const TOKEN_COUNT: usize = 6;
        //grouped to signify that exact_depth depends on balances and amp_factor
        let (balances, amp_factor, exact_depth) = (
            create_array(|i| AmountT::from([20, 10, 20, 5, 2, 1][i])),
            DecT::from(1),
            //DecimalU128::new(5797595776747225261683921277u128.into(), 26).unwrap()
            DecimalU128::new(3770007484983239375907243892u128.into(), 26).unwrap(),
        );

        let exponent = 6 + 4;ount, &balances);
        let shifted_depth =
            DecimalU128::new(exact_depth.get_raw(), exact_depth.get_decimals() - exponent as u8).unwrap();
        let expected_depth = shifted_depth.trunc();

        let depth = fast_round(
        // println!(">>>        balances: {:?}", balances);
        for i in 0..TOKEN_COUNT {
            let input_balances = exclude_index(i, &balances);
            // println!(">>>  input_balances: {:?}", input_balances);
            // println!(">>> --------------------------");
            let unknown_balance = Invariant::<TOKEN_COUNT>::calculate_unknown_balance(
                &input_balances,
            )
            .unwrap();
            // println!(">>> unknown_balance: {}", unknown_balance);
            assert_close_enough(unknown_balance, balances[i], 1.into());
        }
    }

    #[test]
    fn swap_in_vs_out() {
        let amp_factor = DecT::new(1313, 3).unwrap();
        let lp_fee = DecT::new(10, 2).unwrap();
        // let lp_fee = DecT::from(0);
        let governance_fee = DecT::new(40, 2).unwrap();

        let balances = [BASE; TOKEN_COUNT];
        let original_input = balances[0] / 2;
        amounts[0] = original_input;

        let (yielded_output, government_mint_in, _) = Invariant::<TOKEN_COUNT>::swap_exact_input(
            &amounts,
            1,
            &balances,
            amp_factor,
            lp_fee,
            governance_fee,
            lp_total_supply,

        amounts[0] = yielded_output;

        let (required_input, government_mint_out, _) = Invariant::<TOKEN_COUNT>::swap_exact_output(
            1,
            &amounts,
            &balances,
            amp_factor,
            lp_fee,
            governance_fee,
            lp_total_supply,

        assert_close_enough(required_input, original_input, 1.into());
        assert_close_enough(government_mint_in, government_mint_out, 1.into());
    }

    #[test]
    fn remove_consistency() {
        let amp_factor = DecT::from(1);
        let lp_fee = DecT::new(10, 2).unwrap();
        // let lp_fee = DecT::from(0);
        let governance_fee = DecT::new(40, 2).unwrap();

        let balances = [BASE; TOKEN_COUNT];
        output[0] = balances[0] / 2;

        let (lp_required, gov_fee_token_remove, _) = Invariant::<TOKEN_COUNT>::remove_exact_output(
            &output,
            &balances,
            amp_factor,
            lp_fee,
            governance_fee,
            lp_total_supply,
        )
        .unwrap();

        let (amount_received, gov_fee_lp_burn, _) = Invariant::<TOKEN_COUNT>::remove_exact_burn(
            lp_required,
            0,
            &balances,
            amp_factor,
            lp_fee,
            governance_fee,
            lp_total_supply,
    }

    #[test]
    fn uniform_and_imbalanced_vs_together_add() {
        uniform_and_imbalanced_vs_together(true);
    }

    #[test]
    fn uniform_and_imbalanced_vs_together_remove() {
        uniform_and_imbalanced_vs_together(false);
    }

    fn uniform_and_imbalanced_vs_together(is_add: bool) {
        let amp_factor = DecT::new(1313, 3).unwrap();
        let lp_fee = DecT::new(10, 2).unwrap();
        let governance_fee = DecT::new(20, 2).unwrap();
        let balanced_divisor = 2;

        let balances = [BASE; TOKEN_COUNT];
        let balanced_amounts = [balances[0] / balanced_divisor; TOKEN_COUNT];

        let pool_op = if is_add {
            Invariant::<TOKEN_COUNT>::add
        } else {
            Invariant::<TOKEN_COUNT>::remove_exact_output
        };

        let (split_first_lp, nothing, _) = pool_op(
            &balanced_amounts,
            &balances,
            amp_factor,
            lp_fee,
            governance_fee,
            lp_total_supply,
        imbalanced_amounts[0] = balances[0] / balanced_divisor / 2;

        let (split_second_lp, split_governance_fee, _) = pool_op(
            &imbalanced_amounts,
            &binary_op_balances(
                if is_add { AmountT::add } else { AmountT::sub },
                &balances,
                &balanced_amounts,
            ),
            amp_factor,
            lp_fee,
            governance_fee,
            if is_add { AmountT::add } else { AmountT::sub }(lp_total_supply, lp_total_supply / balanced_divisor),
        )
        .unwrap();
        // println!(">>>         split_second_lp: {}", split_second_lp);
        // println!(">>>    split_governance_fee: {}", split_governance_fee);

        let (together_lp, together_governance_fee, _) = pool_op(
            &binary_op_balances(AmountT::add, &balanced_amounts, &imbalanced_amounts),
            &balances,
            amp_factor,
            lp_fee,
            governance_fee,
            lp_total_supply,
        )
        .unwrap();
        // println!(">>>             together_lp: {}", together_lp);
        // println!(">>> together_governance_fee: {}", together_governance_fee);
    }
}
