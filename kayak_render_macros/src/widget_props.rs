use proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use proc_macro_error::{emit_error, emit_warning};
use quote::{format_ident, quote, ToTokens};
use syn::{AttributeArgs, Data, DeriveInput, Field, Fields, Generics, ItemStruct, Meta, NestedMeta, parse_macro_input, spanned::Spanned};
use syn::parse::Parser;
use crate::attribute::Attribute;

use crate::get_core_crate;

/// The ident for the props helper attribute (`#[prop_field(Children)]`)
const PROPS_HELPER_IDENT: &str = "prop_field";

const PROP_CHILDREN: CommonProps = CommonProps {
    key: "Children",
    ident: "children",
};
const PROP_STYLE: CommonProps = CommonProps {
    key: "Styles",
    ident: "styles",
};
const PROP_ON_EVENT: CommonProps = CommonProps {
    key: "OnEvent",
    ident: "on_event",
};
const PROP_FOCUSABLE: CommonProps = CommonProps {
    key: "Focusable",
    ident: "focusable",
};

#[derive(Default)]
struct CommonProps {
    key: &'static str,
    ident: &'static str,
}

#[derive(Default)]
struct PropsHelpers {
    children_ident: Option<Ident>,
    styles_ident: Option<Ident>,
    on_event_ident: Option<Ident>,
    focusable_ident: Option<Ident>,
}

pub(crate) fn derive_widget_props(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, generics, attrs, ..
    } = parse_macro_input!(input);

    let has_widget_props_attr = attrs.iter().any(|attr| {
        if let Ok(meta) = attr.parse_meta() {
            let path = meta.path().to_token_stream().to_string();
            path.ends_with("widget_props")
        } else {
            false
        }
    });

    if has_widget_props_attr {
        // Has the #[widget_props(...)] attribute -> Let that macro handle the impl
        return TokenStream::new();
    }

    let helpers = process_data(data);

    impl_widget_props(&ident, &generics, helpers)
}

pub(crate) fn add_widget_props(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut prop_struct = parse_macro_input!(item as ItemStruct);
    let args = parse_macro_input!(attrs as AttributeArgs);
    let helpers = insert_common_props(&mut prop_struct, args);

    emit_warning!(prop_struct.span(), "Attrs: {:?}", prop_struct.attrs);

    let has_derive = prop_struct.attrs.iter().any(|attr| {
        if let Ok(meta) = attr.parse_meta() {
            let path = meta.path().to_token_stream().to_string();
            if !path.ends_with("derive") {
                return false;
            }

            if let Meta::List(list) = meta {
                return list.nested.iter().any(|nested| {
                    if let NestedMeta::Meta(meta) = nested {
                        let path = meta.path().to_token_stream().to_string();
                        return path.ends_with("WidgetProps");
                    }
                    false
                });
            }
        }

        false
    });

    let impl_stream = if has_derive {
        // Derive has definitely not been applied yet -> Let it
        proc_macro2::TokenStream::new()
    } else {
        // Derive may or may not have been applied already -> Implement (it should have detected this macro and skipped)
        proc_macro2::TokenStream::from(impl_widget_props(&prop_struct.ident, &prop_struct.generics, helpers))
    };

    let output = quote! {
        #prop_struct
        #impl_stream
    };

    output.into()
}

fn impl_widget_props(ident: &Ident, generics: &Generics, helpers: PropsHelpers) -> TokenStream {
    let children_return = quote_clone_field(helpers.children_ident);
    let styles_return = quote_clone_field(helpers.styles_ident);
    let on_event_return = quote_clone_field(helpers.on_event_ident);
    let focusable_return = quote_clone_field(helpers.focusable_ident);

    let kayak_core = get_core_crate();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics #kayak_core::WidgetProps for #ident #ty_generics #where_clause {
            fn get_children(&self) -> Option<#kayak_core::Children> {
                #children_return
            }

            fn get_styles(&self) -> Option<#kayak_core::styles::Style> {
                #styles_return
            }

            fn get_on_event(&self) -> Option<#kayak_core::OnEvent> {
                #on_event_return
            }

            fn get_focusable(&self) -> Option<bool> {
                #focusable_return
            }

        }
    };

    output.into()
}

fn insert_common_props(prop_struct: &mut ItemStruct, args: AttributeArgs) -> PropsHelpers {
    let mut helpers = PropsHelpers::default();
    let kayak_core = get_core_crate();
    for arg in args {
        match arg {
            NestedMeta::Meta(meta) => {
                match meta {
                    Meta::Path(path) => {
                        if path.segments.len() > 1 {
                            emit_error!(path.span(), "Invalid argument: {}", path.to_token_stream().to_string())
                        } else {
                            let ident = path.get_ident().unwrap();
                            match ident.to_string().as_str() {
                                key if key == PROP_CHILDREN.key => {
                                    push_field_raw(
                                        prop_struct, key, PROP_CHILDREN.ident, quote!(Option<#kayak_core::Children>),
                                    );
                                    helpers.children_ident = Some(Ident::new(PROP_CHILDREN.ident, ident.span()));
                                }
                                key if key == PROP_STYLE.key => {
                                    push_field_raw(
                                        prop_struct, key, PROP_STYLE.ident, quote!(Option<#kayak_core::styles::Style>),
                                    );
                                    helpers.styles_ident = Some(Ident::new(PROP_STYLE.ident, ident.span()));
                                }
                                key if key == PROP_ON_EVENT.key => {
                                    push_field_raw(
                                        prop_struct, key, PROP_ON_EVENT.ident, quote!(Option<#kayak_core::OnEvent>),
                                    );
                                    helpers.on_event_ident = Some(Ident::new(PROP_ON_EVENT.ident, ident.span()));
                                }
                                key if key == PROP_FOCUSABLE.key => {
                                    push_field_raw(
                                        prop_struct, key, PROP_FOCUSABLE.ident, quote!(Option<bool>),
                                    );
                                    helpers.focusable_ident = Some(Ident::new(PROP_FOCUSABLE.ident, ident.span()));
                                }
                                err => emit_error!(ident.span(), "Invalid attribute: {}", err)
                            }
                        }
                    }
                    err => emit_error!(err.span(), "Invalid argument: {:?}", err)
                }
            }
            err => emit_error!(err.span(), "Invalid argument: {:?}", err)
        }
    }

    helpers
}

fn push_field_raw(prop_struct: &mut ItemStruct, field_key: &str, field_name: &str, field_type: proc_macro2::TokenStream) {
    let attr = quote::format_ident!("{}", PROPS_HELPER_IDENT);
    let key = quote::format_ident!("{}", field_key);
    let ident = quote::format_ident!("{}", field_name);
    let field = quote::quote! {
        #[#attr(#key)]
        pub #ident: #field_type
    };
    push_field(prop_struct, field);
}

fn push_field(prop_struct: &mut ItemStruct, field: proc_macro2::TokenStream) {
    let span = prop_struct.span();
    match &mut prop_struct.fields {
        Fields::Named(fields) => {
            let field = Field::parse_named.parse2(quote! {
                #field
            }).unwrap();
            fields.named.push(field);
        }
        Fields::Unit => emit_error!(span, "Cannot be unit struct"),
        Fields::Unnamed(fields) => emit_error!(fields.span(), "Cannot be tuple struct"),
    }
}

/// Checks for the widget props naming convention (`<Widget Name>Props`), emitting a warning if not followed
fn check_naming_convention(ident: &Ident) {
    let name = ident.to_string();
    if !name.ends_with("Props") {
        emit_warning!(
            ident.span(),
            "Struct should be named according to the convention \"<Widget Name>Props\" when implementing WidgetProps"
        );
    }
}

/// Processes all fields of the given struct to collect the helper attribute data
///
/// Attributes are processed in order and may overwrite previous attributes of the same type. This
/// results in the returned `PropsHelpers` reflecting the last instance of each respective helper.
fn process_data(data: Data) -> PropsHelpers {
    let mut helpers = PropsHelpers::default();

    match data {
        Data::Struct(data) => {
            for field in data.fields {
                process_field(field, &mut helpers);
            }
        }
        Data::Union(data) => {
            for field in data.fields.named {
                process_field(field, &mut helpers);
            }
        }
        Data::Enum(data) => emit_error!(data.enum_token.span(), "Cannot derive WidgetProp for enum"),
    }

    helpers
}

/// Process a field to collect the helper attribute
fn process_field(field: Field, props: &mut PropsHelpers) {
    for attr in field.attrs {
        if let Ok(meta) = attr.parse_meta() {
            if let Meta::List(meta) = meta {
                if !meta.path.is_ident(PROPS_HELPER_IDENT) {
                    continue;
                }

                for nested in meta.nested {
                    let ident = match nested {
                        NestedMeta::Meta(meta) => meta.path().get_ident().cloned(),
                        err => {
                            emit_error!(err.span(), "Invalid attribute: {:?}", err);
                            None
                        }
                    };

                    if let Some(ident) = ident {
                        match ident.to_string().as_str() {
                            key if key == PROP_CHILDREN.key => props.children_ident = field.ident.clone(),
                            key if key == PROP_STYLE.key => props.styles_ident = field.ident.clone(),
                            key if key == PROP_ON_EVENT.key => props.on_event_ident = field.ident.clone(),
                            key if key == PROP_FOCUSABLE.key => props.focusable_ident = field.ident.clone(),
                            err => emit_error!(ident.span(), "Invalid attribute: {}", err)
                        }
                    }
                }
            }
        }
    }
}

fn quote_clone_field(field_ident: Option<Ident>) -> proc_macro2::TokenStream {
    if let Some(field_ident) = field_ident {
        quote! {
            self.#field_ident.clone()
        }
    } else {
        quote! {
            None
        }
    }
}