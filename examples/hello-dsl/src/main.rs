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

#[allow(non_snake_case)]
pub fn LayerView() -> ViewLayerBuilder {
    ViewLayerBuilder::default()
}
fn main() {
    if let Ok(root) = ::syn::parse2::<LayerItem>(quote!(
        LayerView()
            .size((Point { x: 100.0, y: 100.0 }, None))
            .background_color((
                PaintColor::Solid {
                    color: Color::new_hex("#ffffff"),
                },
                None,
            )){
                LayerView()
                    .size((Point { x: 100.0, y: 100.0 }, None))
               //     {
               //         // for i in 0..10 {
               //         Layer("text")
               //         // }
               //     }
            }
    )) {
        // println!("{}", root);
        let code = code_gen(&root);
        println!("{}", code);
    }
    // let engine = LayersEngine::new();

    let tree = layers! {
        LayerView()
        .size((Point { x: 100.0, y: 100.0 }, None))
        .background_color((
            PaintColor::Solid {
                color: Color::new_hex("#ffffff"),
            },
            None,
        )){
            LayerView()
                .size((Point { x: 100.0, y: 100.0 }, None))
           //     {
           //         // for i in 0..10 {
           //         Layer("text")
           //         // }
           //     }
        }
    };
}
