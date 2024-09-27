from typing import Optional

from bnzn_token import Token, TokenType

class Scanner:
    def __init__(self, source: str):
        self.source = source
        self.tokens = []
        self.i = 0
        self.line = 1

    def add_token(self, token_type: TokenType, lexeme: Optional[str] = None):
        self.tokens.append(Token(token_type, lexeme, self.line))

    def scan(self):
        while not self.is_at_end():
            self.scan_token()

        self.add_token(TokenType.EOF)

        return self.tokens

    def scan_token(self):
        c = self.advance()
        match c:
            case '#':
                self.add_token(TokenType.HASH, "#")

            case '(':
                self.add_token(TokenType.LEFT_PAREN, "(")

            case ')':
                self.add_token(TokenType.RIGHT_PAREN, ")")

            case '<':
                self.add_token(TokenType.LESS, "<")

            case '>':
                self.add_token(TokenType.MORE, ">")

            case ';':
                self.add_token(TokenType.SEMICOLON, ";")

            case ',':
                self.add_token(TokenType.COMMA, ",")

            case '-':
                if self.match('>'):
                    self.add_token(TokenType.ARROW, "->")
                else:
                    print(f"[line {self.line}] Unexpected character after '-': '{self.peek()}'")

            case '/':
                if self.match('/'):
                    # Comment
                    while self.peek() != '\n' and not self.is_at_end():
                        self.advance()
                else:
                    print(f"[line {self.line}] Unexpected character after '/': '{self.peek()}'")

            case ' ' | '\r' | '\t':
                pass

            case '\n':
                self.line += 1

            case _:
                if c.isalpha():
                    self.identifier(c)
                elif c.isnumeric():
                    self.number(c)
                else:
                    print(f"[line {self.line}] Unexpected character: '{c}'")

    def identifier(self, initial_character):
        value = initial_character[:]
        while self.peek().isalnum():
            value += self.advance()

        self.add_token(TokenType.ID, value)

    def number(self, initial_character):
        value = initial_character[:]
        while self.peek().isnumeric():
            value += self.advance()

        self.add_token(TokenType.NUM_LITERAL, value)

    def is_at_end(self):
        return self.i >= len(self.source)

    def advance(self):
        self.i += 1
        return self.source[self.i - 1]
    
    def peek(self):
        return '\0' if self.is_at_end() else self.source[self.i]
    
    def match(self, expected):
        if self.is_at_end() or self.source[self.i] != expected:
            return False
        
        self.i += 1
        return True
