use crate::priv_prelude::*;

pub struct TopLevelStatement {
    pub attribute_list: Vec<AttributeDecl>,
    pub item: Item,
}

impl TopLevelStatement {
    pub fn span(&self) -> Span {
        if self.attribute_list.is_empty() {
            self.item.span()
        } else {
            Span::join(self.attribute_list[0].span(), self.item.span())
        }
    }
}

impl Parse for TopLevelStatement {
    fn parse(parser: &mut Parser) -> ParseResult<TopLevelStatement> {
        let mut attribute_list = Vec::new();
        loop {
            // This is the only way I could get it to work but I'm not happy because we shouldn't
            // know/care about Attribute's syntax (that it starts with `#`) here.
            //
            // Ideally we'd try_parse() an attribute, if it fails then break and parse the item
            // below.  But if there's a malformed attribute it should be partially parsed and the
            // syntax error reported correctly.
            //
            // An alternative might be to loop and `take()` them while we can, but it uses Peek
            // which isn't a fully fledged parser and is really only for small keywords or
            // punctuation.
            //
            // Or perhaps to `enter_delimited()` and `parse_to_end()` except there is no delimiter
            // here; we have zero or more attributes before a declaration.
            if parser.peek::<HashToken>().is_some() {
                attribute_list.push(parser.parse()?);
            } else {
                break;
            }
        }
        let item = parser.parse()?;
        Ok(TopLevelStatement {
            attribute_list,
            item,
        })
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_top_level(input: &str) -> TopLevelStatement {
        let token_stream = crate::token::lex(&Arc::from(input), 0, input.len(), None).unwrap();
        let mut errors = Vec::new();
        let mut parser = Parser::new(&token_stream, &mut errors);
        match TopLevelStatement::parse(&mut parser) {
            Ok(item) => item,
            Err(_) => {
                //println!("Tokens: {:?}", token_stream);
                panic!("Parse error: {:?}", errors);
            }
        }
    }

    #[test]
    fn parse_attributes_none() {
        let stmt = parse_top_level(
            r#"
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(stmt.item, Item::Fn(_)));
        assert!(stmt.attribute_list.is_empty());
    }

    #[test]
    fn parse_attributes_fn_basic() {
        let stmt = parse_top_level(
            r#"
            #[foo]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(stmt.item, Item::Fn(_)));

        assert_eq!(stmt.attribute_list.len(), 1);

        let attrib = stmt.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_none());
    }

    #[test]
    fn parse_attributes_fn_two_basic() {
        let stmt = parse_top_level(
            r#"
            #[foo]
            #[bar]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(stmt.item, Item::Fn(_)));

        assert_eq!(stmt.attribute_list.len(), 2);

        let attrib = stmt.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_none());

        let attrib = stmt.attribute_list.get(1).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "bar");
        assert!(attrib.attribute.get().args.is_none());
    }

    #[test]
    fn parse_attributes_fn_one_arg() {
        let stmt = parse_top_level(
            r#"
            #[foo(one)]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(stmt.item, Item::Fn(_)));

        assert_eq!(stmt.attribute_list.len(), 1);

        let attrib = stmt.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("one"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);
    }

    #[test]
    fn parse_attributes_fn_empty_parens() {
        let stmt = parse_top_level(
            r#"
            #[foo()]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(stmt.item, Item::Fn(_)));

        assert_eq!(stmt.attribute_list.len(), 1);

        let attrib = stmt.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");

        // Args are still parsed as 'some' but with an empty collection.
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), None);
    }

    #[test]
    fn parse_attributes_fn_zero_and_one_arg() {
        let stmt = parse_top_level(
            r#"
            #[bar]
            #[foo(one)]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(stmt.item, Item::Fn(_)));

        assert_eq!(stmt.attribute_list.len(), 2);

        let attrib = stmt.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "bar");
        assert!(attrib.attribute.get().args.is_none());

        let attrib = stmt.attribute_list.get(1).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("one"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);
    }

    #[test]
    fn parse_attributes_fn_one_and_zero_arg() {
        let stmt = parse_top_level(
            r#"
            #[foo(one)]
            #[bar]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(stmt.item, Item::Fn(_)));

        assert_eq!(stmt.attribute_list.len(), 2);

        let attrib = stmt.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("one"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);

        let attrib = stmt.attribute_list.get(1).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "bar");
        assert!(attrib.attribute.get().args.is_none());
    }

    #[test]
    fn parse_attributes_fn_two_args() {
        let stmt = parse_top_level(
            r#"
            #[foo(one, two)]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(stmt.item, Item::Fn(_)));

        assert_eq!(stmt.attribute_list.len(), 1);

        let attrib = stmt.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("one"));
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("two"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);
    }

    #[test]
    fn parse_attributes_fn_zero_one_and_three_args() {
        let stmt = parse_top_level(
            r#"
            #[bar]
            #[foo(one)]
            #[baz(two,three,four)]
            fn f() -> bool {
                false
            }
            "#,
        );

        assert!(matches!(stmt.item, Item::Fn(_)));

        assert_eq!(stmt.attribute_list.len(), 3);

        let attrib = stmt.attribute_list.get(0).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "bar");
        assert!(attrib.attribute.get().args.is_none());

        let attrib = stmt.attribute_list.get(1).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "foo");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("one"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);

        let attrib = stmt.attribute_list.get(2).unwrap();
        assert_eq!(attrib.attribute.get().name.as_str(), "baz");
        assert!(attrib.attribute.get().args.is_some());

        let mut args = attrib
            .attribute
            .get()
            .args
            .as_ref()
            .unwrap()
            .get()
            .into_iter();
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("two"));
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("three"));
        assert_eq!(args.next().map(|arg| arg.as_str()), Some("four"));
        assert_eq!(args.next().map(|arg| arg.as_str()), None);
    }
}
