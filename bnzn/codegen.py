from jinja2 import Environment, PackageLoader, select_autoescape
import re

env = Environment(
    loader=PackageLoader("templates"),
    # autoescape=select_autoescape()
)

camel2snake_re = re.compile(r"(?<=[a-z])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])")

def camel2snake(camel):
    return camel2snake_re.sub('_', camel).lower()

def camel2upper_camel(camel):
    return camel[0].upper() + camel[1:]

def gen_server(ast):
    template = env.get_template("server_template.rs.jinja")
    fns = []

    for fn_ast in ast:
        args_name_type = ', '.join(map(lambda i: f"arg{i[0]}: {i[1]}", enumerate(fn_ast["args"])))
        args_type_maybe_pipelined = ', '.join(map(lambda i: f"MaybePipelinedValue<{i}>", fn_ast["args"]))

        fns.append({
            "name": camel2snake(fn_ast["name"]),
            "id": fn_ast["id"],
            "args_type": fn_ast["args"],
            "args_name_type": args_name_type,
            "args_type_maybe_pipelined": args_type_maybe_pipelined,
            "ret": fn_ast["ret"],
        })

    return template.render(fns=fns, len=len)

def gen_client(ast):
    template = env.get_template("client_template.rs.jinja")
    fns = []

    for fn_ast in ast:
        pub_args_name_type = ',\n    '.join(map(lambda i: f"pub arg{i[0]}: {i[1]}", enumerate(fn_ast["args"])))
        pub_args_name_type_maybe_pipelined = ',\n    '.join(map(lambda i: f"pub arg{i[0]}: MaybePipelinedValue<{i[1]}>", enumerate(fn_ast["args"])))

        fns.append({
            "name": camel2upper_camel(fn_ast["name"]),
            "id": fn_ast["id"],
            "pub_args_name_type": pub_args_name_type,
            "pub_args_name_type_maybe_pipelined": pub_args_name_type_maybe_pipelined,
            "ret": fn_ast["ret"],
        })

    return template.render(fns=fns, len=len)
