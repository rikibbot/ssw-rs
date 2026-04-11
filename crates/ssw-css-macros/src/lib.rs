//! Proc-macro implementation for `ssw_css::css!`.

use std::collections::BTreeSet;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenTree};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Ident, LitStr, Result, Token, braced, parse_macro_input};

#[proc_macro]
/// Builds a scoped stylesheet for `ssw-rs`.
///
/// The current supported surface intentionally stays small:
/// - string-literal selectors
/// - string-literal declaration values
/// - local class selector rewriting
/// - `@media "..." { ... }` blocks
pub fn css(input: TokenStream) -> TokenStream {
    let source = proc_macro2::TokenStream::from(input.clone()).to_string();
    let stylesheet = parse_macro_input!(input as StyleSheetInput);
    let scope = hash_scope(&source);

    match stylesheet.render(&scope) {
        Ok((css, slots)) => {
            let scope = LitStr::new(&scope, Span::call_site());
            let css = LitStr::new(&css, Span::call_site());
            let slots = slots
                .into_iter()
                .map(|slot| LitStr::new(&slot, Span::call_site()));

            TokenStream::from(quote! {
                ::ssw_css::StyleSheet::new(#scope, #css, &[#(#slots),*])
            })
        }
        Err(error) => TokenStream::from(error.to_compile_error()),
    }
}

struct StyleSheetInput {
    items: Vec<Item>,
}

impl Parse for StyleSheetInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Self { items })
    }
}

impl StyleSheetInput {
    fn render(&self, scope: &str) -> Result<(String, Vec<String>)> {
        let mut css = String::new();
        let mut slots = BTreeSet::new();

        for item in &self.items {
            item.render(&mut css, scope, &mut slots)?;
        }

        Ok((css, slots.into_iter().collect()))
    }
}

enum Item {
    Rule(Rule),
    Media(MediaRule),
}

impl Parse for Item {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(Token![@]) {
            Ok(Self::Media(input.parse()?))
        } else {
            Ok(Self::Rule(input.parse()?))
        }
    }
}

impl Item {
    fn render(&self, output: &mut String, scope: &str, slots: &mut BTreeSet<String>) -> Result<()> {
        match self {
            Self::Rule(rule) => rule.render(output, scope, slots),
            Self::Media(media) => media.render(output, scope, slots),
        }
    }
}

struct Rule {
    selector: LitStr,
    declarations: Vec<Declaration>,
}

impl Parse for Rule {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let selector: LitStr = input.parse()?;
        let content;
        braced!(content in input);

        let mut declarations = Vec::new();
        while !content.is_empty() {
            declarations.push(content.parse()?);
        }

        Ok(Self {
            selector,
            declarations,
        })
    }
}

impl Rule {
    fn render(&self, output: &mut String, scope: &str, slots: &mut BTreeSet<String>) -> Result<()> {
        let selector = rewrite_selector(&self.selector, scope, slots)?;
        output.push_str(&selector);
        output.push('{');

        for declaration in &self.declarations {
            declaration.render(output);
        }

        output.push('}');
        Ok(())
    }
}

struct MediaRule {
    query: LitStr,
    rules: Vec<Rule>,
}

impl Parse for MediaRule {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        input.parse::<Token![@]>()?;
        let name: Ident = input.parse()?;

        if name != "media" {
            return Err(Error::new(
                name.span(),
                "only `@media` is supported in `css!`",
            ));
        }

        let query: LitStr = input.parse()?;
        let content;
        braced!(content in input);

        let mut rules = Vec::new();
        while !content.is_empty() {
            rules.push(content.parse()?);
        }

        Ok(Self { query, rules })
    }
}

impl MediaRule {
    fn render(&self, output: &mut String, scope: &str, slots: &mut BTreeSet<String>) -> Result<()> {
        output.push_str("@media ");
        output.push_str(&self.query.value());
        output.push('{');

        for rule in &self.rules {
            rule.render(output, scope, slots)?;
        }

        output.push('}');
        Ok(())
    }
}

struct Declaration {
    name: String,
    value: LitStr,
}

impl Parse for Declaration {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name = parse_property_name(input)?;
        input.parse::<Token![:]>()?;
        let value: LitStr = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(Self { name, value })
    }
}

impl Declaration {
    fn render(&self, output: &mut String) {
        output.push_str(&self.name);
        output.push(':');
        output.push_str(&self.value.value());
        output.push(';');
    }
}

fn parse_property_name(input: ParseStream<'_>) -> Result<String> {
    let mut name = String::new();

    while !input.peek(Token![:]) {
        if input.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "expected `:` after CSS property name",
            ));
        }

        let tree: TokenTree = input.parse()?;
        match tree {
            TokenTree::Ident(ident) => name.push_str(&ident.to_string()),
            TokenTree::Punct(punct) if punct.as_char() == '-' => name.push('-'),
            other => {
                return Err(Error::new(
                    other.span(),
                    "unsupported token in CSS property name",
                ));
            }
        }
    }

    if name.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "expected a CSS property name",
        ));
    }

    Ok(name)
}

fn rewrite_selector(
    selector: &LitStr,
    scope: &str,
    slots: &mut BTreeSet<String>,
) -> Result<String> {
    let selector_value = selector.value();
    let mut output = String::new();
    let mut chars = selector_value.chars().peekable();
    let mut found_local_class = false;

    while let Some(ch) = chars.next() {
        if ch == '.' {
            let Some(next) = chars.peek().copied() else {
                output.push('.');
                continue;
            };

            if !is_ident_start(next) {
                output.push('.');
                continue;
            }

            let mut slot = String::new();
            while let Some(next) = chars.peek().copied() {
                if is_ident_continue(next) {
                    slot.push(next);
                    chars.next();
                } else {
                    break;
                }
            }

            found_local_class = true;
            slots.insert(slot.clone());
            output.push('.');
            output.push_str("sswc-");
            output.push_str(scope);
            output.push('-');
            output.push_str(&slot);
            continue;
        }

        output.push(ch);
    }

    if !found_local_class {
        return Err(Error::new(
            selector.span(),
            "scoped CSS selectors must include at least one local class selector like `.root`",
        ));
    }

    Ok(output)
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch == '-' || ch.is_ascii_alphanumeric()
}

fn hash_scope(source: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;

    for byte in source.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("{:08x}", (hash >> 32) as u32)
}
