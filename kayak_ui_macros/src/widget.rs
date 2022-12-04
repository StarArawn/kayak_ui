use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::Path;

use crate::children::Children;
use crate::tags::ClosingTag;
use crate::widget_attributes::CustomWidgetAttributes;
use crate::widget_builder::build_widget_stream;
use crate::{tags::OpenTag, widget_attributes::WidgetAttributes};

#[derive(Clone, Debug)]
pub struct Widget {
    pub attributes: WidgetAttributes,
    pub children: Children,
    declaration: TokenStream,
    pub entity_id: TokenStream,
}

#[derive(Clone)]
pub struct ConstructedWidget {
    pub widget: Widget,
}

#[derive(Clone)]
pub struct ForcedWidget {
    pub widget: Widget,
}

#[derive(Clone)]
pub struct ForcedConstructedWidget {
    pub widget: Widget,
}

impl Parse for ConstructedWidget {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            widget: Widget::custom_parse(input, true, false, 0).unwrap(),
        })
    }
}

impl Parse for ForcedWidget {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            widget: Widget::custom_parse(input, false, true, 0).unwrap(),
        })
    }
}

impl Parse for ForcedConstructedWidget {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            widget: Widget::custom_parse(input, true, true, 0).unwrap(),
        })
    }
}

impl Parse for Widget {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::custom_parse(input, false, false, 0)
    }
}

impl Widget {
    pub fn is_custom_element(name: &syn::Path) -> bool {
        match name.get_ident() {
            None => true,
            Some(ident) => {
                let name = ident.to_string();
                let first_letter = name.get(0..1).unwrap();
                first_letter.to_uppercase() == first_letter
            }
        }
    }

    pub fn custom_parse(
        input: ParseStream,
        as_prop: bool,
        forced: bool,
        index: usize,
    ) -> Result<Widget> {
        //let o = input.parse::<OpenCodeBlock>()?;
        let open_tag = input.parse::<OpenTag>()?;

        let children = if open_tag.self_closing {
            Children::new(vec![])
        } else {
            let children = input.parse::<Children>()?;
            let closing_tag = input.parse::<ClosingTag>()?;
            closing_tag.validate(&open_tag);
            children
        };

        let (entity_id, declaration) = if let Some(name) = open_tag.name {
            if Self::is_custom_element(&name) {
                let attrs = &open_tag.attributes.for_custom_element(&children);
                let (entity_id, props, constructor) =
                    Self::construct(&name, attrs, as_prop, forced, false, index);
                if !as_prop {
                    let widget_block =
                        build_widget_stream(quote! { built_widget }, constructor, 0, false);
                    (
                        entity_id,
                        quote! {{
                            let parent_org = parent_id;
                            #props
                            #widget_block
                        }},
                    )
                } else {
                    (
                        entity_id.clone(),
                        quote! {{
                            #props
                            #constructor;
                            #entity_id
                        }},
                    )
                }
            } else {
                panic!("Couldn't find widget!");
            }
        } else {
            let attrs = &open_tag.attributes.for_custom_element(&children);
            let name = syn::parse_str::<syn::Path>("fragment").unwrap();
            let (entity_id, props, _) = Self::construct(&name, attrs, true, forced, true, index);
            (
                entity_id,
                quote! {
                    #props
                },
            )
        };

        Ok(Widget {
            attributes: open_tag.attributes,
            children,
            declaration,
            entity_id,
        })
    }

    /// Constructs a widget and its props
    ///
    /// The returned tuple contains:
    /// 1. The props constructor and assignment
    /// 2. The widget constructor
    ///
    /// # Arguments
    ///
    /// * `name`: The full-path name of the widget
    /// * `attrs`: The attributes (props) to apply to this widget
    ///
    /// returns: (TokenStream, TokenStream)
    fn construct(
        name: &Path,
        attrs: &CustomWidgetAttributes,
        as_prop: bool,
        forced: bool,
        only_children: bool,
        _index: usize,
    ) -> (TokenStream, TokenStream, TokenStream) {
        // let kayak_core = get_core_crate();

        let entity_name_id = attrs.attributes.iter().find_map(|attribute| {
            let key = attribute.ident();
            let value = attribute.value_tokens();
            let key_name = quote! { #key }.to_string();
            if key_name == "id" {
                Some(value)
            } else {
                None
            }
        });

        let prop_ident = format_ident!("internal_rsx_props");
        let entity_id = if let Some(entity_name_id) = entity_name_id {
            let entity_name_id = format_ident!("{}", entity_name_id.to_string().replace('"', ""));
            quote! { #entity_name_id }
        } else {
            quote! { widget_entity }
        };
        let assigned_attrs = attrs.assign_attributes(&prop_ident);

        // If this widget contains children, add it (should result in error if widget does not accept children)
        let children = if attrs.should_add_children() {
            // let kayak_core = get_core_crate();
            let children_tuple = attrs
                .children
                .as_option_of_tuples_tokens(only_children || attrs.children.is_block());

            // attrs.push(quote! {
            //     let children = children.clone();
            //     #kayak_core::WidgetProps::set_children(&mut #ident, #children_tuple);
            // });
            let start = if !only_children {
                quote! {
                    let parent_id_old = parent_id;
                    let parent_id = Some(#entity_id);
                    let mut children = KChildren::new();
                }
            } else {
                quote! {}
            };
            let middle = quote! {
                #children_tuple
            };
            let end = if !only_children {
                quote! {
                    // #prop_ident.children.despawn(&mut commands);
                    #prop_ident.children = children;
                    let parent_id = parent_id_old;
                }
            } else {
                quote! {}
            };
            quote! {
                #start
                #middle
                #end
            }
        } else {
            quote! {}
        };

        if only_children {
            return (entity_id, quote! { #children }, quote! {});
        }

        let props = if forced {
            quote! {
                let #entity_id = widget_context.force_spawn_widget(&mut commands, parent_org);
                let mut #prop_ident = #name {
                    #assigned_attrs
                    ..Default::default()
                };

                #children
            }
        } else {
            quote! {
                let #entity_id = widget_context.spawn_widget(&mut commands, parent_org);
                let mut #prop_ident = #name {
                    #assigned_attrs
                    ..Default::default()
                };

                #children
            }
        };

        let add_widget = if as_prop {
            quote! {}
        } else {
            quote! { widget_context.add_widget(parent_id, #entity_id); }
        };

        let constructor = quote! {
            commands.entity(#entity_id).insert(#prop_ident);
            #add_widget
        };

        (entity_id, props, constructor)
    }
}

impl ToTokens for Widget {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.declaration.to_tokens(tokens);
    }
}
