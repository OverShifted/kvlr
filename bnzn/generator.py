import argparse

import scanner, parser
from codegen import gen_server, gen_client

arg_parser = argparse.ArgumentParser(
                    prog='generator.py',
                    description='Generates Rust code from benzen source files',
                )#epilog='Text at the bottom of help')

arg_parser.add_argument('input_file')
arg_parser.add_argument('-os', '--output-server', required=True)
arg_parser.add_argument('-oc', '--output-client', required=True)

args = arg_parser.parse_args()

source = open(args.input_file).read()
tokens = scanner.Scanner(source).scan()

# print(*tokens, sep='\n')

ast = parser.Parser(tokens).parse()
# print(*ast, sep='\n')

print("Generating server...")
f = open(args.output_server, 'w')
f.write(gen_server(ast))
f.close()

print("Generating client...")
f = open(args.output_client, 'w')
f.write(gen_client(ast))
f.close()

print("Done.")
