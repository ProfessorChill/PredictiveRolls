#![recursion_limit = "256"]

pub mod data;
pub mod dataset;
pub mod inference;
pub mod model;
pub mod sites;
pub mod strategies;
pub mod training;
pub mod util;

use burn::{
    backend::wgpu::{Wgpu, WgpuDevice},
    prelude::*,
    record::{CompactRecorder, FullPrecisionSettings, NamedMpkFileRecorder, Recorder},
};
use colored::Colorize;
use model::Model;
use sha2::{Digest, Sha256};
use training::TrainingConfig;

#[allow(unused_imports)]
use crate::sites::{crypto_games::CryptoGames, duck_dice::DuckDiceIo, free_bitco_in::FreeBitcoIn};
use crate::sites::{BetError, BetResult, Site};

struct Game<B: Backend> {
    confidence: f32,
    site: Box<dyn Site>,
    model: Model<B>,
    device: B::Device,
    prediction: f32,
}

impl<B: Backend> Game<B> {
    async fn bet(&mut self) -> Result<(), BetError> {
        let bet_result = match self.site.do_bet(self.prediction, self.confidence).await {
            Ok(res) => res,
            Err(err) => match err {
                BetError::EmptyReply => return Ok(()),
                _ => return Err(err),
            },
        };

        if bet_result.result {
            self.site.on_win(&bet_result);
            self.print_res(&bet_result, true);
        } else {
            self.site.on_lose(&bet_result);
            self.print_res(&bet_result, false);
        }

        let history = self.site.get_history();
        let history_size = self.site.get_history_size();
        // Get server seed hash next roll and convert it to a tensor of shape (-1, 256).
        if history.len() >= history_size {
            let inputs_hash = history
                .iter()
                .flat_map(|itm| {
                    let mut vals = itm
                        .hash_next_roll
                        .chars()
                        .flat_map(|chr| {
                            let value = chr.to_digit(16).unwrap_or(0);
                            (0..4).rev().map(move |i| (((value >> i) & 1) as u8) as f32)
                        })
                        .collect::<Vec<f32>>();

                    vals.resize(512, 0.);

                    vals.append(
                        &mut itm
                            .hash_previous_roll
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
                [history.len() / history_size, history_size, 2048],
            );
            let hash_data: Tensor<B, 3> = Tensor::from(hash_data).to_device(&self.device);

            let output = self.model.forward(hash_data);
            let predicted_output = output
                .clone()
                .argmax(1)
                .into_data()
                .to_vec::<i32>()
                .unwrap();
            let predicted_output = predicted_output[0];
            let confidence = output.clone().into_data().to_vec::<f32>().unwrap()
                [predicted_output as usize]
                * 100.;
            // let predicted = (predicted_output[0] + 1.) * 10000. / 2.;
            // let predicted = (((predicted - 4500.) / (5500. - 4500.)) * (10000. - 0.)) + 0.;

            self.confidence = confidence;
            self.prediction = predicted_output as f32;
        }

        Ok(())
    }

    fn print_res(&self, bet_result: &BetResult, win: bool) {
        let profit_str = &format!("Profit: {:.8}", self.site.get_profit());
        let profit_str = if self.site.get_profit() > 0. {
            profit_str.green()
        } else {
            profit_str.red()
        };

        let golden_roll = if bet_result.number > 9900 || bet_result.number < 100 {
            (&format!("{: <5}", bet_result.number)).yellow()
        } else {
            format!("{: <5}", bet_result.number).normal()
        };

        let output_str = &format!(
            "#{: >6} || Balance: {:0>.8} || Roll: {: <5} || Multiplier: {: <6.2} || Wagered: {:.8} || Predicted: {: <5.0} || Confidence: {: <2.2} || {}",
            self.site.get_rolls(),
            self.site.get_balance(),
            golden_roll,
            self.site.get_current_multiplier(),
            self.site.get_current_bet(),
            self.prediction,
            self.confidence,
            profit_str,
        );
        let output_str = if win {
            output_str.green()
        } else {
            output_str.red()
        };

        println!("{output_str}");
    }
}

#[tokio::main]
async fn main() -> Result<(), BetError> {
    type MyBackend = Wgpu<f32, i32>;

    let device = WgpuDevice::default();
    let artifact_dir = "/home/jvne/Projects/rust/random_guesser/guide";

    let config = TrainingConfig::load(format!("{artifact_dir}/config.json"))
        .expect("Config should exist for the model; run train first.");
    /*
    let record = NamedMpkFileRecorder::<FullPrecisionSettings>::default()
        .load(format!("{artifact_dir}/model").into(), &device)
        .expect("Trained model should exist; run train first.");
    */
    let record = CompactRecorder::new()
        .load(format!("{artifact_dir}/model").into(), &device)
        .expect("Trained model should exist; run train first.");

    let model: Model<MyBackend> = config.model.init(&device).load_record(record);

    let mut game = Game::<MyBackend> {
        confidence: 0.,
        site: Box::new(DuckDiceIo::default()),
        model,
        device,
        prediction: 0.,
    };
    game.site.login().await?;

    loop {
        game.bet().await?;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
