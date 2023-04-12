use layers::prelude::*;
use layers_dsl::layers;
use layers_dsl_core::{code_gen, LayerItem};
use syn::__private::quote::quote;

macro_rules! engine {
    ($init:expr) => {
        engine.new_layer();
        // $init
    };
}
struct LayersBuilder {}
impl LayersBuilder {
    pub fn Layer(&self, name: &str) {
        println!("Layer: {}", name);
    }
}
fn main() {
    if let Ok(root) = ::syn::parse2::<LayerItem>(quote!(
        Layer()
            .size(100, 100)
            .background("red")
        {
            Layer()
            {
                for i in 0..10 {
                    Layer("text")
                }
            }
        }
    )) {
        // println!("{}", root);
        let code = code_gen(&root);
        println!("{}", code);
    }
    let engine = LayersEngine::new();

    let tree = layers! {
    Layer()
    .size(100.0, 100.0)
    .background("red")
    // {
    //     Layer()
    //     {
    //         Layer()
    //     }
    // }

    };
}
