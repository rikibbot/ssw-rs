use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{
    Error, Expr, Ident, LitStr, Pat, Result, Token, braced, parenthesized, parse_macro_input,
};

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let template = parse_macro_input!(input as Template);
    TokenStream::from(quote! {{
        let mut __markup = ::ssw_html::Markup::new();
        #template
        __markup
    }})
}

struct Template {
    nodes: Vec<Node>,
}

impl Parse for Template {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            nodes: parse_nodes(input)?,
        })
    }
}

impl quote::ToTokens for Template {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let nodes = self.nodes.iter();
        tokens.extend(quote! {
            #(#nodes)*
        });
    }
}

enum Node {
    Element(Element),
    Text(LitStr),
    Expr(Expr),
    If {
        condition: Expr,
        then_branch: Vec<Node>,
        else_branch: Option<Vec<Node>>,
    },
    For {
        pattern: Pat,
        expr: Expr,
        body: Vec<Node>,
    },
}

impl quote::ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Element(element) => element.to_tokens(tokens),
            Self::Text(text) => tokens.extend(quote! {
                __markup.push_text(#text);
            }),
            Self::Expr(expr) => tokens.extend(quote! {
                ::ssw_html::__private::render_value(&mut __markup, &(#expr));
            }),
            Self::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let then_branch = then_branch.iter();
                match else_branch {
                    Some(else_branch) => {
                        let else_branch = else_branch.iter();
                        tokens.extend(quote! {
                            if #condition {
                                #(#then_branch)*
                            } else {
                                #(#else_branch)*
                            }
                        });
                    }
                    None => {
                        tokens.extend(quote! {
                            if #condition {
                                #(#then_branch)*
                            }
                        });
                    }
                }
            }
            Self::For {
                pattern,
                expr,
                body,
            } => {
                let body = body.iter();
                tokens.extend(quote! {
                    for #pattern in #expr {
                        #(#body)*
                    }
                });
            }
        }
    }
}

struct Element {
    name: LitStr,
    attrs: Vec<Attribute>,
    children: Vec<Node>,
}

impl quote::ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let attrs = self.attrs.iter();
        let children = self.children.iter();

        tokens.extend(quote! {
            ::ssw_html::__private::begin_element(&mut __markup, #name);
            #(#attrs)*
            ::ssw_html::__private::finish_open_tag(&mut __markup);
            #(#children)*
            ::ssw_html::__private::end_element(&mut __markup, #name);
        });
    }
}

enum AttributeValue {
    String(LitStr),
    Expr(Expr),
}

struct Attribute {
    name: LitStr,
    value: Option<AttributeValue>,
}

impl quote::ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        match &self.value {
            Some(AttributeValue::String(value)) => tokens.extend(quote! {
                ::ssw_html::__private::push_attribute_literal(&mut __markup, #name, #value);
            }),
            Some(AttributeValue::Expr(expr)) => tokens.extend(quote! {
                ::ssw_html::__private::push_attribute_expr(&mut __markup, #name, &(#expr));
            }),
            None => tokens.extend(quote! {
                ::ssw_html::__private::push_boolean_attribute(&mut __markup, #name);
            }),
        }
    }
}

fn parse_nodes(input: ParseStream<'_>) -> Result<Vec<Node>> {
    let mut nodes = Vec::new();
    while !input.is_empty() {
        nodes.push(parse_node(input)?);
    }
    Ok(nodes)
}

fn parse_node(input: ParseStream<'_>) -> Result<Node> {
    if input.peek(Token![@]) {
        parse_control(input)
    } else if input.peek(LitStr) {
        Ok(Node::Text(input.parse()?))
    } else if input.peek(syn::token::Paren) {
        let content;
        parenthesized!(content in input);
        Ok(Node::Expr(content.parse()?))
    } else {
        Ok(Node::Element(parse_element(input)?))
    }
}

fn parse_control(input: ParseStream<'_>) -> Result<Node> {
    input.parse::<Token![@]>()?;
    let keyword = input.call(Ident::parse_any)?;

    match keyword.to_string().as_str() {
        "if" => parse_if(input),
        "for" => parse_for(input),
        other => Err(Error::new(
            keyword.span(),
            format!("unsupported control directive '@{other}'"),
        )),
    }
}

fn parse_if(input: ParseStream<'_>) -> Result<Node> {
    let condition = parse_expr_until_block(input)?;
    let then_branch = parse_block(input)?;
    let else_branch = if input.peek(Token![else]) {
        input.parse::<Token![else]>()?;
        if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let nested = input.call(Ident::parse_any)?;
            if nested != "if" {
                return Err(Error::new(nested.span(), "expected '@if' after else"));
            }

            Some(vec![parse_if(input)?])
        } else {
            Some(parse_block(input)?)
        }
    } else {
        None
    };

    Ok(Node::If {
        condition,
        then_branch,
        else_branch,
    })
}

fn parse_for(input: ParseStream<'_>) -> Result<Node> {
    let pattern = Pat::parse_single(input)?;
    input.parse::<Token![in]>()?;
    let expr = parse_expr_until_block(input)?;
    let body = parse_block(input)?;

    Ok(Node::For {
        pattern,
        expr,
        body,
    })
}

fn parse_block(input: ParseStream<'_>) -> Result<Vec<Node>> {
    let content;
    braced!(content in input);
    parse_nodes(&content)
}

fn parse_expr_until_block(input: ParseStream<'_>) -> Result<Expr> {
    let mut tokens = TokenStream2::new();

    while !input.is_empty() && !input.peek(syn::token::Brace) {
        let token: TokenTree = input.parse()?;
        tokens.extend(std::iter::once(token));
    }

    if tokens.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "expected expression before HTML block",
        ));
    }

    syn::parse2(tokens)
}

fn parse_element(input: ParseStream<'_>) -> Result<Element> {
    let name_ident = input.call(Ident::parse_any)?;
    let mut attrs = Vec::new();

    while !input.peek(syn::token::Brace) {
        attrs.push(parse_attribute(input)?);
    }

    let children = parse_block(input)?;

    Ok(Element {
        name: LitStr::new(&normalize_name(&name_ident), name_ident.span()),
        attrs,
        children,
    })
}

fn parse_attribute(input: ParseStream<'_>) -> Result<Attribute> {
    let name_ident = input.call(Ident::parse_any)?;
    let name = LitStr::new(&normalize_name(&name_ident), name_ident.span());

    let value = if input.peek(Token![=]) {
        input.parse::<Token![=]>()?;

        if input.peek(LitStr) {
            Some(AttributeValue::String(input.parse()?))
        } else if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            Some(AttributeValue::Expr(content.parse()?))
        } else {
            return Err(Error::new(
                Span::call_site(),
                "expected string literal or parenthesized expression after '='",
            ));
        }
    } else {
        None
    };

    Ok(Attribute { name, value })
}

fn normalize_name(ident: &Ident) -> String {
    ident.to_string().replace('_', "-")
}
