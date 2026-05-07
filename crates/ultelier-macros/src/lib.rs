use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, parse_quote, Stmt, Token};

mod kw {
    syn::custom_keyword!(inline);
    syn::custom_keyword!(offset);
    syn::custom_keyword!(pointer_offset);
    syn::custom_keyword!(replace);
}

#[derive(Default)]
struct HookAttrs {
    replace: Option<syn::Path>,
    pointer_offset: Option<syn::Expr>,
    offset: Option<syn::Expr>,
    inline: bool,
}

fn merge(left: HookAttrs, right: HookAttrs) -> HookAttrs {
    HookAttrs {
        replace: left.replace.or(right.replace),
        pointer_offset: left.pointer_offset.or(right.pointer_offset),
        offset: left.offset.or(right.offset),
        inline: left.inline || right.inline,
    }
}

impl Parse for HookAttrs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let attr = if lookahead.peek(kw::offset) {
            let _: kw::offset = input.parse()?;
            input.parse::<Token![=]>()?;
            HookAttrs {
                offset: Some(input.parse()?),
                ..HookAttrs::default()
            }
        } else if lookahead.peek(kw::pointer_offset) {
            let _: kw::pointer_offset = input.parse()?;
            input.parse::<Token![=]>()?;
            HookAttrs {
                pointer_offset: Some(input.parse()?),
                ..HookAttrs::default()
            }
        } else if lookahead.peek(kw::replace) {
            let _: kw::replace = input.parse()?;
            input.parse::<Token![=]>()?;
            HookAttrs {
                replace: Some(input.parse()?),
                ..HookAttrs::default()
            }
        } else if lookahead.peek(kw::inline) {
            let _: kw::inline = input.parse()?;
            HookAttrs {
                inline: true,
                ..HookAttrs::default()
            }
        } else {
            return Err(lookahead.error());
        };

        Ok(if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            if input.is_empty() {
                attr
            } else {
                merge(attr, input.parse()?)
            }
        } else {
            attr
        })
    }
}

fn extern_c_abi() -> syn::Abi {
    syn::Abi {
        extern_token: syn::token::Extern {
            span: Span::call_site(),
        },
        name: Some(syn::LitStr::new("C", Span::call_site())),
    }
}

fn remove_mut(arg: &syn::FnArg) -> syn::FnArg {
    let mut arg = arg.clone();

    if let syn::FnArg::Typed(ref mut arg) = arg {
        if let syn::Pat::Ident(ref mut arg) = *arg.pat {
            arg.by_ref = None;
            arg.mutability = None;
            arg.subpat = None;
        }
    }

    arg
}

fn generate_install_fn(name: &syn::Ident, orig: &syn::Ident, attrs: &HookAttrs) -> TokenStream2 {
    let install_fn = quote::format_ident!("{}_skyline_internal_install_hook", name);
    let pointer_offset = attrs
        .pointer_offset
        .as_ref()
        .map(ToTokens::into_token_stream)
        .unwrap_or(quote! { 0 });

    let replace = attrs
        .replace
        .as_ref()
        .map(ToTokens::into_token_stream)
        .or_else(|| {
            attrs.offset.as_ref().map(|offset| {
                quote! {
                    unsafe {
                        ::skyline::hooks::getRegionAddress(
                            ::skyline::hooks::Region::Text
                        ) as *mut u8
                    }.add(#offset)
                }
            })
        })
        .unwrap_or_else(|| {
            quote_spanned!(Span::call_site() =>
                compile_error!("Missing 'replace' or 'offset' item in local_hook macro");
            )
        });

    if attrs.inline {
        quote! {
            #[allow(non_snake_case)]
            pub fn #install_fn() {
                if (::skyline::hooks::A64InlineHook as *const ()).is_null() {
                    panic!("A64InlineHook is null");
                }

                unsafe {
                    ::skyline::hooks::A64InlineHook(
                        ((#replace as *const u8).offset(#pointer_offset) as *const ::skyline::libc::c_void),
                        #name as *const ::skyline::libc::c_void,
                    )
                }
            }
        }
    } else {
        quote! {
            #[allow(non_snake_case)]
            pub fn #install_fn() {
                if (::skyline::hooks::A64HookFunction as *const ()).is_null() {
                    panic!("A64HookFunction is null");
                }

                unsafe {
                    let mut temp = 0 as u64;
                    #[allow(static_mut_refs)]
                    ::skyline::hooks::A64HookFunction(
                        ((#replace as *const u8).offset(#pointer_offset) as *const ::skyline::libc::c_void),
                        #name as *const ::skyline::libc::c_void,
                        &mut temp as *mut u64 as *mut *mut ::skyline::libc::c_void
                    );

                    let _ = #orig.set(temp);
                }
            }
        }
    }
}

/// Skyline hook macro variant for reusable crates.
///
/// This preserves Skyline's generated installer and original-call support, but
/// intentionally does not add `#[no_mangle]` to the hook body. The hook is
/// installed with a direct function pointer, so exporting the hook symbol is not
/// needed and causes conflicts when a crate is linked into multiple plugins.
#[proc_macro_attribute]
pub fn local_hook(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut hook_fn = parse_macro_input!(input as syn::ItemFn);
    let attrs = parse_macro_input!(attrs as HookAttrs);
    let mut output = TokenStream2::new();

    hook_fn.sig.abi = Some(extern_c_abi());

    let args_tokens = hook_fn.sig.inputs.iter().map(remove_mut);
    let return_tokens = hook_fn.sig.output.to_token_stream();
    let original_fn = quote::format_ident!(
        "{}_skyline_internal_original_fn",
        hook_fn.sig.ident
    );

    if !attrs.inline {
        let original_macro: Stmt = parse_quote! {
            #[allow(unused_macros)]
            macro_rules! original {
                () => {
                    {
                        #[allow(unused_unsafe)]
                        if true {
                            let temp = #original_fn.get().unwrap();

                            unsafe {
                                core::mem::transmute::<*const (), extern "C" fn(#(#args_tokens),*) #return_tokens>(
                                    *temp as *const u64 as *const()
                                )
                            }
                        } else {
                            unreachable!()
                        }
                    }
                }
            }
        };
        hook_fn.block.stmts.insert(0, original_macro);

        let call_original_macro: Stmt = parse_quote! {
            #[allow(unused_macros)]
            macro_rules! call_original {
                ($($args:expr),* $(,)?) => {
                    original!()($($args),*)
                }
            }
        };
        hook_fn.block.stmts.insert(1, call_original_macro);
    }

    hook_fn.to_tokens(&mut output);

    let install_fn = generate_install_fn(&hook_fn.sig.ident, &original_fn, &attrs);
    if attrs.inline {
        install_fn.to_tokens(&mut output);
    } else {
        quote! {
            #install_fn

            #[allow(non_upper_case_globals)]
            pub static #original_fn: ::std::sync::OnceLock<u64> = ::std::sync::OnceLock::new();
        }
        .to_tokens(&mut output);
    }

    output.into()
}
