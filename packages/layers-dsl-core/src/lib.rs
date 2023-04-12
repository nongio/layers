use core::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

use ::syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    Result, // explicitly shadow it
    *,
};
use proc_macro2::{Ident, Span, TokenStream};
use syn::__private::quote::quote;

static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
pub mod prelude;
pub struct LayerItem {
    ident: Ident,
    init: TokenStream,
    block: TokenStream,
    children: Vec<LayerItem>,
}
impl fmt::Display for LayerItem {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let ident = &self.ident;
        let init_code = &self.init;
        self.children.iter().for_each(|child| {
            fmt.write_str("\n");
            fmt.write_str(&child.to_string());
            fmt.write_str("\n");
        });
        let return_string = quote!(let #ident = #init_code).to_string();
        fmt.write_str(&return_string);
        self.children.iter().for_each(|child| {
            let child_ident = &child.ident;
            let return_string = quote!(#ident . add_child( #child_ident )).to_string();
            fmt.write_str("\n");
            fmt.write_str(&return_string);
        });
        Ok(())
    }
}
fn parse_expr(call: &Expr, varname: &Ident) -> (Result<TokenStream>, Result<TokenStream>) {
    let (mut init, mut block): (Result<TokenStream>, Result<TokenStream>) =
        (Ok(quote! {}), Ok(quote! {}));
    match call {
        Expr::MethodCall(method_call) => {
            // Ok(Expr::MethodCall(method_call))
            let ExprMethodCall {
                attrs,
                receiver,
                dot_token,
                method,
                turbofish,
                paren_token,
                args,
            } = method_call.clone();
            // println!("attrs: {:?}", attrs);
            // println!("receiver: {:?}", receiver);
            // println!("dot_token: {:?}", dot_token);
            // println!("method: {:?}", method);
            // println!("turbofish: {:?}", turbofish);
            // println!("paren_token: {:?}", paren_token);
            // println!("args: {:?}", args);

            init = Ok(quote! {
                ViewLayerTreeBuilder::default()
                .root(Arc::new(
                    ViewLayerBuilder::default()
                    #method_call;
                ))
            });
        }
        Expr::Call(call) => {
            // destructuring the call
            let ExprCall {
                attrs,
                paren_token,
                func,
                args,
            } = call.clone();
            // println!("func: {:?}", func);
            // println!("args: {:?}", args);
            // println!("attrs: {:?}", attrs);
            // println!("paren_token: {:?}", paren_token);
            // let variable_name = std::fmt::format(format_args!("engine_{:?}!", func));
            // let varname = syn::Ident::new(&variable_name, Span::call_site());

            init = Ok(quote! {
                ViewLayerTreeBuilder::default()
                .root(Arc::new(
                    ViewLayerBuilder::default()
                    #call;
                ));
            });
            // Ok(Expr::Call(call))
        }
        Expr::ForLoop(forloop) => {
            let ExprForLoop {
                attrs,
                body,
                expr,
                for_token,
                in_token,
                label,
                pat,
            } = &forloop;
            // println!("attrs {:?}", attrs);
            // println!("body {:?}", body);
            // println!("expr {:?}", expr);
            // println!("for_token {:?}", for_token);
            // println!("in_token {:?}", in_token);
            // println!("label {:?}", label);
            // println!("pat {:?}", pat);
            let loopvarname = syn::Ident::new("loop_layer", Span::call_site());
            let mut inside_loop: Vec<TokenStream> = Vec::new();
            body.stmts.iter().for_each(|stmt| match stmt {
                Stmt::Local(local) => {
                    println!("local {:?}", local);
                }
                Stmt::Item(item) => {
                    println!("item {:?}", item);
                }
                Stmt::Expr(expr) => {
                    // println!("expr {:?}", expr);

                    let (init, code) = parse_expr(expr, &loopvarname);
                    if let Ok(code) = code {
                        inside_loop.push(code);
                    }
                    if let Ok(init) = init {
                        inside_loop.push(init);
                    }
                }
                Stmt::Semi(expr, semi) => {
                    println!("semi {:?}", semi);
                    println!("expr {:?}", expr);
                }
            });

            // let body = Expr::parse(input)
            // let parsed_children = parse_layer_item_children(body);
            // let inside_loop = inside_loop.iter();
            block = Ok(quote!(
                #for_token #pat #in_token #expr {
                    #(#inside_loop)*
                    #varname.add_child(loop_layer)
                }
            ));
        }
        _ => {
            // Err(syn::Error::new(Span::call_site(), "noooo"));
        }
    };
    (init, block)
}

fn parse_layer_item(input: ParseStream<'_>) -> Result<LayerItem> {
    CALL_COUNT.fetch_add(1, Ordering::SeqCst);
    let variable_name =
        std::fmt::format(format_args!("layer_{}", CALL_COUNT.load(Ordering::SeqCst)));
    let varname = syn::Ident::new(&variable_name, Span::call_site());
    let call: Expr = input.parse()?;
    let (init_code, block) = parse_expr(&call, &varname);

    if let Ok(init_code) = init_code {
        let mut children: Vec<LayerItem> = Vec::new();
        let parsed_children = parse_layer_item_children(input);
        if let Ok(parsed_children) = parsed_children {
            children = parsed_children;
        }
        let block = block.unwrap_or(quote! {});
        Ok(LayerItem {
            ident: varname,
            init: init_code,
            block,
            children,
        })
    } else {
        Err(syn::Error::new(Span::call_site(), "li morté"))
    }
}

fn parse_layer_item_children(input: ParseStream<'_>) -> Result<Vec<LayerItem>> {
    let mut children: Vec<LayerItem> = Vec::new();
    if input.peek(token::Brace) {
        let content;
        braced!(content in input);

        loop {
            if let Ok(child) = parse_layer_item(&content) {
                children.push(child);
            } else {
                break;
            }
        }
    }
    Ok(children)
}
impl Parse for LayerItem {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let root = parse_layer_item(input)?;
        Ok(root)
    }
}

fn layer_to_code(
    LayerItem {
        ident,
        init,
        block,
        children,
    }: &LayerItem,
) -> proc_macro2::TokenStream {
    let cc: Vec<proc_macro2::TokenStream> = children
        .iter()
        .map(|child| {
            let child_code = layer_to_code(child);
            let child_ident = &child.ident;
            quote! {
                #child_code
                // #ident . add_child( #child_ident );
            }
        })
        .collect();

    quote! {

        // #block
        #init
        // #(#cc )*
    }
}
pub fn code_gen(li: &LayerItem) -> proc_macro2::TokenStream {
    let code = layer_to_code(&li);
    let root = &li.ident;
    quote! {
        {
            // let BUILDER = layers::engine::LayersBuilder::new();
            // compile(|state| -> something {
                #code
                // #root
            // });
        }
    }
    .into()
}
