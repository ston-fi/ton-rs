use super::label_type::DictLabelType;
use super::leading_bit_utils::{add_leading_bit, all_bits_same, common_prefix_len, remove_leading_bit};
use crate::tlb_adapters::DictValAdapter;
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::marker::PhantomData;
use std::mem::swap;
use ton_lib_core::bail_ton_core_data;
use ton_lib_core::cell::CellBuilder;
use ton_lib_core::cell::TonCell;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::tlb_core::UnaryLen;

pub struct DictDataBuilder<'a, T, VA: DictValAdapter<T>> {
    keys_sorted: Vec<BigUint>, // contains 1 extra leading bit set to 1
    values_sorted: &'a [&'a T],
    key_bits_len_left: usize,
    _phantom: PhantomData<VA>,
}

impl<'a, T, VA: DictValAdapter<T>> DictDataBuilder<'a, T, VA> {
    pub fn new(
        key_bits_len: usize,
        mut keys_sorted: Vec<BigUint>,
        values_sorted: &'a [&'a T],
    ) -> Result<Self, TonCoreError> {
        // we support writing empty dict, but it's usually handled by 0 bit in parent cell
        prepare_keys(&mut keys_sorted, key_bits_len)?;
        let builder = DictDataBuilder {
            keys_sorted,
            values_sorted,
            key_bits_len_left: key_bits_len,
            _phantom: PhantomData,
        };
        Ok(builder)
    }

    pub fn build(mut self) -> Result<TonCell, TonCoreError> {
        let mut builder = TonCell::builder();
        if self.keys_sorted.is_empty() {
            return builder.build();
        }
        let mut keys = vec![];
        swap(&mut self.keys_sorted, &mut keys);

        let keys = keys.into_iter().enumerate().collect();
        self.fill_cell(&mut builder, keys)?;
        builder.build()
    }

    // keys: Vec<(original_key_position, remaining_key_part)>
    fn fill_cell(&mut self, builder: &mut CellBuilder, keys: Vec<(usize, BigUint)>) -> Result<(), TonCoreError> {
        if keys.len() == 1 {
            let (orig_key_pos, remaining_key) = &keys[0];
            return self.store_leaf(builder, *orig_key_pos, remaining_key);
        }

        // will restore it at the end
        let key_len_bits_left_original = self.key_bits_len_left;

        let key = &keys[0].1;
        let key_len = key.bits() as usize; // includes leading bit

        let common_prefix_len = common_prefix_len(key, &keys.last().unwrap().1);
        let label = {
            let ignored_suffix_len = key_len - common_prefix_len - 1;
            key >> ignored_suffix_len
        };
        self.store_label(builder, &label)?;

        let mut left_keys = Vec::with_capacity(keys.len() / 2);
        let mut right_keys = Vec::with_capacity(keys.len() / 2);

        let new_key_len = key_len - common_prefix_len - 1;
        let new_key_mask = (BigUint::one() << new_key_len) - 1u32;
        for (pos, key) in keys {
            let new_key = key & new_key_mask.clone();
            let is_right = new_key.bits() as usize == new_key_len;
            let new_key_internal = add_leading_bit(&new_key, new_key_len - 1);
            if is_right {
                right_keys.push((pos, new_key_internal));
            } else {
                left_keys.push((pos, new_key_internal));
            }
        }

        self.key_bits_len_left -= common_prefix_len + 1; // branch consumes 1 more bit
        let mut left_builder = TonCell::builder();
        self.fill_cell(&mut left_builder, left_keys)?;
        builder.write_ref(left_builder.build()?.into_ref())?;

        let mut right_builder = TonCell::builder();
        self.fill_cell(&mut right_builder, right_keys)?;
        builder.write_ref(right_builder.build()?.into_ref())?;

        self.key_bits_len_left = key_len_bits_left_original;
        Ok(())
    }

    fn store_leaf(
        &mut self,
        builder: &mut CellBuilder,
        orig_key_pos: usize,
        label: &BigUint,
    ) -> Result<(), TonCoreError> {
        self.store_label(builder, label)?;
        VA::write(builder, self.values_sorted[orig_key_pos])?;
        Ok(())
    }

    // expect label with leading one
    fn store_label(&self, builder: &mut CellBuilder, label: &BigUint) -> Result<(), TonCoreError> {
        assert!(label.bits() > 0);
        if label.is_one() {
            // it's leading bit => label_type == short, len == 0 => store [false, false]
            builder.write_num(&0, 2)?;
            return Ok(());
        }
        let all_bits_same = all_bits_same(label);

        let label_len = label.bits() as usize - 1;
        let label_len_len = (self.key_bits_len_left as f32 + 1.0).log2().ceil() as usize;
        let fair_label = remove_leading_bit(label);
        let same_label_len = if all_bits_same { 3 + label_len_len } else { usize::MAX };
        let short_label_len = 2 + label_len * 2;
        let long_label_len = 2 + label_len_len + label_len;

        let mut label_type = DictLabelType::Short;
        if long_label_len < short_label_len {
            label_type = DictLabelType::Long;
        }
        if same_label_len < short_label_len {
            label_type = DictLabelType::Same;
        }
        match label_type {
            DictLabelType::Same => {
                builder.write_bit(true)?;
                builder.write_bit(true)?;
                builder.write_bit(!fair_label.is_zero())?;
                builder.write_num(&label_len, label_len_len)?;
            }
            DictLabelType::Short => {
                builder.write_bit(false)?;
                let unary_len = UnaryLen(label_len);
                unary_len.write(builder)?;
                builder.write_num(&fair_label, label_len)?;
            }
            DictLabelType::Long => {
                builder.write_bit(true)?;
                builder.write_bit(false)?;
                builder.write_num(&label_len, label_len_len)?;
                builder.write_num(&fair_label, label_len)?;
            }
        }
        Ok(())
    }
}

fn prepare_keys(keys: &mut [BigUint], key_bits_len: usize) -> Result<(), TonCoreError> {
    for key in keys {
        let received_len_bits = key.bits() as usize;
        if received_len_bits > key_bits_len {
            bail_ton_core_data!(
                "dict key too long: expected max {key_bits_len} bits, got {received_len_bits} bits, key={key}"
            );
        }

        // add leading bit to maintain proper bits length
        *key = add_leading_bit(key, key_bits_len);
    }
    Ok(())
}
