pub mod method_node {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use syn::{punctuated::Punctuated, token::Comma, FnArg, Ident, ReturnType, TraitItemFn};
    use crate::errors::errors;
    use crate::input_node::input_node::{get_input_param, InputParam, InputTypes};

    pub struct MethodNode {
        ident: Ident,
        params: Punctuated<FnArg, Comma>,
        input_params: Vec<InputParam>,
        return_type: ReturnType,
    }

    impl MethodNode {
        pub fn new(method: TraitItemFn) -> Result<Self, errors::Error> {
            let ident = method.sig.ident;
            let params = method.sig.inputs;
            let return_type = method.sig.output;
            let sanitised = Self::sanitise_params(params.clone())?;
            Ok(MethodNode { ident, params, return_type, input_params: sanitised })
        }

        pub fn sanitise_params(params: Punctuated<FnArg, Comma>) -> Result<Vec<InputParam>, errors::Error> {
            let mut sanitised = vec![];
            let mut optional_register = vec![];
            for p in params {
                let input_param = get_input_param(p)?;
                match input_param.input_type {
                    InputTypes::OptionType => {
                        optional_register.push(input_param);
                    },
                    _ => {
                        sanitised.push(input_param);
                    }
                }
            }
            sanitised.append(&mut optional_register);
            Ok(sanitised)
        }

        pub fn to_method(&self, ipc_buffer_name: String, msg_name: String, label: usize) -> TokenStream {
            let marshal_code = self.marshal_code(ipc_buffer_name, msg_name, label);
            let method_ident = self.ident.clone();
            let method_params = self.params.clone();
            let method_return_type = self.return_type.clone();
            quote! {
                fn #method_ident(#method_params) #method_return_type {
                    #marshal_code
                }
            }
        }

        pub fn marshal_code(&self, ipc_buffer_name: String, msg_name: String, label: usize) -> TokenStream {
            let ipc_buffer_name = format_ident!("{}", ipc_buffer_name);
            let msg_name = format_ident!("{}", msg_name);
            let (marshalls, msg_len) = self.marshal_all_inputs(ipc_buffer_name.to_string());
            quote! {
                sel4::with_ipc_buffer_mut(|#ipc_buffer_name| {
                    #(#marshalls)*
                });
                let mut #msg_name = sel4::MessageInfoBuilder::default()
                    .label(#label)
                    .length(#msg_len)
                    .build();
            }
        }

        pub fn marshal_all_inputs(&self, buffer_name: String) -> (Vec<TokenStream>, usize) {
            let mut marshalls = vec![];
            let mut msg_index = 0;
            for param in &self.input_params {
                marshalls.push(param.get_marshal_code(String::from(buffer_name.clone()), &mut msg_index));
            }
            (marshalls, msg_index)
        }

        pub fn to_unimplemented(method: TraitItemFn) -> TokenStream {
            let method_ident = method.sig.ident;
            let method_params = method.sig.inputs;
            let method_return_type = method.sig.output;
            quote! {
                fn #method_ident(#method_params) #method_return_type {
                    unimplemented!()
                }
            }
        }
    }

}


