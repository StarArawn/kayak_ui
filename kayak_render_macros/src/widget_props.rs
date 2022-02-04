use proc_macro::TokenStream;

use proc_macro2::{Ident};
use proc_macro_error::{emit_error, emit_warning};
use quote::quote;
use syn::{AttributeArgs, Data, DeriveInput, Field, Fields, ItemStruct, Meta, NestedMeta, parse_macro_input, spanned::Spanned};
use crate::attribute::Attribute;

use crate::get_core_crate;

/// The ident for the props helper attribute (`#[prop_field(Children)]`)
const PROPS_HELPER_IDENT: &str = "prop_field";

const PROP_CHILDREN: &str = "Children";
const PROP_STYLE: &str = "Styles";
const PROP_ON_EVENT: &str = "OnEvent";
const PROP_FOCUSABLE: &str = "Focusable";

#[derive(Default)]
struct PropsHelpers {
    children_ident: Option<Ident>,
    styles_ident: Option<Ident>,
    on_event_ident: Option<Ident>,
    focusable_ident: Option<Ident>,
}

pub(crate) fn impl_widget_props(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, generics, ..
    } = parse_macro_input!(input);

    check_naming_convention(&ident);

    let helpers = process_data(data);

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
                        let ident_str = ident.to_string();
                        match ident_str.as_str() {
                            PROP_CHILDREN => props.children_ident = field.ident.clone(),
                            PROP_STYLE => props.styles_ident = field.ident.clone(),
                            PROP_ON_EVENT => props.on_event_ident = field.ident.clone(),
                            PROP_FOCUSABLE => props.focusable_ident = field.ident.clone(),
                            err => emit_error!(err.span(), "Invalid attribute: {}", err)
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