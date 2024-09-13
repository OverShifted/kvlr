from bnzn_token import TokenType

class Parser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.i = 0

        # Not really an AST
        self.ast = []

    def parse(self):
        while not self.is_at_end():
            self.parse_fn()

        return self.ast

    def parse_fn(self):
        if not self.match(TokenType.ID):
            print("Error: expected identifier", self.i)
            raise RuntimeError()

        fn_name = self.previous()
        if not self.match(TokenType.HASH):
            print("Error: expected '#'")
            raise RuntimeError()

        if not self.match(TokenType.NUM_LITERAL):
            print("Error: expected number")
            raise RuntimeError()

        fn_id = self.previous()

        if not self.match(TokenType.LEFT_PAREN):
            print("Error: expected '('")
            raise RuntimeError()
        
        args = self.arglist()

        if not self.match(TokenType.RIGHT_PAREN):
            print("Error: expected ')'")
            raise RuntimeError()
        
        if not self.match(TokenType.ARROW):
            print("Error: expected '->'")
            raise RuntimeError()

        ret = self.typename()

        if not self.match(TokenType.SEMICOLON):
            print("Error: expected ';'")
            raise RuntimeError()
        
        self.ast.append({
            "type": "fn",
            "name": fn_name.lexeme,
            "id": int(fn_id.lexeme),
            "args": args,
            "ret": ret,
        })

    def arglist(self):
        args = []

        while self.peek().token_type != TokenType.RIGHT_PAREN:
            args.append(self.typename())
            if not self.match(TokenType.COMMA):
                if self.peek().token_type != TokenType.RIGHT_PAREN:
                    print("Error: unexpected token after arglist", self.i, self.peek())
                    raise RuntimeError()
                
        return args

    def typename(self):
        if self.match(TokenType.LEFT_PAREN):
            # TODO: Tuple

            if self.match(TokenType.RIGHT_PAREN):
                # Void
                return "()"

        # TODO: Check false
        if not self.match(TokenType.ID):
            print("Error: unexpected token for typename")
            raise RuntimeError()

        name_str = self.previous().lexeme

        if self.peek().token_type == TokenType.LESS:
            while self.peek().token_type != TokenType.MORE:
                name_str += self.advance().lexeme

            name_str += self.advance().lexeme

        return name_str

    def is_at_end(self):
        # return self.i >= len(self.tokens)
        return self.peek().token_type == TokenType.EOF

    def advance(self):
        self.i += 1
        return self.tokens[self.i - 1]
    
    def peek(self):
        # return {} if self.is_at_end() else self.tokens[self.i]
        return self.tokens[self.i]
    
    def previous(self):
        return self.tokens[self.i - 1]
    
    def match(self, *types):
        for type_ in types:
            # print("matchidam", self.is_at_end(), self.peek(), type_)
            if not self.is_at_end() and self.peek().token_type == type_:
                self.advance()
                return True
            
        return False