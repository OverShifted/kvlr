from enum import Enum
from typing import Optional

class TokenType(Enum):
    HASH = 0
    LEFT_PAREN = 1
    RIGHT_PAREN = 2
    LESS = 3
    MORE = 4 # TODO: Better name?
    SEMICOLON = 5
    COMMA = 6
    ARROW = 7

    ID = 8
    NUM_LITERAL = 9

    EOF = -1


class Token:
    def __init__(self, token_type: TokenType, lexeme: Optional[str], line: int):
        self.token_type = token_type
        self.lexeme = lexeme
        self.line = line
