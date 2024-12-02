use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, token::Bracket, Expr, Ident, Lit, Token, Type};

enum IdentOrExpr {
    Ident(Ident),
    Expr(Expr),
}

struct FunctionsInput {
    handle: Type,
    functions: Type,
    field: IdentOrExpr,
}

struct ObjectInput {
    r#struct: Ident,
    name: Option<Lit>,
    functions: Option<FunctionsInput>,
    class: Option<Lit>,
    impls: Vec<Ident>,
}

impl Parse for IdentOrExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let result = if input.peek(Bracket) {
            let content;
            syn::bracketed!(content in input);

            Self::Expr(content.parse()?)
        } else {
            Self::Ident(input.parse()?)
        };

        Ok(result)
    }
}

impl Parse for FunctionsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let handle = input.parse()?;
        input.parse::<Token![,]>()?;

        let functions = input.parse()?;
        input.parse::<Token![,]>()?;

        let field = input.parse()?;

        input.parse::<Token![,]>().ok();

        Ok(Self {
            handle,
            functions,
            field,
        })
    }
}

impl Parse for ObjectInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let r#struct = input.parse()?;

        let mut result = ObjectInput {
            r#struct,
            name: None,
            functions: None,
            class: None,
            impls: vec![],
        };

        // Exit early if only a struct was given
        if input.is_empty() {
            return Ok(result);
        }

        if input.peek2(Lit) {
            input.parse::<Token![,]>()?;
            result.name = Some(input.parse()?)
        }

        // Exit early if only a struct and lit were given
        if input.is_empty() {
            return Ok(result);
        }

        while !input.is_empty() {
            input.parse::<Token![,]>()?;

            // Allow trailing comma
            if input.is_empty() {
                break;
            }

            input.parse::<Token![@]>()?;

            let name = input.parse::<Ident>()?;

            if name == "functions" {
                let content;
                syn::parenthesized!(content in input);

                let functions = content.parse::<FunctionsInput>()?;

                result.functions = Some(functions);
            } else if name == "class" {
                let content;
                syn::parenthesized!(content in input);

                let class = content.parse::<Lit>()?;

                result.class = Some(class);
            } else if name == "impls" {
                let content;
                syn::parenthesized!(content in input);

                let impls = content.parse_terminated(Ident::parse, Token![,])?;

                result.impls = impls.into_iter().collect();
            }
        }

        Ok(result)
    }
}

/// A procedural macro that defines a struct, its associated methods, and additional functionality
/// for integrating with the `rusty_uevr` and Unreal Engine API. This macro can generate the necessary
/// code to support features like pointer conversions, function bindings, class associations,
/// and custom trait implementations for Unreal Engine objects in Rust.
///
/// ## Usage
///
/// The `define_object!` macro allows you to define a new struct with various optional attributes:
///
/// - **Struct name**: The first argument is the name of the struct that will be defined.
/// - **Static name**: Optionally, you can specify a static name for the object using a string literal.
/// - **Functions**: You can specify function bindings using `@functions` which will set up the handle,
///   function list, and the associated SDk field for the object.
/// - **Class association**: Using `@class`, you can associate the struct with a specific Unreal class object.
/// - **Trait implementations**: You can implement traits for the object using `@impls`.
///
/// The macro generates the following for each object:
/// - A struct with a pointer to `std::ffi::c_void`.
/// - Implementation of the `rusty_uevr::api::Ptr` trait for converting the struct to/from a raw pointer.
/// - Optional methods such as `internal_name` and `to_handle`/`from_handle` for easier interaction with the engine.
/// - Static variable for function bindings and initialization via the `initialize` method.
/// - Static class association for UObject discovery.
/// - Optionally, user-defined trait implementations for the struct.
///
/// ### Example Usage
///
/// ```rust
/// define_object!(
///     UEnum,
/// );
///
/// define_object!(
///     UEnum,
///     "Enum",
/// );
///
/// define_object!(
///     UEnum,
///     "Enum",
///     @impls(RUObject)
/// );
///
/// define_object!(
///     FField,
///     @functions(UEVR_FFieldHandle, UEVR_FFieldFunctions, ffield)
/// );
///
/// define_object!(
///     FProperty,
///     @functions(UEVR_FPropertyHandle, UEVR_FPropertyFunctions, fproperty),
///     @impls(RFField)
/// );
///
/// define_object!(
///     UScriptStruct,
///     "ScriptStruct",
///     @functions(UEVR_UScriptStructHandle, UEVR_UScriptStructFunctions, uscriptstruct),
///     @class("Class /Script/CoreUObject.ScriptStruct"),
///     @impls(RUObject, RUField, RUStruct)
/// );
/// ```
#[proc_macro]
pub fn define_object(input: TokenStream) -> TokenStream {
    let ObjectInput {
        r#struct,
        name,
        functions,
        class,
        impls,
    } = parse_macro_input!(input);

    let mut fragments = vec![quote! {
        #[derive(Clone, Copy)]
        pub struct #r#struct(*mut std::ffi::c_void);

        #[automatically_derived]
        impl rusty_uevr::api::Ptr for #r#struct {
            fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
                Self(ptr)
            }

            fn to_ptr(&self) -> *mut std::ffi::c_void {
                self.0
            }
        }
    }];

    if let Some(name) = name {
        fragments.push(quote! {
            #[automatically_derived]
            impl #r#struct {
                pub const fn internal_name() -> &'static str {
                    #name
                }
            }
        });
    }

    if let Some(FunctionsInput {
        handle,
        functions,
        field,
    }) = functions
    {
        let global = Ident::new(&format!("__{}_fns", r#struct), Span::call_site());
        let field = match field {
            IdentOrExpr::Expr(expr) => quote! { #expr },
            IdentOrExpr::Ident(ident) => quote! { (*rusty_uevr::api::API::get().sdk()).#ident },
        };

        fragments.push(quote! {
            #[doc(hidden)]
            #[allow(non_upper_case_globals)]
            static mut #global: *const rusty_uevr::bindings::#functions = std::ptr::null();

            #[automatically_derived]
            impl #r#struct {
                pub fn to_handle(&self) -> rusty_uevr::bindings::#handle {
                    self.to_ptr() as rusty_uevr::bindings::#handle
                }

                pub fn from_handle(handle: rusty_uevr::bindings::#handle) -> Self {
                    Self(handle as *mut std::ffi::c_void)
                }

                pub fn from_handle_safe(handle: rusty_uevr::bindings::#handle) -> Option<Self> {
                    if handle.is_null() {
                        None
                    } else {
                        Some(Self(handle as *mut std::ffi::c_void))
                    }
                }

                fn initialize<'a>() -> &'a rusty_uevr::bindings::#functions {
                    unsafe {
                        if #global.is_null() {
                            #global = #field;
                        }

                        &*#global
                    }
                }
            }
        });
    }

    if let Some(class) = class {
        fragments.push(quote! {
            impl rusty_uevr::api::StaticClass for #r#struct {
                fn static_class_safe() -> Option<rusty_uevr::api::UClass> {
                    rusty_uevr::api::API::get().find_uobject(#class)
                }
            }
        });
    }

    if !impls.is_empty() {
        fragments.push(quote! {
            #(
                #[automatically_derived]
                impl #impls for #r#struct {}
            )*
        });
    }

    fragments
        .into_iter()
        .fold(quote! {}, |acc, fragment| {
            quote! {
                #acc
                #fragment
            }
        })
        .into()
}
