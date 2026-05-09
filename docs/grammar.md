# AXON Grammar Reference v0.3.1

> Formal grammar specification for the AXON programming language.
> Copyright © 2026 Edison Lepiten — AIEONYX

---

## Notation

This document uses Extended Backus-Naur Form (EBNF):

```
rule     = production ;
?        = zero or one
*        = zero or more
+        = one or more
|        = alternation
( ... )  = grouping
[ ... ]  = character class
```

---

## Top Level

```ebnf
program       = module_decl? import* top_level_item* ;

module_decl   = "module" ident_path NEWLINE ;

import        = "import" ident_path ("as" IDENT)? NEWLINE ;

ident_path    = IDENT ("." IDENT)* ;

top_level_item
  = fn_decl
  | task_decl
  | struct_decl
  | enum_decl
  | type_alias
  | const_decl
  | impl_block
  | trait_decl
  ;
```

---

## Declarations

### Functions

```ebnf
fn_decl     = decorator* "fn" IDENT generic_params? params ret_type? ":" NEWLINE block ;

task_decl   = decorator* "task" IDENT generic_params? params uses_clause? ret_type? ":" NEWLINE block ;

params      = "(" (param ("," param)*)? ")" ;

param       = IDENT ":" type ;

ret_type    = "->" type ;

uses_clause = "uses" "[" ident_path ("," ident_path)* "]" ;
```

### Structs

```ebnf
struct_decl = decorator* "struct" IDENT generic_params? ":" NEWLINE INDENT field_decl+ DEDENT ;

field_decl  = IDENT ":" type NEWLINE ;
```

### Enums

```ebnf
enum_decl   = decorator* "enum" IDENT generic_params? ":" NEWLINE INDENT variant_decl+ DEDENT ;

variant_decl = IDENT ("(" field_decl+ ")")? NEWLINE ;
```

### Type Aliases

```ebnf
type_alias  = "type" IDENT "=" type NEWLINE ;
```

### Constants

```ebnf
const_decl  = "const" IDENT ":" type "=" expr NEWLINE ;
```

---

## Decorators

```ebnf
decorator   = "@" decorator_name decorator_args? NEWLINE ;

decorator_name
  = IDENT ("." IDENT)*
  | "program_intent"
  ;

decorator_args = "(" decorator_arg ("," decorator_arg)* ")" ;

decorator_arg  = (IDENT ":")? expr ;
```

---

## Blocks and Statements

```ebnf
block       = INDENT stmt+ DEDENT ;

stmt
  = pass_stmt
  | let_stmt
  | mut_stmt
  | assign_stmt
  | return_stmt
  | if_stmt
  | for_stmt
  | while_stmt
  | match_stmt
  | defer_stmt
  | expr_stmt
  ;

pass_stmt   = "pass" NEWLINE ;

let_stmt    = "let" IDENT (":" type)? "=" expr NEWLINE
            | "let@" IDENT (":" type)? "=" expr NEWLINE ;

mut_stmt    = "mut" IDENT (":" type)? "=" expr NEWLINE ;

assign_stmt = assign_target "=" expr NEWLINE ;

assign_target
  = IDENT
  | expr "." IDENT
  | expr "[" expr "]"
  ;

return_stmt = "return" expr? NEWLINE ;

if_stmt     = "if" expr ":" NEWLINE block
              ("elif" expr ":" NEWLINE block)*
              ("else" ":" NEWLINE block)? ;

for_stmt    = "for" IDENT "in" expr ":" NEWLINE block ;

while_stmt  = "while" expr ":" NEWLINE block ;

match_stmt  = "match" expr ":" NEWLINE INDENT match_arm+ DEDENT ;

match_arm   = pattern "=>" (expr | block) NEWLINE ;

defer_stmt  = "defer" expr NEWLINE ;

expr_stmt   = expr NEWLINE ;
```

---

## Patterns

```ebnf
pattern
  = "_"                         # wildcard
  | IDENT                       # binding
  | literal                     # literal match
  | IDENT "(" pattern* ")"      # constructor
  | pattern "|" pattern         # or-pattern
  ;
```

---

## Expressions

```ebnf
expr
  = literal
  | IDENT
  | expr "." IDENT              # field access
  | expr "[" expr "]"           # index
  | expr "(" args ")"           # call
  | expr binop expr             # binary operation
  | unop expr                   # unary operation
  | "if" expr "then" expr "else" expr    # if expression
  | "return" expr?              # return expression
  | "(" expr ")"                # grouping
  | string_interp               # interpolated string
  ;

args        = (arg ("," arg)*)? ;
arg         = (IDENT ":")? expr ;

binop
  = "+" | "-" | "*" | "/" | "%"          # arithmetic
  | "==" | "!=" | "<" | ">" | "<=" | ">=" # comparison
  | "and" | "or"                          # logical
  | "&" | "|" | "^" | "<<" | ">>"        # bitwise
  | "??"                                  # null coalescing
  ;

unop = "-" | "not" | "~" ;

string_interp = '"' (char | "${" expr "}")* '"' ;
```

---

## Literals

```ebnf
literal
  = INT_LIT
  | FLOAT_LIT
  | BOOL_LIT
  | STR_LIT
  | BYTES_LIT
  | "None"
  ;

INT_LIT     = [0-9]+ | "0x" [0-9a-fA-F]+ | "0b" [01]+ ;
FLOAT_LIT   = [0-9]+ "." [0-9]+ ;
BOOL_LIT    = "true" | "false" ;
STR_LIT     = '"' [^"]* '"' ;
BYTES_LIT   = "b'" [^']* "'" ;
```

---

## Types

```ebnf
type
  = primitive_type
  | IDENT                        # named type
  | "Option" "<" type ">"
  | "Result" "<" type "," type ">"
  | "List" "<" type ">"
  | "Map" "<" type "," type ">"
  | "Set" "<" type ">"
  | "(" type ")"
  ;

primitive_type
  = "Int" | "Int32" | "Int8"
  | "UInt" | "UInt32" | "UInt8"
  | "Float" | "Float32"
  | "Bool" | "Char" | "Str" | "Bytes"
  ;
```

---

## Generics

```ebnf
generic_params = "<" generic_param ("," generic_param)* ">" ;

generic_param  = IDENT (":" trait_bound)? ;

trait_bound    = IDENT ("+" IDENT)* ;
```

---

## Tokens

```ebnf
IDENT       = [a-zA-Z_] [a-zA-Z0-9_]* ;

NEWLINE     = "\n" ;

INDENT      = increase in indentation level ;

DEDENT      = decrease in indentation level ;

COMMENT     = "#" [^\n]* ;
```

### Keywords

```
and         as          assert      async
await       break       const       continue
defer       elif        else        enum
false       fn          for         if
impl        import      in          let
match       mod         module      mut
not         or          pass        pub
return      self        struct      task
then        trait       true        type
uses        while
```

### Operators

```
+   -   *   /   %   **
==  !=  <   >   <=  >=
and or  not
&   |   ^   <<  >>  ~
=   +=  -=  *=  /=
.   ..  ...
->  =>  ?   ??  @
```

---

## Indentation Rules

1. The first non-empty, non-comment line sets the base indentation (0)
2. A block begins after `:` on the preceding line
3. All statements in a block must have identical indentation
4. Dedentation closes one block per level
5. Tabs are not permitted — spaces only
6. Blank lines are ignored for indentation purposes
7. A blank line between top-level items is recommended

---

## Grammar Changes from v0.3 to v0.3.1

- Added `let@` ephemeral binding syntax
- Added `uses` clause to task declarations
- Added `@program_intent` as a recognized decorator name
- Added `??` null-coalescing operator
- Clarified that blank lines between top-level items are required
  for the parser to correctly associate decorators with their targets

---

*Version 0.3.1 — May 2026*
*Copyright © 2026 Edison Lepiten — AIEONYX*
