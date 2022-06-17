use std::collections::HashMap;
use std::marker::PhantomData;

use itertools::{izip, multiunzip, Itertools};
use plonky2::field::extension_field::{Extendable, FieldExtension};
use plonky2::field::packed_field::PackedField;
use plonky2::field::polynomial::PolynomialValues;
use plonky2::hash::hash_types::RichField;
use plonky2::timed;
use plonky2::util::timing::TimingTree;
use rand::{thread_rng, Rng};

use crate::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use crate::memory::registers::{
    sorted_value_limb, value_limb, ADDR_CONTEXT, ADDR_SEGMENT, ADDR_VIRTUAL, CONTEXT_FIRST_CHANGE,
    COUNTER, COUNTER_PERMUTED, IS_READ, NUM_REGISTERS, RANGE_CHECK, RANGE_CHECK_PERMUTED,
    SEGMENT_FIRST_CHANGE, SORTED_ADDR_CONTEXT, SORTED_ADDR_SEGMENT, SORTED_ADDR_VIRTUAL,
    SORTED_IS_READ, SORTED_TIMESTAMP, TIMESTAMP, VIRTUAL_FIRST_CHANGE,
};
use crate::stark::Stark;
use crate::util::{permuted_cols, trace_rows_to_poly_values};
use crate::vars::{StarkEvaluationTargets, StarkEvaluationVars};

#[derive(Default)]
pub struct TransactionMemory {
    pub calls: Vec<ContractMemory>,
}

/// A virtual memory space specific to the current contract call.
pub struct ContractMemory {
    pub code: MemorySegment,
    pub main: MemorySegment,
    pub calldata: MemorySegment,
    pub returndata: MemorySegment,
}

pub struct MemorySegment {
    pub content: Vec<u8>,
}

pub(crate) const NUM_PUBLIC_INPUTS: usize = 0;

#[derive(Copy, Clone)]
pub struct MemoryStark<F, const D: usize> {
    pub(crate) f: PhantomData<F>,
}

pub fn generate_random_memory_ops<F: RichField>(num_ops: usize) -> Vec<(F, F, F, F, F, [F; 8])> {
    let mut memory_ops = Vec::new();

    let mut rng = thread_rng();

    let mut current_memory_values: HashMap<(F, F, F), [F; 8]> = HashMap::new();
    let mut cur_timestamp = 0;
    for i in 0..num_ops {
        let is_read = if i == 0 { false } else { rng.gen() };
        let is_read_F = F::from_bool(is_read);

        let (context, segment, virt, vals) = if is_read {
            let written: Vec<_> = current_memory_values.keys().collect();
            let &(context, segment, virt) = written[rng.gen_range(0..written.len())];
            let &vals = current_memory_values
                .get(&(context, segment, virt))
                .unwrap();

            (context, segment, virt, vals)
        } else {
            let context = F::from_canonical_usize(rng.gen_range(0..256));
            let segment = F::from_canonical_usize(rng.gen_range(0..8));
            let virt = F::from_canonical_usize(rng.gen_range(0..20));

            let val: [u32; 8] = rng.gen();
            let vals: [F; 8] = val.map(F::from_canonical_u32);

            current_memory_values.insert((context, segment, virt), vals);

            (context, segment, virt, vals)
        };

        let timestamp = F::from_canonical_usize(cur_timestamp);
        cur_timestamp += 1;

        memory_ops.push((timestamp, is_read_F, context, segment, virt, vals))
    }

    memory_ops
}

pub fn sort_memory_ops<F: RichField>(
    timestamp: &[F],
    is_read: &[F],
    context: &[F],
    segment: &[F],
    virtuals: &[F],
    values: &Vec<[F; 8]>,
) -> (Vec<F>, Vec<F>, Vec<F>, Vec<F>, Vec<F>, Vec<[F; 8]>) {
    let mut ops: Vec<(F, F, F, F, F, [F; 8])> = izip!(
        timestamp.iter().cloned(),
        is_read.iter().cloned(),
        context.iter().cloned(),
        segment.iter().cloned(),
        virtuals.iter().cloned(),
        values.iter().cloned(),
    )
    .collect();

    ops.sort_by_key(|&(t, _, c, s, v, _)| {
        (
            c.to_noncanonical_u64(),
            s.to_noncanonical_u64(),
            v.to_noncanonical_u64(),
            t.to_noncanonical_u64(),
        )
    });

    multiunzip(ops)
}

pub fn generate_first_change_flags<F: RichField>(
    context: &Vec<F>,
    segment: &Vec<F>,
    virtuals: &Vec<F>,
) -> (Vec<F>, Vec<F>, Vec<F>) {
    let num_ops = context.len();
    let mut context_first_change = Vec::new();
    let mut segment_first_change = Vec::new();
    let mut virtual_first_change = Vec::new();
    for idx in 0..num_ops - 1 {
        let this_context_first_change = if context[idx] != context[idx + 1] {
            F::ONE
        } else {
            F::ZERO
        };
        let this_segment_first_change = if segment[idx] != segment[idx + 1] {
            F::ONE * (F::ONE - this_context_first_change)
        } else {
            F::ZERO
        };
        let this_virtual_first_change = if virtuals[idx] != virtuals[idx + 1] {
            F::ONE * (F::ONE - this_context_first_change) * (F::ONE - this_segment_first_change)
        } else {
            F::ZERO
        };

        context_first_change.push(this_context_first_change);
        segment_first_change.push(this_segment_first_change);
        virtual_first_change.push(this_virtual_first_change);
    }

    context_first_change.push(F::ZERO);
    segment_first_change.push(F::ZERO);
    virtual_first_change.push(F::ZERO);

    (
        context_first_change,
        segment_first_change,
        virtual_first_change,
    )
}

pub fn generate_range_check_value<F: RichField>(
    context: &Vec<F>,
    segment: &Vec<F>,
    virtuals: &Vec<F>,
    timestamp: &Vec<F>,
    context_first_change: &Vec<F>,
    segment_first_change: &Vec<F>,
    virtual_first_change: &Vec<F>,
) -> Vec<F> {
    let num_ops = context.len();
    let mut range_check = Vec::new();

    for idx in 0..num_ops - 1 {
        let this_address_unchanged = F::ONE
            - context_first_change[idx]
            - segment_first_change[idx]
            - virtual_first_change[idx];

        range_check.push(
            context_first_change[idx] * (context[idx + 1] - context[idx] - F::ONE)
                + segment_first_change[idx] * (segment[idx + 1] - segment[idx] - F::ONE)
                + virtual_first_change[idx] * (virtuals[idx + 1] - virtuals[idx] - F::ONE)
                + this_address_unchanged * (timestamp[idx + 1] - timestamp[idx] - F::ONE),
        );
    }

    range_check.push(F::ZERO);

    range_check
}

impl<F: RichField + Extendable<D>, const D: usize> MemoryStark<F, D> {
    pub(crate) fn generate_trace_rows(
        &self,
        memory_ops: Vec<(F, F, F, F, F, [F; 8])>,
    ) -> Vec<[F; NUM_REGISTERS]> {
        let num_ops = memory_ops.len();

        let mut trace_cols: [Vec<F>; NUM_REGISTERS] = vec![vec![F::ZERO; num_ops]; NUM_REGISTERS]
            .try_into()
            .unwrap();
        for i in 0..num_ops {
            let (timestamp, is_read, context, segment, virt, values) = memory_ops[i];
            trace_cols[TIMESTAMP][i] = timestamp;
            trace_cols[IS_READ][i] = is_read;
            trace_cols[ADDR_CONTEXT][i] = context;
            trace_cols[ADDR_SEGMENT][i] = segment;
            trace_cols[ADDR_VIRTUAL][i] = virt;
            for j in 0..8 {
                trace_cols[value_limb(j)][i] = values[j];
            }
        }

        self.generate_memory(&mut trace_cols);

        let mut trace_rows = vec![[F::ZERO; NUM_REGISTERS]; num_ops];
        for (i, col) in trace_cols.iter().enumerate() {
            for (j, &val) in col.iter().enumerate() {
                trace_rows[j][i] = val;
            }
        }
        trace_rows
    }

    fn generate_memory(&self, trace_cols: &mut [Vec<F>]) {
        let num_trace_rows = trace_cols[0].len();

        let timestamp = &trace_cols[TIMESTAMP];
        let is_read = &trace_cols[IS_READ];
        let context = &trace_cols[ADDR_CONTEXT];
        let segment = &trace_cols[ADDR_SEGMENT];
        let virtuals = &trace_cols[ADDR_VIRTUAL];
        let values: Vec<[F; 8]> = (0..num_trace_rows)
            .map(|i| {
                let arr: [F; 8] = (0..8)
                    .map(|j| &trace_cols[value_limb(j)][i])
                    .cloned()
                    .collect_vec()
                    .try_into()
                    .unwrap();
                arr
            })
            .collect();

        let (
            sorted_timestamp,
            sorted_is_read,
            sorted_context,
            sorted_segment,
            sorted_virtual,
            sorted_values,
        ) = sort_memory_ops(timestamp, is_read, context, segment, virtuals, &values);

        let (context_first_change, segment_first_change, virtual_first_change) =
            generate_first_change_flags(&sorted_context, &sorted_segment, &sorted_virtual);

        let range_check_value = generate_range_check_value(
            &sorted_context,
            &sorted_segment,
            &sorted_virtual,
            &sorted_timestamp,
            &context_first_change,
            &segment_first_change,
            &virtual_first_change,
        );

        trace_cols[SORTED_TIMESTAMP] = sorted_timestamp;
        trace_cols[SORTED_IS_READ] = sorted_is_read;
        trace_cols[SORTED_ADDR_CONTEXT] = sorted_context;
        trace_cols[SORTED_ADDR_SEGMENT] = sorted_segment;
        trace_cols[SORTED_ADDR_VIRTUAL] = sorted_virtual;
        for i in 0..num_trace_rows {
            for j in 0..8 {
                trace_cols[sorted_value_limb(j)][i] = sorted_values[i][j];
            }
        }

        trace_cols[CONTEXT_FIRST_CHANGE] = context_first_change;
        trace_cols[SEGMENT_FIRST_CHANGE] = segment_first_change;
        trace_cols[VIRTUAL_FIRST_CHANGE] = virtual_first_change;

        trace_cols[RANGE_CHECK] = range_check_value;
        trace_cols[COUNTER] = (0..trace_cols[0].len())
            .map(|i| F::from_canonical_usize(i))
            .collect();

        let (permuted_inputs, permuted_table) =
            permuted_cols(&trace_cols[RANGE_CHECK], &trace_cols[COUNTER]);
        trace_cols[RANGE_CHECK_PERMUTED] = permuted_inputs;
        trace_cols[COUNTER_PERMUTED] = permuted_table;
    }

    pub fn generate_trace(
        &self,
        memory_ops: Vec<(F, F, F, F, F, [F; 8])>,
    ) -> Vec<PolynomialValues<F>> {
        let mut timing = TimingTree::new("generate trace", log::Level::Debug);

        // Generate the witness, except for permuted columns in the lookup argument.
        let trace_rows = timed!(
            &mut timing,
            "generate trace rows",
            self.generate_trace_rows(memory_ops)
        );

        let trace_polys = timed!(
            &mut timing,
            "convert to PolynomialValues",
            trace_rows_to_poly_values(trace_rows)
        );

        timing.print();
        trace_polys
    }
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for MemoryStark<F, D> {
    const COLUMNS: usize = NUM_REGISTERS;
    const PUBLIC_INPUTS: usize = NUM_PUBLIC_INPUTS;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        vars: StarkEvaluationVars<FE, P, { Self::COLUMNS }, { Self::PUBLIC_INPUTS }>,
        yield_constr: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let one = P::from(FE::ONE);

        let timestamp = vars.local_values[SORTED_TIMESTAMP];
        let addr_context = vars.local_values[SORTED_ADDR_CONTEXT];
        let addr_segment = vars.local_values[SORTED_ADDR_SEGMENT];
        let addr_virtual = vars.local_values[SORTED_ADDR_VIRTUAL];
        let values: Vec<_> = (0..8)
            .map(|i| vars.local_values[sorted_value_limb(i)])
            .collect();

        let next_timestamp = vars.next_values[SORTED_TIMESTAMP];
        let next_is_read = vars.next_values[SORTED_IS_READ];
        let next_addr_context = vars.next_values[SORTED_ADDR_CONTEXT];
        let next_addr_segment = vars.next_values[SORTED_ADDR_SEGMENT];
        let next_addr_virtual = vars.next_values[SORTED_ADDR_VIRTUAL];
        let next_values: Vec<_> = (0..8)
            .map(|i| vars.next_values[sorted_value_limb(i)])
            .collect();

        let context_first_change = vars.local_values[CONTEXT_FIRST_CHANGE];
        let segment_first_change = vars.local_values[SEGMENT_FIRST_CHANGE];
        let virtual_first_change = vars.local_values[VIRTUAL_FIRST_CHANGE];
        let address_unchanged =
            one - context_first_change - segment_first_change - virtual_first_change;

        let range_check = vars.local_values[RANGE_CHECK];

        let not_context_first_change = one - context_first_change;
        let not_segment_first_change = one - segment_first_change;
        let not_virtual_first_change = one - virtual_first_change;
        let not_address_unchanged = one - address_unchanged;

        // First set of ordering constraint: first_change flags are boolean.
        yield_constr.constraint(context_first_change * not_context_first_change);
        yield_constr.constraint(segment_first_change * not_segment_first_change);
        yield_constr.constraint(virtual_first_change * not_virtual_first_change);
        yield_constr.constraint(address_unchanged * not_address_unchanged);

        // Second set of ordering constraints: no change before the column corresponding to the nonzero first_change flag.
        yield_constr
            .constraint_transition(segment_first_change * (next_addr_context - addr_context));
        yield_constr
            .constraint_transition(virtual_first_change * (next_addr_context - addr_context));
        yield_constr
            .constraint_transition(virtual_first_change * (next_addr_segment - addr_segment));
        yield_constr.constraint_transition(address_unchanged * (next_addr_context - addr_context));
        yield_constr.constraint_transition(address_unchanged * (next_addr_segment - addr_segment));
        yield_constr.constraint_transition(address_unchanged * (next_addr_virtual - addr_virtual));

        // Third set of ordering constraints: range-check difference in the column that should be increasing.
        let range_check_value = context_first_change * (next_addr_context - addr_context - one)
            + segment_first_change * (next_addr_segment - addr_segment - one)
            + virtual_first_change * (next_addr_virtual - addr_virtual - one)
            + address_unchanged * (next_timestamp - timestamp - one);
        yield_constr.constraint_transition(range_check - range_check_value);

        // Enumerate purportedly-ordered log.
        for i in 0..8 {
            yield_constr
                .constraint(next_is_read * address_unchanged * (next_values[i] - values[i]));
        }

        // Lookup argument for the range check.
        let local_perm_input = vars.local_values[RANGE_CHECK_PERMUTED];
        let next_perm_table = vars.next_values[COUNTER_PERMUTED];
        let next_perm_input = vars.next_values[COUNTER_PERMUTED];

        // A "vertical" diff between the local and next permuted inputs.
        let diff_input_prev = next_perm_input - local_perm_input;
        // A "horizontal" diff between the next permuted input and permuted table value.
        let diff_input_table = next_perm_input - next_perm_table;

        yield_constr.constraint(diff_input_prev * diff_input_table);

        // This is actually constraining the first row, as per the spec, since `diff_input_table`
        // is a diff of the next row's values. In the context of `constraint_last_row`, the next
        // row is the first row.
        yield_constr.constraint_last_row(diff_input_table);
    }

    fn eval_ext_circuit(
        &self,
        builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
        vars: StarkEvaluationTargets<D, { Self::COLUMNS }, { Self::PUBLIC_INPUTS }>,
        yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let one = builder.one_extension();

        let addr_context = vars.local_values[SORTED_ADDR_CONTEXT];
        let addr_segment = vars.local_values[SORTED_ADDR_SEGMENT];
        let addr_virtual = vars.local_values[SORTED_ADDR_VIRTUAL];
        let values: Vec<_> = (0..8)
            .map(|i| vars.local_values[sorted_value_limb(i)])
            .collect();
        let timestamp = vars.local_values[SORTED_TIMESTAMP];

        let next_addr_context = vars.next_values[SORTED_ADDR_CONTEXT];
        let next_addr_segment = vars.next_values[SORTED_ADDR_SEGMENT];
        let next_addr_virtual = vars.next_values[SORTED_ADDR_VIRTUAL];
        let next_values: Vec<_> = (0..8)
            .map(|i| vars.next_values[sorted_value_limb(i)])
            .collect();
        let next_is_read = vars.next_values[SORTED_IS_READ];
        let next_timestamp = vars.next_values[SORTED_TIMESTAMP];

        let context_first_change = vars.local_values[CONTEXT_FIRST_CHANGE];
        let segment_first_change = vars.local_values[SEGMENT_FIRST_CHANGE];
        let virtual_first_change = vars.local_values[VIRTUAL_FIRST_CHANGE];
        let address_unchanged = {
            let mut cur = builder.sub_extension(one, context_first_change);
            cur = builder.sub_extension(cur, segment_first_change);
            builder.sub_extension(cur, virtual_first_change)
        };

        let range_check = vars.local_values[RANGE_CHECK];

        let not_context_first_change = builder.sub_extension(one, context_first_change);
        let not_segment_first_change = builder.sub_extension(one, segment_first_change);
        let not_virtual_first_change = builder.sub_extension(one, virtual_first_change);
        let not_address_unchanged = builder.sub_extension(one, address_unchanged);
        let addr_context_diff = builder.sub_extension(next_addr_context, addr_context);
        let addr_segment_diff = builder.sub_extension(next_addr_segment, addr_segment);
        let addr_virtual_diff = builder.sub_extension(next_addr_virtual, addr_virtual);

        // First set of ordering constraint: traces are boolean.
        let context_first_change_bool =
            builder.mul_extension(context_first_change, not_context_first_change);
        yield_constr.constraint(builder, context_first_change_bool);
        let segment_first_change_bool =
            builder.mul_extension(segment_first_change, not_segment_first_change);
        yield_constr.constraint(builder, segment_first_change_bool);
        let virtual_first_change_bool =
            builder.mul_extension(virtual_first_change, not_virtual_first_change);
        yield_constr.constraint(builder, virtual_first_change_bool);
        let address_unchanged_bool =
            builder.mul_extension(address_unchanged, not_address_unchanged);
        yield_constr.constraint(builder, address_unchanged_bool);

        // Second set of ordering constraints: no change before the column corresponding to the nonzero first_change flag.
        let segment_first_change_check =
            builder.mul_extension(segment_first_change, addr_context_diff);
        yield_constr.constraint_transition(builder, segment_first_change_check);
        let virtual_first_change_check_1 =
            builder.mul_extension(virtual_first_change, addr_context_diff);
        yield_constr.constraint_transition(builder, virtual_first_change_check_1);
        let virtual_first_change_check_2 =
            builder.mul_extension(virtual_first_change, addr_segment_diff);
        yield_constr.constraint_transition(builder, virtual_first_change_check_2);
        let address_unchanged_check_1 = builder.mul_extension(address_unchanged, addr_context_diff);
        yield_constr.constraint_transition(builder, address_unchanged_check_1);
        let address_unchanged_check_2 = builder.mul_extension(address_unchanged, addr_segment_diff);
        yield_constr.constraint_transition(builder, address_unchanged_check_2);
        let address_unchanged_check_3 = builder.mul_extension(address_unchanged, addr_virtual_diff);
        yield_constr.constraint_transition(builder, address_unchanged_check_3);

        // Third set of ordering constraints: range-check difference in the column that should be increasing.
        let context_diff = {
            let diff = builder.sub_extension(next_addr_context, addr_context);
            builder.sub_extension(diff, one)
        };
        let context_range_check = builder.mul_extension(context_first_change, context_diff);
        let segment_diff = {
            let diff = builder.sub_extension(next_addr_segment, addr_segment);
            builder.sub_extension(diff, one)
        };
        let segment_range_check = builder.mul_extension(segment_first_change, segment_diff);
        let virtual_diff = {
            let diff = builder.sub_extension(next_addr_virtual, addr_virtual);
            builder.sub_extension(diff, one)
        };
        let virtual_range_check = builder.mul_extension(virtual_first_change, virtual_diff);
        let timestamp_diff = {
            let diff = builder.sub_extension(next_timestamp, timestamp);
            builder.sub_extension(diff, one)
        };
        let timestamp_range_check = builder.mul_extension(address_unchanged, timestamp_diff);

        let range_check_value = {
            let mut sum = builder.add_extension(context_range_check, segment_range_check);
            sum = builder.add_extension(sum, virtual_range_check);
            builder.add_extension(sum, timestamp_range_check)
        };
        let range_check_diff = builder.sub_extension(range_check, range_check_value);
        yield_constr.constraint_transition(builder, range_check_diff);

        // Enumerate purportedly-ordered log.
        for i in 0..8 {
            let value_diff = builder.sub_extension(next_values[i], values[i]);
            let zero_if_read = builder.mul_extension(address_unchanged, value_diff);
            let read_constraint = builder.mul_extension(next_is_read, zero_if_read);
            yield_constr.constraint(builder, read_constraint);
        }

        // Lookup argument for range check.
        let local_perm_input = vars.local_values[RANGE_CHECK_PERMUTED];
        let next_perm_table = vars.next_values[COUNTER_PERMUTED];
        let next_perm_input = vars.next_values[COUNTER_PERMUTED];

        // A "vertical" diff between the local and next permuted inputs.
        let diff_input_prev = builder.sub_extension(next_perm_input, local_perm_input);
        // A "horizontal" diff between the next permuted input and permuted table value.
        let diff_input_table = builder.sub_extension(next_perm_input, next_perm_table);

        let diff_product = builder.mul_extension(diff_input_prev, diff_input_table);
        yield_constr.constraint(builder, diff_product);

        // This is actually constraining the first row, as per the spec, since `diff_input_table`
        // is a diff of the next row's values. In the context of `constraint_last_row`, the next
        // row is the first row.
        yield_constr.constraint_last_row(builder, diff_input_table);
    }

    fn constraint_degree(&self) -> usize {
        3
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

    use crate::memory::memory_stark::MemoryStark;
    use crate::stark_testing::{test_stark_circuit_constraints, test_stark_low_degree};

    #[test]
    fn test_stark_degree() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;
        type S = MemoryStark<F, D>;

        let stark = S {
            f: Default::default(),
        };
        test_stark_low_degree(stark)
    }

    #[test]
    fn test_stark_circuit() -> Result<()> {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;
        type S = MemoryStark<F, D>;

        let stark = S {
            f: Default::default(),
        };
        test_stark_circuit_constraints::<F, C, S, D>(stark)
    }
}
