use crate::{
    data::{BetBatch, BetBatcher},
    dataset::BetResultsDataset,
    model::{Model, ModelConfig},
};

use burn::{
    data::dataloader::DataLoaderBuilder,
    lr_scheduler::noam::NoamLrSchedulerConfig,
    nn::loss::{
        BinaryCrossEntropyLossConfig, CrossEntropyLossConfig, HuberLossConfig, MseLoss,
        PoissonNllLossConfig, Reduction,
    },
    optim::AdamConfig,
    prelude::*,
    record::CompactRecorder,
    tensor::backend::AutodiffBackend,
    train::{
        metric::{AccuracyMetric, HammingScore, LearningRateMetric, LossMetric},
        renderer::{MetricState, MetricsRenderer, TrainingProgress},
        LearnerBuilder, MultiLabelClassificationOutput, RegressionOutput, TrainOutput, TrainStep,
        ValidStep,
    },
};

impl<B: Backend> Model<B> {
    pub fn forward_classification(
        &self,
        input_server_seed_hash_next_roll: Tensor<B, 3>,
        targets: Tensor<B, 2, Int>,
    ) -> RegressionOutput<B> {
        let output = self.forward(input_server_seed_hash_next_roll);
        let loss = HuberLossConfig::new(0.5).init().forward(
            output.clone(),
            targets.clone().float(),
            Reduction::Sum,
        );

        RegressionOutput::new(loss, output, targets.float())
    }
}

impl<B: AutodiffBackend> TrainStep<BetBatch<B>, RegressionOutput<B>> for Model<B> {
    fn step(&self, batch: BetBatch<B>) -> TrainOutput<RegressionOutput<B>> {
        let item = self.forward_classification(batch.input_server_seed_hash_data, batch.targets);

        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<BetBatch<B>, RegressionOutput<B>> for Model<B> {
    fn step(&self, batch: BetBatch<B>) -> RegressionOutput<B> {
        self.forward_classification(batch.input_server_seed_hash_data, batch.targets)
    }
}

#[derive(Config)]
pub struct TrainingConfig {
    pub model: ModelConfig,
    pub optimizer: AdamConfig,
    #[config(default = 2500)]
    pub num_epochs: usize,
    #[config(default = 1000)]
    pub batch_size: usize,
    #[config(default = 1)]
    pub num_workers: usize,
    #[config(default = 42)]
    pub seed: u64,
    #[config(default = 1e-8)]
    pub learning_rate: f64,
}

#[allow(dead_code)]
struct NoRenderer {}

impl MetricsRenderer for NoRenderer {
    fn update_train(&mut self, _state: MetricState) {}

    fn update_valid(&mut self, _state: MetricState) {}

    fn render_train(&mut self, _item: TrainingProgress) {}

    fn render_valid(&mut self, _item: TrainingProgress) {}
}

#[allow(dead_code)]
fn create_artifact_dir(artifact_dir: &str) {
    std::fs::remove_dir_all(artifact_dir).ok();
    std::fs::create_dir_all(artifact_dir).ok();
}

pub fn train<B: AutodiffBackend>(artifact_dir: &str, config: TrainingConfig, device: B::Device) {
    create_artifact_dir(artifact_dir);
    config
        .save(format!("{artifact_dir}/config.json"))
        .expect("Config should be saved successfully");
    B::seed(config.seed);

    let batcher_train = BetBatcher::<B>::new(device.clone());
    let batcher_valid = BetBatcher::<B::InnerBackend>::new(device.clone());

    let dataloader_train = DataLoaderBuilder::new(batcher_train)
        .batch_size(config.batch_size)
        .num_workers(config.num_workers)
        .build(BetResultsDataset::train().unwrap());

    let dataloader_test = DataLoaderBuilder::new(batcher_valid)
        .batch_size(config.batch_size)
        .num_workers(config.num_workers)
        .build(BetResultsDataset::test().unwrap());

    let learner = LearnerBuilder::new(artifact_dir)
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .with_file_checkpointer(CompactRecorder::new())
        .devices(vec![device.clone()])
        .num_epochs(config.num_epochs)
        // .checkpoint(1085)
        // .renderer(NoRenderer {})
        .summary()
        .build(
            config.model.init::<B>(&device),
            config.optimizer.init(),
            config.learning_rate,
        );

    let model_trained = learner.fit(dataloader_train, dataloader_test);

    model_trained
        .save_file(format!("{artifact_dir}/model"), &CompactRecorder::new())
        .expect("Trained model should be saved successfully");
}

