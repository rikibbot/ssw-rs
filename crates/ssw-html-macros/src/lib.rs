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
    id: Option<LitStr>,
    class: Option<LitStr>,
    attrs: Vec<Attribute>,
    body: ElementBody,
}

impl quote::ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let id = self.id.as_ref().map(|value| {
            quote! {
                ::ssw_html::__private::push_attribute_literal(&mut __markup, "id", #value);
            }
        });
        let class = self.class.as_ref().map(|value| {
            quote! {
                ::ssw_html::__private::push_attribute_literal(&mut __markup, "class", #value);
            }
        });
        let attrs = self.attrs.iter();

        let render_open = quote! {
            ::ssw_html::__private::begin_element(&mut __markup, #name);
            #id
            #class
            #(#attrs)*
            ::ssw_html::__private::finish_open_tag(&mut __markup);
        };

        match &self.body {
            ElementBody::Block(children) => {
                let children = children.iter();
                tokens.extend(quote! {
                    #render_open
                    #(#children)*
                    ::ssw_html::__private::end_element(&mut __markup, #name);
                });
            }
            ElementBody::Empty => {
                tokens.extend(quote! {
                    #render_open
                    if !::ssw_html::__private::is_void_element(#name) {
                        ::ssw_html::__private::end_element(&mut __markup, #name);
                    }
                });
            }
        }
    }
}

enum ElementBody {
    Block(Vec<Node>),
    Empty,
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
    let mut id = None;
    let mut classes = Vec::new();
    let mut attrs = Vec::new();

    while !input.peek(syn::token::Brace) && !input.peek(Token![;]) {
        if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let value = parse_name_literal(input)?;
            if id.replace(value).is_some() {
                return Err(Error::new(
                    name_ident.span(),
                    "only one id may be assigned to an element",
                ));
            }
            continue;
        }

        if input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            classes.push(parse_name_literal(input)?);
            continue;
        }

        match parse_attribute(input)? {
            ParsedAttribute { name, value, span } if name == "id" => {
                if id.is_some() {
                    return Err(Error::new(
                        span,
                        "cannot mix '#id' shorthand with an explicit id attribute",
                    ));
                }
                let value = value.ok_or_else(|| {
                    Error::new(span, "the 'id' attribute requires an explicit value")
                })?;
                attrs.push(Attribute {
                    name: LitStr::new("id", span),
                    value: Some(value),
                });
            }
            ParsedAttribute { name, value, span } if name == "class" => {
                if !classes.is_empty() {
                    return Err(Error::new(
                        span,
                        "cannot mix '.class' shorthand with an explicit class attribute yet",
                    ));
                }
                let value = value.ok_or_else(|| {
                    Error::new(span, "the 'class' attribute requires an explicit value")
                })?;
                attrs.push(Attribute {
                    name: LitStr::new("class", span),
                    value: Some(value),
                });
            }
            ParsedAttribute { name, value, span } => attrs.push(Attribute {
                name: LitStr::new(&name, span),
                value,
            }),
        }
    }

    let body = if input.peek(syn::token::Brace) {
        ElementBody::Block(parse_block(input)?)
    } else {
        input.parse::<Token![;]>()?;
        ElementBody::Empty
    };

    Ok(Element {
        name: LitStr::new(&normalize_name(&name_ident), name_ident.span()),
        id,
        class: if classes.is_empty() {
            None
        } else {
            let mut joined = String::new();
            for (index, class) in classes.iter().enumerate() {
                if index > 0 {
                    joined.push(' ');
                }
                joined.push_str(&class.value());
            }
            Some(LitStr::new(&joined, name_ident.span()))
        },
        attrs,
        body,
    })
}

struct ParsedAttribute {
    name: String,
    value: Option<AttributeValue>,
    span: Span,
}

fn parse_attribute(input: ParseStream<'_>) -> Result<ParsedAttribute> {
    let name_ident = input.call(Ident::parse_any)?;
    let span = name_ident.span();
    let name = normalize_name(&name_ident);

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

    Ok(ParsedAttribute { name, value, span })
}

fn parse_name_literal(input: ParseStream<'_>) -> Result<LitStr> {
    let ident = input.call(Ident::parse_any)?;
    Ok(LitStr::new(&normalize_name(&ident), ident.span()))
}

fn normalize_name(ident: &Ident) -> String {
    ident.to_string().replace('_', "-")
}
