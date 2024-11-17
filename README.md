# Making a Language Interpreter

<img src="https://craftinginterpreters.com/image/header-small.png" width="300">

https://craftinginterpreters.com/

This book walks through how to implement an interpreter for a scripting language.

Lox is a dynamically typed, interpreted scripting language designed by Robert Nystrom for his book "Crafting Interpreters".

## How to Run

### Running a Code File
```
$ cd rlox
$ cargo run my_code.lox
```

## Language Features
- operators
  - arithmetic (+, -, *, /)
  - Comparison (<, <=, =, >, >=)
  - logical (!, and, or)
- variables
- if statements
- loops
- Functions
- Closures
- Classes
- Inheiritance

## Interpreter Steps
```
     Raw Text Input
          |
          ▼
    Scanner/Lexxer
          |
          ▼
        Tokens
          |
          ▼
        Parser
          |
          ▼
  Abstract Syntax Tree
          |
          ▼
     Interpreter
          |
          ▼
     Code Executed
```

## Language Grammar
```
program     -> declaration* EOF;
```

### Declarations
```
declaration -> classDecl
             | funDecl
             | varDecl
             | statement ;

classDecl   -> "class" IDENTIFIER ( "<" IDENTIFIER )?
                "{" function* "}" ;
funDecl     -> "fun" function ;
varDecl     -> "var" IDENTIFIER ( "=" expression )? ";" ;
```

### Statements
```
statement   -> exprStmt
             | forStmt
             | ifStmt
             | printStmt
             | returnStmt
             | returnStmt
             | whileStmt
             | block ;

exprStmt    -> expression ";" ;
forStmt     -> "for (" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")"
                statement ;
ifStmt      -> "if (" expression ")" statement
                ( "else" statement )? ;
printStmt   -> "print" expression ";" ;
returnStmt  -> "return" expression? ";" ;
whileStmt   -> "while (" expression ")" statement ;
block       -> "{" declaration* "}" ;
```
### Expressions
```
expression  -> assignment ;

assignment  -> ( call "." )? IDENTIFIER "=" assignment | logic_or ;

logic_or    -> logic_and ( "or" logic_and )* ;
logic_and   -> equality ( "and" equality )* ;
equality    -> comparison ( ( "!=" | "==" ) comparison )* ;
comparison  -> term ( ( ">" | ">=" | "<" | "<=" ) term)* ;
term        -> factor ( ( "-" | "+" ) factor )*
factor      -> unary ( ( "/" | "*" ) unary )* ;

unary       -> ( "!" | "-" ) unary | call ;
call        -> primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
primary     -> "true" | "false" | "nil" | "this"
               | NUMBER | STRING | IDENTIFIER | "(" expression ")"
               | "super." IDENTIFIER ;
```

### Utility Rules
```
function    -> IDENTIFIER "(" parameters? ")" block ;
parameters  -> IDENTIFIER ( "," IDENTIFIER )* ;
arguments   -> expression ( "," expression )* ;
```
