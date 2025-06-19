use burn::{
    nn::{
        conv::{
            Conv1d, Conv1dConfig, Conv2d, Conv2dConfig, ConvTranspose2d, ConvTranspose2dConfig,
        },
        transformer::{PositionWiseFeedForward, PositionWiseFeedForwardConfig},
        Dropout, DropoutConfig, Gelu, Linear, LinearConfig, Lstm, LstmConfig, PaddingConfig1d,
        Relu,
    },
    prelude::*,
    tensor::activation::{sigmoid, softmax},
};

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    input_layer: Conv1d<B>,
    hidden_layer: Conv1d<B>,
    dropout_layer: Dropout,
    hidden_layer_2: Linear<B>,
    output_layer: PositionWiseFeedForward<B>,
}

#[derive(Config, Debug)]
pub struct ModelConfig {}

impl ModelConfig {
    pub fn init<B: Backend>(&self, device: &B::Device) -> Model<B> {
        Model {
            input_layer: Conv1dConfig::new(10, 64, 3)
                .with_padding(PaddingConfig1d::Same)
                .init(device),
            hidden_layer: Conv1dConfig::new(64, 3, 15)
                .with_padding(PaddingConfig1d::Same)
                .init(device),
            dropout_layer: DropoutConfig::new(0.3).init(),
            hidden_layer_2: LinearConfig::new(6144, 10001).init(device),
            output_layer: PositionWiseFeedForwardConfig::new(10001, 5000).init(device),
        }
    }
}

impl<B: Backend> Model<B> {
    pub fn forward(&self, input_server_seed_hash_next_roll: Tensor<B, 3>) -> Tensor<B, 2> {
        let x = self.input_layer.forward(input_server_seed_hash_next_roll);
        let x = self.hidden_layer.forward(x);
        let x = self.dropout_layer.forward(x);
        let x = self.hidden_layer_2.forward(x.flatten::<2>(1, 2));
        let x = self.output_layer.forward(x);

        x
    }
}

