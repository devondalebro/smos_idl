pub mod method_node {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use syn::{punctuated::Punctuated, token::Comma, FnArg, Ident, ReturnType, TraitItemFn};
    use crate::errors::errors;
    use crate::input_node::input_node::get_input_param;

    pub struct MethodNode {
        ident: Ident,
        params: Punctuated<FnArg, Comma>,
        return_type: ReturnType,
    }

    impl MethodNode {
        pub fn new(method: TraitItemFn) -> Self {
            let ident = method.sig.ident;
            let params = method.sig.inputs;
            let return_type = method.sig.output;
            MethodNode { ident, params, return_type }
        }

        pub fn to_method(&self, ipc_buffer_name: String, msg_name: String, label: usize) -> Result<TokenStream, errors::Error> {
            let marshal_code = self.marshal_code(ipc_buffer_name, msg_name, label)?;
            let method_ident = self.ident.clone();
            let method_params = self.params.clone();
            let method_return_type = self.return_type.clone();
            Ok(quote! {
                fn #method_ident(#method_params) #method_return_type {
                    #marshal_code
                }
            })
        }

        pub fn marshal_code(&self, ipc_buffer_name: String, msg_name: String, label: usize) -> Result<TokenStream, errors::Error> {
            let ipc_buffer_name = format_ident!("{}", ipc_buffer_name);
            let msg_name = format_ident!("{}", msg_name);
            let (marshalls, msg_len) = self.marshal_all_inputs(ipc_buffer_name.to_string())?;
            Ok(quote! {
                sel4::with_ipc_buffer_mut(|#ipc_buffer_name| {
                    #(#marshalls)*
                });
                let mut #msg_name = sel4::MessageInfoBuilder::default()
                    .label(#label)
                    .length(#msg_len)
                    .build();
            })
        }

        pub fn to_unimplemented(&self) -> TokenStream {
            let method_ident = self.ident.clone();
            let method_params = self.params.clone();
            let method_return_type = self.return_type.clone();
            quote! {
                fn #method_ident(#method_params) #method_return_type {
                    unimplemented!()
                }
            }
        }

        pub fn marshal_all_inputs(&self, buffer_name: String) -> Result<(Vec<TokenStream>, usize), errors::Error> {
            let mut marshalls = vec![];
            let mut msg_index = 0;
            for param in self.params.clone() {
                let input_param = get_input_param(param)?;
                marshalls.push(input_param.get_marshal_code(String::from(buffer_name.clone()), &mut msg_index));
            }
            Ok((marshalls, msg_index))
        }
    }

}


