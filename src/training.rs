use crate::{
    data::{BetBatch, BetBatcher},
    dataset::BetResultsDataset,
    model::{Model, ModelConfig},
};

use burn::{
    data::dataloader::DataLoaderBuilder,
    lr_scheduler::noam::NoamLrSchedulerConfig,
    nn::loss::CrossEntropyLossConfig,
    optim::AdamConfig,
    prelude::*,
    record::CompactRecorder,
    tensor::backend::AutodiffBackend,
    train::{
        metric::{CudaMetric, HammingScore, LearningRateMetric, LossMetric},
        renderer::{MetricState, MetricsRenderer, TrainingProgress},
        LearnerBuilder, MultiLabelClassificationOutput, TrainOutput, TrainStep, ValidStep,
    },
};

impl<B: Backend> Model<B> {
    pub fn forward_classification(&self, item: BetBatch<B>) -> MultiLabelClassificationOutput<B> {
        let class_indices = item.targets.clone().argmax(1).flatten::<1>(0, 1);
        let output = self.forward(item.clone());
        let loss = CrossEntropyLossConfig::new()
            .init(&output.device())
            .forward(output.clone(), class_indices.clone());

        MultiLabelClassificationOutput::new(loss, output, item.targets)
    }
}

impl<B: AutodiffBackend> TrainStep<BetBatch<B>, MultiLabelClassificationOutput<B>> for Model<B> {
    fn step(&self, batch: BetBatch<B>) -> TrainOutput<MultiLabelClassificationOutput<B>> {
        let item = self.forward_classification(batch);

        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<BetBatch<B>, MultiLabelClassificationOutput<B>> for Model<B> {
    fn step(&self, batch: BetBatch<B>) -> MultiLabelClassificationOutput<B> {
        self.forward_classification(batch)
    }
}

#[derive(Config)]
pub struct TrainingConfig {
    pub optimizer: AdamConfig,
    #[config(default = 512)]
    pub max_seq_len: usize,
    #[config(default = 10000000)]
    pub num_epochs: usize,
    #[config(default = 100)]
    pub batch_size: usize,
    #[config(default = 1)]
    pub num_workers: usize,
    #[config(default = 42)]
    pub seed: u64,
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

    let model = ModelConfig::new().init::<B>(&device);

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

    let accum = 6;
    let optim = config.optimizer.init();
    let lr_scheduler = NoamLrSchedulerConfig::new(0.01 / accum as f64)
        .with_warmup_steps(6000)
        .init()
        .expect("Failed to create learning rate scheduler");

    let learner = LearnerBuilder::new(artifact_dir)
        .metric_train(CudaMetric::new())
        .metric_valid(CudaMetric::new())
        .metric_train(LossMetric::new())
        .metric_valid(LossMetric::new())
        .metric_train_numeric(LearningRateMetric::new())
        .metric_train_numeric(HammingScore::new())
        .with_file_checkpointer(CompactRecorder::new())
        .grads_accumulation(accum)
        .num_epochs(config.num_epochs)
        // .renderer(NoRenderer {})
        .summary()
        .build(model, optim, lr_scheduler);

    let model_trained = learner.fit(dataloader_train, dataloader_test);

    model_trained
        .save_file(format!("{artifact_dir}/model"), &CompactRecorder::new())
        .expect("Trained model should be saved successfully");
}
