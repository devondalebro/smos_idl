pub mod input_node {
    use std::env::Args;

    use proc_macro2::TokenStream;
    use quote::{format_ident, quote, ToTokens};
    use syn::{parse_str, AngleBracketedGenericArguments, FnArg, GenericArgument, Ident, PatType, PathArguments, Type};
    use crate::errors::errors::Error;

    pub trait InputType {
        fn into_ipc_buf(&self, ident: Ident, ty: Type, buffer_name: Ident, msg_index: &mut usize) -> TokenStream;
        fn type_parses(&self) -> Vec<Type>;
        fn is_type(&self, ty: syn::Type) -> bool {
            self.type_parses().iter().any(|t| t.clone() == ty)
        }
    }

    fn all_input_types() -> Vec<Box<dyn InputType>> {
        vec![
            Box::from(NumberType {}),
            Box::from(BoolType {}),
            Box::from(AbsoluteCPtrType {}),
            Box::from(LocalHandleType {}),
            Box::from(OptionType {})
        ]
    }

    pub fn get_input_type(param: FnArg) -> Result<Box<dyn InputType>, Error> {
        let ty = if let FnArg::Typed(
            PatType { ref ty, .. }
        ) = param {
            ty.as_ref().clone()
        } else {
            return Err(Error::InvalidArg(param.to_token_stream().to_string()));
        };
        match_type(ty)
    }

    pub fn match_type(ty: Type) -> Result<Box<dyn InputType>, Error> {
        for input_ty in all_input_types() {
            if input_ty.is_type(ty.clone()) {
                return Ok(input_ty);
            }
        }
        Err(Error::InvalidArg(ty.to_token_stream().to_string()))
    }
    
    pub fn get_input_param(param: FnArg) -> Result<InputParam, Error> {
        let (var_name, ty) = if let FnArg::Typed(
            PatType { ref pat, ref ty, .. }
        ) = param {
            let var_name = if let syn::Pat::Ident(pat_ident) = pat.as_ref() {
                pat_ident.ident.to_string()
            } else {
                return Err(Error::InvalidArg(param.to_token_stream().to_string()));
            };
            let ty_inner = ty.as_ref().clone();
            (var_name, ty_inner)
        } else {
            return Err(Error::InvalidArg(param.to_token_stream().to_string()));
        };
        let input_type = self::get_input_type(param)?;
        Ok(
            InputParam { ident: var_name, ty, input_type }
        )
    }

    pub struct InputParam {
        ident: String,
        ty: Type,
        input_type: Box<dyn InputType>
    }

    impl InputParam {
        pub fn get_marshal_code(&self, ipc_buffer_name: String, msg_index: &mut usize) -> TokenStream {
            self.input_type.into_ipc_buf(
                format_ident!("{}", self.ident.clone()), 
                self.ty.clone(), 
                format_ident!("{}", ipc_buffer_name), 
                msg_index
            )
        }
    }

    struct NumberType {}
    impl InputType for NumberType {
        fn into_ipc_buf(&self, ident: Ident, _: Type, buffer_name: Ident, msg_index: &mut usize) -> TokenStream {
            let ident = format_ident!("{}", ident);
            let buffer_name = format_ident!("{}", buffer_name);
            let idx = (*msg_index).clone();
            let ret = quote! {
                #buffer_name.msg_regs_mut()[#idx] = #ident as u64;
            };
            *msg_index += 1;
            ret
        }

        fn type_parses(&self) -> Vec<syn::Type> {
            vec![
                parse_str("usize").expect("Couldn't parse"),
                parse_str("u8").expect("Couldn't parse"),
                parse_str("u64").expect("Couldn't parse"),
            ]
        }
    }

    struct BoolType {}
    impl InputType for BoolType {
        fn into_ipc_buf(&self, ident: Ident, _: Type, buffer_name: Ident, msg_index: &mut usize) -> TokenStream {
            let idx = (*msg_index).clone();
            let ret = quote! {
                #buffer_name.msg_regs_mut()[#idx] = #ident.into();
            };
            *msg_index += 1;
            ret
        }

        fn type_parses(&self) -> Vec<syn::Type> {
            vec![
                parse_str("bool").expect("Couldn't parse"),
            ]
        }
    }

    struct AbsoluteCPtrType {}
    impl InputType for AbsoluteCPtrType {
        fn into_ipc_buf(&self, ident: Ident, _: Type, buffer_name: Ident, _msg_index: &mut usize) -> TokenStream {
            quote! {
                #buffer_name.set_recv_slot(#ident);
            }
        }

        fn type_parses(&self) -> Vec<syn::Type> {
            vec![
                parse_str("&AbsoluteCPtr").expect("Couldn't parse")
            ]
        }
    }

    struct LocalHandleType {}
    impl InputType for LocalHandleType {
        fn into_ipc_buf(&self, ident: Ident, _: Type, buffer_name: Ident, msg_index: &mut usize) -> TokenStream {
            let idx = (*msg_index).clone();
            let ret = quote! {
                #buffer_name.msg_regs_mut()[#idx] = #ident.idx as u64;
            };
            *msg_index += 1;
            ret 
        }
        fn type_parses(&self) -> Vec<syn::Type> {
            vec![
                parse_str("&LocalHandle<WindowHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<ViewHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<ObjectHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<ConnectionHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<PublishHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<ReplyHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<HandleCapHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<ProcessHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<ConnRegistrationHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<WindowRegistrationHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<IRQRegistrationHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<ChannelAuthorityHandle>").expect("Couldn't parse"),
                parse_str("&LocalHandle<ChannelHandle>").expect("Couldn't parse"),
            ]
        }
    }

    struct OptionType {}
    impl InputType for OptionType {
        // the code here is really shit
        fn into_ipc_buf(&self, ident: Ident, ty: Type, buffer_name: Ident, msg_index: &mut usize) -> TokenStream {
            let inner_type = if let Ok(inner_type) = self.get_inner_type(ty.clone()) {
                inner_type
            } else {
                unimplemented!() // this will never happen
            };

            let inner_input_type = if let Ok(inner_input_type) = match_type(inner_type.clone()) {
                inner_input_type
            } else {
                unimplemented!() // this will never happen
            };
            
            let inner_ident = format_ident!("{}_inner", ident);
            let inner_type_marshall = inner_input_type.into_ipc_buf(inner_ident.clone(), inner_type, buffer_name, msg_index);

            let ret = quote! {
                if #ident.is_some() {
                    let #inner_ident = #ident.unwrap();
                    #inner_type_marshall
                }
            };
            ret
        }

        fn type_parses(&self) -> Vec<Type> {
            unimplemented!()
            // this will never be called
        }
        
        fn is_type(&self, ty: Type) -> bool {
            if let Type::Path(ref type_path) = ty {
                if let Some(seg) = type_path.path.segments.last() {
                    if seg.ident != "Option" {
                        return false;
                    }                 
                    if let Ok(inner_ty) = self.get_inner_type(ty.clone()) {
                        if let Ok(_) = match_type(inner_ty) {
                            return true;
                        }
                    }
                    return false;
                } else {
                    false
                }
            } else {
                false
            }
        }
    }

    impl OptionType {
        fn get_inner_type(&self, ty: Type) -> Result<Type, Error> {
            if let Type::Path(ref type_path) = ty {
                if let Some(seg) = type_path.path.segments.last() {
                    if let PathArguments::AngleBracketed(ref bracketed_args) = seg.arguments {
                        if bracketed_args.args.len() != 1 {
                            return Err(Error::InvalidArg(ty.to_token_stream().to_string()));
                        }
                        let gen_arg = bracketed_args.args.first().unwrap();
                        match gen_arg {
                            GenericArgument::Type(inner_ty) => {
                                Ok(inner_ty.clone())
                            },
                            _ => Err(Error::InvalidArg(ty.to_token_stream().to_string()))
                        }
                    } else {
                        Err(Error::InvalidArg(ty.to_token_stream().to_string()))
                    }
                } else {
                    Err(Error::InvalidArg(ty.to_token_stream().to_string()))
                }
            } else {
                Err(Error::InvalidArg(ty.to_token_stream().to_string()))
            }
        }
    }
}