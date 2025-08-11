use burn::{data::dataloader::batcher::Batcher, prelude::*};

use crate::dataset::BetResultCsvRecord;

#[derive(Clone)]
pub struct BetBatcher<B: Backend> {
    device: B::Device,
}

impl<B: Backend> BetBatcher<B> {
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }
}

#[derive(Clone, Debug)]
pub struct BetBatch<B: Backend> {
    pub inputs: Tensor<B, 4>,
    pub targets: Tensor<B, 2, Int>,
}

impl<B: Backend> Batcher<B, BetResultCsvRecord, BetBatch<B>> for BetBatcher<B> {
    fn batch(&self, items: Vec<BetResultCsvRecord>, device: &B::Device) -> BetBatch<B> {
        let history_size: usize = 10;

        let inputs_data = items.clone();
        let inputs_hash = inputs_data
            .iter()
            .flat_map(|itm| {
                let mut vals = itm
                    .server_seed_hash_next_roll
                    .chars()
                    .flat_map(|chr| {
                        let value = chr.to_digit(16).unwrap_or(0);
                        (0..4)
                            .rev()
                            .map(move |i| ((value >> i) & 1).elem::<B::FloatElem>())
                    })
                    .collect::<Vec<B::FloatElem>>();

                vals.resize(256, 0f32.elem::<B::FloatElem>());

                vals.append(
                    &mut itm
                        .server_seed_hash_previous_roll
                        .chars()
                        .flat_map(|chr| {
                            let value = chr.to_digit(16).unwrap_or(0);
                            (0..4)
                                .rev()
                                .map(move |i| ((value >> i) & 1).elem::<B::FloatElem>())
                        })
                        .collect::<Vec<B::FloatElem>>(),
                );

                vals.resize(512, 0f32.elem::<B::FloatElem>());

                vals.append(
                    &mut itm
                        .client_seed
                        .chars()
                        .flat_map(|chr| {
                            let value = chr.to_digit(16).unwrap_or(0);
                            (0..4)
                                .rev()
                                .map(move |i| ((value >> i) & 1).elem::<B::FloatElem>())
                        })
                        .collect::<Vec<B::FloatElem>>(),
                );

                vals.resize(768, 0f32.elem::<B::FloatElem>());

                vals.append(
                    &mut (0..32)
                        .map(|i| ((itm.nonce >> i) & 1).elem::<B::FloatElem>())
                        .collect::<Vec<B::FloatElem>>(),
                );

                vals.resize(1024, 0f32.elem::<B::FloatElem>());

                vals
            })
            .collect::<Vec<B::FloatElem>>();

        let hash_data = TensorData::new(
            inputs_hash,
            [items.len() / history_size, history_size, 4, 256],
        );
        let hash_data: Tensor<B, 4> =
            Tensor::from(hash_data.convert::<B::FloatElem>()).to_device(&self.device);

        let targets = items
            .chunks(history_size)
            .flat_map(|itm| {
                let mut arr = [(-1f32).elem::<B::FloatElem>(); 100];
                if let Some(itm) = itm.last() {
                    arr[itm.next_number as usize / 100] = 1f32.elem::<B::FloatElem>();
                }
                arr
            })
            .collect::<Vec<B::FloatElem>>();

        let target_data = TensorData::new(targets, [items.len() / history_size, 100]);
        let target_data: Tensor<B, 2> =
            Tensor::from(target_data.convert::<B::FloatElem>()).to_device(device);
        let target_data = target_data.int();

        BetBatch {
            inputs: hash_data,
            targets: target_data,
        }
    }
}
