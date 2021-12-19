extern crate proc_macro;

mod function_component;
mod tags;

mod arc_function;
mod attribute;
mod child;
mod children;
mod partial_eq;
mod widget;
mod widget_attributes;

use function_component::WidgetArguments;
use partial_eq::impl_dyn_partial_eq;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, parse_quote};
use widget::ConstructedWidget;

use crate::widget::Widget;

#[proc_macro]
#[proc_macro_error]
pub fn render(input: TokenStream) -> TokenStream {
    let widget = parse_macro_input!(input as Widget);

    #[cfg(feature = "internal")]
    let kayak_core = quote! { kayak_core };
    #[cfg(not(feature = "internal"))]
    let kayak_core = quote! { kayak_ui::core };

    let result = quote! {
        let parent_id: Option<Index> = None;
        let children: Option<#kayak_core::Children> = None;
        let tree = #kayak_core::WidgetTree::new();
        #widget
    };

    TokenStream::from(result)
}

/// Generate a renderable widget tree, before rendering it
#[proc_macro]
#[proc_macro_error]
pub fn rsx(input: TokenStream) -> TokenStream {
    let widget = parse_macro_input!(input as Widget);
    let result = quote! { #widget };
    TokenStream::from(result)
}

#[proc_macro]
#[proc_macro_error]
pub fn constructor(input: TokenStream) -> TokenStream {
    let el = parse_macro_input!(input as ConstructedWidget);
    let widget = el.widget;
    let result = quote! { #widget };
    TokenStream::from(result)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn widget(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut widget_args = WidgetArguments::default();
    if !args.is_empty() {
        // Parse stuff..
        let parsed = args.to_string();
        widget_args.focusable = parsed.contains("focusable");
    }

    let f = parse_macro_input!(item as syn::ItemFn);
    function_component::create_function_widget(f, widget_args)
}

#[proc_macro_derive(DynPartialEq)]
pub fn dyn_partial_eq_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_dyn_partial_eq(&ast)
}

#[proc_macro_attribute]
pub fn dyn_partial_eq(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as syn::ItemTrait);

    let name = &input.ident;

    let bound: syn::TypeParamBound = parse_quote! {
      DynPartialEq
    };

    input.supertraits.push(bound);

    (quote! {
      #input

      impl core::cmp::PartialEq for Box<dyn #name> {
        fn eq(&self, other: &Self) -> bool {
          self.box_eq(other.as_any())
        }
      }
    })
    .into()
}

/// Create a state and its setter
///
/// # Arguments
///
/// * `initial_state`: The expression
///
/// returns: (state, set_state)
///
/// # Examples
///
/// ```
/// # use kayak_core::{EventType, OnEvent};
/// use kayak_render_macros::use_state;
///
/// let (count, set_count) = use_state!(0);
///
/// let on_event = OnEvent::new(move |_, event| match event.event_type {
///         EventType::Click => {
///             set_count(foo + 1);
///         }
///         _ => {}
/// });
///
/// rsx! {
///         <>
///             <Button on_event={Some(on_event)}>
///                 <Text size={16.0} content={format!("Count: {}", count)}>{}</Text>
///             </Button>
///         </>
///     }
/// ```
#[proc_macro]
pub fn use_state(initial_state: TokenStream) -> TokenStream {
    let initial_state = parse_macro_input!(initial_state as syn::Expr);
    let result = quote! {{
        let state = context.create_state(#initial_state).unwrap();
        let cloned_state = state.clone();
        let set_state = move |value| {
            cloned_state.set(value);
        };

        let state_value = state.get();

        (state.get(), set_state)
    }};
    TokenStream::from(result)
}
