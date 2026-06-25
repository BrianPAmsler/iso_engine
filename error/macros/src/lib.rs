use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Nothing, parse_macro_input};

mod derive;
mod union;

#[proc_macro_derive(Error, attributes(error))]
pub fn derive_error(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive::derive_error(tokens)
}

#[proc_macro]
pub fn union(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    union::union(tokens)
}

#[proc_macro]
pub fn create_error(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(tokens as Nothing);

    let conversion = if std::env::var("CARGO_PKG_NAME") != Ok("opengl_engine".into()) {
        quote! {
            impl<E: ::std::error::Error> From<::opengl_engine::error::Error<E>> for Error<E> {
                fn from(value: ::opengl_engine::error::Error<E>) -> Error<E> {
                    Error::from_raw(value.source, value.backtrace)
                }
            }
            impl<E: ::std::error::Error> From<Error<E>> for ::opengl_engine::error::Error<E> {
                fn from(value: Error<E>) -> ::opengl_engine::error::Error<E> {
                    ::opengl_engine::error::Error::from_raw(value.source, value.backtrace)
                }
            }
        }
    } else {
        TokenStream::new()
    };
    quote! {
        #[derive(Debug)]
        pub struct DynamicMessageErorr(pub String);

        impl ::std::fmt::Display for DynamicMessageErorr {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl ::std::error::Error for DynamicMessageErorr {}

        #[derive(Debug)]
        pub struct MessageErorr(pub &'static str);

        impl ::std::fmt::Display for MessageErorr {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl ::std::error::Error for MessageErorr {}

        #[derive(Debug)]
        pub struct Error<E: ::std::error::Error> {
            source: E,
            backtrace: ::std::cell::RefCell<::opengl_engine::error::backtrace::Backtrace>
        }

        impl<E: ::std::error::Error> Error<E> {
            pub fn new(error: E) -> Error<E> {
                Error { source: error, backtrace: ::std::cell::RefCell::new(::opengl_engine::error::backtrace::Backtrace::new_unresolved()) }
            }

            pub fn from_existing<E2: ::std::error::Error>(error: Error<E2>) -> Error<E>
            where
                E: From<E2>
            {
                let Error { source, backtrace } = error;
                let source = source.into();

                Error { source, backtrace }
            }

            pub fn from_raw(source: E, backtrace: ::opengl_engine::error::backtrace::Backtrace) -> Error<E>
            {
                let backtrace = ::std::cell::RefCell::new(backtrace);
                Error { source, backtrace }
            }

            pub fn source(&self) -> &E {
                &self.source
            }

            pub fn backtrace(&self) -> ::opengl_engine::error::backtrace::Backtrace {
                self.backtrace.borrow().clone()
            }
        }

        impl<E: ::std::error::Error> ::std::fmt::Display for Error<E> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.backtrace.borrow_mut().resolve();
                write!(f, "Error: {}\n\nStack Backtrace\n{:?}", self.source, self.backtrace.borrow())
            }
        }

        impl<E: ::std::error::Error> From<E> for Error<E> {
            fn from(value: E) -> Error<E> {
                Error::new(value)
            }
        }

        pub type Result<T, E> = std::result::Result<T, Error<E>>;

        pub mod any {
            #[derive(Debug)]
            pub struct Error {
                source: Box<dyn ::std::error::Error>,
                backtrace: ::std::cell::RefCell<::opengl_engine::error::backtrace::Backtrace>,
            }

            impl Error {
                pub fn new<E: ::std::error::Error + 'static>(error: E) -> Error {
                    Error { source: Box::new(error), backtrace: ::std::cell::RefCell::new(::opengl_engine::error::backtrace::Backtrace::new_unresolved()) }
                }

                pub fn source(&self) -> &dyn ::std::error::Error {
                    & *self.source
                }

                pub fn backtrace(&self) -> ::opengl_engine::error::backtrace::Backtrace {
                    self.backtrace.borrow().clone()
                }
            }

            impl ::std::fmt::Display for Error {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.backtrace.borrow_mut().resolve();
                    write!(f, "Error: {}\n\nStack Backtrace\n{:?}", self.source, self.backtrace.borrow())
                }
            }

            pub type Result<T> = std::result::Result<T, Error>;

            impl<E: ::std::error::Error + 'static> From<super::Error<E>> for Error {
                fn from(value: super::Error<E>) -> Self {
                    Error { source: Box::new(value.source), backtrace: value.backtrace }
                }
            }

            impl<E: ::std::error::Error + 'static> From<E> for Error {
                fn from(value: E) -> Self {
                    Error::new(value)
                }
            }
        }
    }.into()
}