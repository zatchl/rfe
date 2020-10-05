use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit, LitByteStr, Meta};

#[proc_macro_derive(RfeMessage, attributes(prefix, optional))]
pub fn derive_rfe_message(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let struct_name = &derive_input.ident;
    let struct_fields = match derive_input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!(),
        },
        _ => panic!(),
    };
    let parsed_fields: Vec<_> = struct_fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            match field.attrs.first() {
                Some(attr) if attr.path.is_ident("optional") => {
                    quote! {#ident: parse_field(fields.next()).ok()}
                }
                _ => quote! {#ident: parse_field(fields.next())?},
            }
        })
        .collect();
    let message_prefix = derive_input
        .attrs
        .into_iter()
        .find_map(|attr| match attr.parse_meta().unwrap() {
            Meta::NameValue(mnv) => match mnv.lit {
                Lit::ByteStr(prefix) => Some(prefix),
                Lit::Str(prefix) => Some(LitByteStr::new(prefix.value().as_bytes(), prefix.span())),
                _ => None,
            },
            _ => None,
        })
        .expect("Missing 'prefix' attribute");

    (quote! {
        fn parse_field<T>(field: Option<&[u8]>) -> Result<T, crate::messages::ParseMessageError>
        where
            T: std::str::FromStr,
            crate::messages::ParseMessageError: From<T::Err>,
        {
            Ok(T::from_str(
                std::str::from_utf8(field.ok_or_else(|| crate::messages::ParseMessageError::MissingField)?)?.trim(),
            )?)
        }

        impl std::convert::TryFrom<&[u8]> for #struct_name {
            type Error = crate::messages::ParseMessageError;

            fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
                if bytes.starts_with(#message_prefix) {
                    let mut fields = bytes.get(#message_prefix.len()..).ok_or_else(|| crate::messages::ParseMessageError::MissingField)?.split(|&byte| byte == b',');
                    Ok(#struct_name {
                        #(#parsed_fields),*
                    })
                } else {
                    Err(crate::messages::ParseMessageError::InvalidData)
                }
            }
        }

        impl crate::messages::RfeMessage for #struct_name {
            const PREFIX: &'static [u8] = #message_prefix;
        }
    })
    .into()
}
