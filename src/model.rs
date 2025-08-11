use burn::{prelude::*, tensor::Distribution};

use crate::data::BetBatch;

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    input_layer: nn::conv::Conv2d<B>,
    positional_encoding: nn::PositionalEncoding<B>,
    transformer_encoder: nn::transformer::TransformerEncoder<B>,
    lstm1: nn::Lstm<B>,
    lstm2: nn::Lstm<B>,
    transformer_decoder: nn::transformer::TransformerDecoder<B>,
    output_layer: nn::Linear<B>,
}

#[derive(Config)]
pub struct ModelConfig {}

impl ModelConfig {
    pub fn init<B: Backend>(&self, device: &B::Device) -> Model<B> {
        let input_layer = nn::conv::Conv2dConfig::new([10, 10], [4, 1]).init(device);
        let positional_encoding = nn::PositionalEncodingConfig::new(256).init(device);
        let transformer_encoder =
            nn::transformer::TransformerEncoderConfig::new(256, 1024, 8, 4).init(device);
        let lstm1 = nn::LstmConfig::new(transformer_encoder.d_model, 512, true).init(device);
        let lstm2 = nn::LstmConfig::new(lstm1.d_hidden, 256, true).init(device);
        let transformer_decoder =
            nn::transformer::TransformerDecoderConfig::new(256, 1024, 8, 4).init(device);
        let output_layer = nn::LinearConfig::new(256, 10).init(device);

        Model {
            input_layer,
            positional_encoding,
            transformer_encoder,
            lstm1,
            lstm2,
            transformer_decoder,
            output_layer,
        }
    }
}

impl<B: Backend> Model<B> {
    pub fn forward(&self, item: BetBatch<B>) -> Tensor<B, 2> {
        let device = &self.devices()[0];

        let inputs = item.inputs.to_device(device);

        let inputs = self.input_layer.forward(inputs);
        let inputs = inputs.flatten(2, 3);

        let pos_encode = self.positional_encoding.forward(inputs.clone());
        let combined = (inputs.clone() + pos_encode) / 2;
        let te_input = nn::transformer::TransformerEncoderInput::new(combined);
        let encoded = self.transformer_encoder.forward(te_input);

        let lstm = self.lstm1.forward(encoded.clone(), None);
        let lstm = self.lstm2.forward(lstm.0, None);

        let te_decode = nn::transformer::TransformerDecoderInput::new(
            Tensor::random(
                Shape::new(encoded.clone().dims()),
                Distribution::Normal(-1., 1.),
                device,
            ),
            lstm.0.clone(),
        );
        let decoded = self.transformer_decoder.forward(te_decode);
        let combined = (lstm.0 + decoded) / 2;

        self.output_layer.forward(combined).flatten(1, 2)
    }
}
