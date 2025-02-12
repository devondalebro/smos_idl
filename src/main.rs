use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, ItemTrait, TraitItem, TraitItemFn};
use std::fs::File as StdFile;
use std::io::{Read, Write};
mod method_node;
use prettyplease;
mod input_node;
mod errors;

fn main() {
    let input_path = "src/input.rs";
    let mut file = StdFile::open(input_path).expect("Failed to open input file");
    let mut content = String::new();
    file.read_to_string(&mut content).expect("Failed to read in file");
    let ast = syn::parse_file(&content).expect("Failed to parse content");

    let mut implementations = vec![];

    for item in &ast.items {
        match item {
            Item::Trait(item_trait) => {
                implementations = parse_trait(item_trait.clone());
            },
            _ => {
                
            }
        }
    }

    let output_code = quote! {
        #(#implementations)*
    };
    let formatted_code = format_rust_code(output_code.to_string());

    let mut file = StdFile::create("target/output.rs").expect("Failed to create output file");
    writeln!(file, "{}", formatted_code).expect("Failed to write to output file");
}

fn parse_trait(item_trait: ItemTrait) -> Vec<TokenStream> {
    println!("Parsing module {}", item_trait.ident.to_string());
    let mut implementations = vec![];
    item_trait.items.iter().for_each(|item| {
        match item {
            TraitItem::Fn(method) => {
                println!("Method encountered {}", method.sig.ident.to_string());
                implementations.push(parse_method(method.clone()));
                println!("")
            },
            _ => {
                unimplemented!()
            }
        }
    });

    implementations
}
fn format_rust_code(code: String) -> String {
    let syntax_tree: syn::File = syn::parse_str(&code).expect("Failed to parse TokenStream into syntax tree");
    prettyplease::unparse(&syntax_tree)
}

fn parse_method(method: TraitItemFn) -> TokenStream {
    let method_node = method_node::method_node::MethodNode::new(method);
    match method_node.to_method(String::from("ipc_buf"), String::from("msg"), 0) {
        Ok(method_impl) => method_impl,
        Err(_) => method_node.to_unimplemented()
    }
}