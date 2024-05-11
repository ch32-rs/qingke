use std::iter;

use proc_macro2::Span;
use proc_macro_error::proc_macro_error;
use syn::{
    parse, parse_macro_input, spanned::Spanned, Ident, ItemFn, PathArguments, ReturnType, Type,
    Visibility,
};

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    // check the function arguments
    if !f.sig.inputs.is_empty() {
        return parse::Error::new(
            f.sig.inputs.last().unwrap().span(),
            "`#[entry]` function accepts no arguments",
        )
        .to_compile_error()
        .into();
    }

    // check the function signature
    let valid_signature = f.sig.constness.is_none()
        && f.sig.asyncness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none()
        && match f.sig.output {
            ReturnType::Default => false,
            ReturnType::Type(_, ref ty) => matches!(**ty, Type::Never(_)),
        };

    if !valid_signature {
        return parse::Error::new(
            f.span(),
            "`#[entry]` function must have signature `[unsafe] fn() -> !`",
        )
        .to_compile_error()
        .into();
    }

    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "This attribute accepts no arguments")
            .to_compile_error()
            .into();
    }

    // XXX should we blacklist other attributes?
    let attrs = f.attrs;
    let unsafety = f.sig.unsafety;
    let args = f.sig.inputs;
    let stmts = f.block.stmts;

    quote!(
        #[allow(non_snake_case)]
        #[export_name = "main"]
        #(#attrs)*
        pub #unsafety fn __risc_v_rt__main(#args) -> ! {
            #(#stmts)*
        }
    )
    .into()
}

#[allow(unused)]
fn is_simple_type(ty: &Type, name: &str) -> bool {
    if let Type::Path(p) = ty {
        if p.qself.is_none() && p.path.leading_colon.is_none() && p.path.segments.len() == 1 {
            let segment = p.path.segments.first().unwrap();
            if segment.ident == name && segment.arguments == PathArguments::None {
                return true;
            }
        }
    }
    false
}

/// This attribute allows placing functions into ram.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn highcode(_args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    let section = quote! {
        #[link_section = ".highcode"]
        #[inline(never)] // make certain function is not inlined
    };

    quote!(
        #section
        #f
    )
    .into()
}

/// Marks a function as an interrupt handler. (Wrapping as a mret function)
///
/// Note that Rust has also introduced the `riscv-interrupt-m` and `riscv-interrupt-s` ABI, which
/// are used for machine and supervisor mode interrupts, respectively. These ABIs can also be used for
/// Qingke cores, yet they add additional register saving and restoring that is not necessary.
///
/// Usage:
/// ```ignore
/// #[interrupt]
/// fn UART0() { ... }
///
/// #[interrupt(core)]
/// fn SysTick() { ... }
/// ```
#[proc_macro_attribute]
pub fn interrupt(args: TokenStream, input: TokenStream) -> TokenStream {
    use syn::{AttributeArgs, Meta, NestedMeta};

    let mut f = parse_macro_input!(input as ItemFn);

    let is_core_irq = if args.is_empty() {
        false
    } else {
        let args: AttributeArgs = parse_macro_input!(args as AttributeArgs);
        if args.len() != 1 {
            return parse::Error::new(
                Span::call_site(),
                "This attribute accepts no arguments or a single 'core' argument",
            )
            .to_compile_error()
            .into();
        }

        if let NestedMeta::Meta(Meta::Path(ref p)) = args[0] {
            if let Some(ident) = p.get_ident() {
                if ident != "core" {
                    return parse::Error::new(
                        ident.span(),
                        "Only core interrupts are allowed without arguments",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        true
    };

    // check the function arguments
    if !f.sig.inputs.is_empty() {
        return parse::Error::new(
            f.sig.inputs.last().unwrap().span(),
            "`#[interrupt]` function accepts no arguments",
        )
        .to_compile_error()
        .into();
    }

    let ident = f.sig.ident.clone();
    let ident_s = ident.to_string();

    let valid_signature = f.sig.constness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none()
        && match f.sig.output {
            ReturnType::Default => true,
            ReturnType::Type(_, ref ty) => match **ty {
                Type::Tuple(ref tuple) => tuple.elems.is_empty(),
                Type::Never(..) => true,
                _ => false,
            },
        }
        && f.sig.inputs.len() <= 1;

    if !valid_signature {
        return parse::Error::new(
            f.span(),
            "`#[interrupt]` handlers must have signature `[unsafe] fn() [-> !]`",
        )
        .to_compile_error()
        .into();
    }

    let ident = f.sig.ident.clone();

    // This will be overwritten by `export_name` in linking process, i.e. name of the interrupt
    let wrapper_ident = Ident::new(&format!("{}_naked_wrapper", f.sig.ident), Span::call_site());

    f.sig.ident = Ident::new(&format!("__qingke_rt_{}", f.sig.ident), Span::call_site());

    let wrapped_ident = &f.sig.ident;

    let stmts = f.block.stmts.clone();
    // check irq names
    if is_core_irq {
        f.block.stmts = iter::once(
            syn::parse2(quote! {{
                // Check that this interrupt actually exists
                ::qingke_rt::CoreInterrupt::#ident;
            }})
            .unwrap(),
        )
        .chain(stmts)
        .collect();
    } else {
        f.block.stmts = iter::once(
            syn::parse2(quote! {{
                // Check that this interrupt actually exists
                crate::pac::interrupt::#ident;
            }})
            .unwrap(),
        )
        .chain(stmts)
        .collect();
    }

    quote!(
        #[doc(hidden)]
        #[export_name = #ident_s]
        #[naked]
        unsafe extern "C" fn #wrapper_ident() {
            asm!("
                addi sp, sp, -4
                sw ra, 0(sp)
                jal {irq_impl}
                lw ra, 0(sp)
                addi sp, sp, 4
                mret",
                options(noreturn),
                irq_impl = sym #wrapped_ident
            );
        }

        #f
    )
    .into()
}
