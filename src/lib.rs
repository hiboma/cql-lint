pub mod diagnostic;
pub(crate) mod lexer;
pub mod linter;
pub(crate) mod parser;

#[cfg(test)]
mod debug_tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn debug_parse_grouped_func() {
        let source = "(bazQux(x))";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        eprintln!("Tokens: {:?}", tokens);
        let parser = Parser::new(tokens);
        let (query, errors) = parser.parse();
        eprintln!("Errors: {:?}", errors);
        eprintln!("Query: {:?}", query);
    }

    #[test]
    fn debug_parse_not_func() {
        let source = "not bazQux(x)";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        eprintln!("Tokens: {:?}", tokens);
        let parser = Parser::new(tokens);
        let (query, errors) = parser.parse();
        eprintln!("Errors: {:?}", errors);
        eprintln!("Query: {:?}", query);
    }
}

#[cfg(test)]
mod debug_tests2 {
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn debug_parse_not_func2() {
        let source = "status=200 and not bazQux(x)";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (query, errors) = parser.parse();
        eprintln!("Errors: {:?}", errors);
        eprintln!("Query: {:?}", query);
    }

    #[test]
    fn debug_parse_grouped_func2() {
        let source = "status=200 and (bazQux(x))";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (query, errors) = parser.parse();
        eprintln!("Errors: {:?}", errors);
        eprintln!("Query: {:?}", query);
    }

    #[test]
    fn debug_parse_not_groupby() {
        let source = "not groupBy(field=src)";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (query, errors) = parser.parse();
        eprintln!("Errors: {:?}", errors);
        eprintln!("Query: {:?}", query);
    }

    #[test]
    fn debug_parse_grouped_groupby() {
        let source = "(groupBy(field=src))";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (query, errors) = parser.parse();
        eprintln!("Errors: {:?}", errors);
        eprintln!("Query: {:?}", query);
    }
}
