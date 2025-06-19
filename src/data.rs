use burn::{data::dataloader::batcher::Batcher, prelude::*};
use sha2::{Digest, Sha256};

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
    pub input_server_seed_hash_data: Tensor<B, 3>,
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
                        (0..4).rev().map(move |i| (((value >> i) & 1) as u8) as f32)
                    })
                    .collect::<Vec<f32>>();

                vals.resize(512, 0.);

                vals.append(
                    &mut itm
                        .server_seed_hash_previous_roll
                        .chars()
                        .flat_map(|chr| {
                            let value = chr.to_digit(16).unwrap_or(0);
                            (0..4).rev().map(move |i| (((value >> i) & 1) as u8) as f32)
                        })
                        .collect::<Vec<f32>>(),
                );

                vals.resize(1024, 0.);

                vals.append(
                    &mut itm
                        .client_seed
                        .chars()
                        .flat_map(|chr| {
                            let value = chr.to_digit(16).unwrap_or(0);
                            (0..8).rev().map(move |i| (((value >> i) & 1) as u8) as f32)
                        })
                        .collect::<Vec<f32>>(),
                );

                vals.resize(1536, 0.);

                vals.append(
                    &mut (0..512)
                        .map(|i| (((itm.nonce >> i) & 1) as u8) as f32)
                        .collect::<Vec<f32>>(),
                );

                vals.resize(2048, 0.);

                vals
            })
            .collect::<Vec<f32>>();

        let hash_data = TensorData::new(
            inputs_hash,
            [items.len() / history_size, history_size, 2048],
        );
        let hash_data: Tensor<B, 3> = Tensor::from(hash_data).to_device(&self.device);

        let targets = items
            .chunks(history_size)
            .flat_map(|itm| {
                let mut arr = [0.; 10001];
                if let Some(itm) = itm.last() {
                    arr[itm.next_number as usize] = 1.;
                }
                arr
            })
            .collect::<Vec<f32>>();

        let target_data = TensorData::new(targets, [items.len() / history_size, 10001]);
        let target_data: Tensor<B, 2> = Tensor::from(target_data).to_device(&self.device);
        let target_data = target_data.int();

        BetBatch {
            input_server_seed_hash_data: hash_data,
            targets: target_data,
        }
    }
}

