<!-- Style guide: One sentence per line. This makes diffing and merging a lot easier. -->

# Rokugo language design

This is a living, informal document. It will be changed and expanded as progress on the langauge goes forward.

The goals of this language's design are:

- **Be predictable.** The foundation of building reliable software is to write code that is easy to predict.
  Therefore it must be easy to predict what a given function is capable of doing, based on its signature alone.
- **Be nice.** Reading and writing code in Rokugo should be a pleasure at the syntactic level.
  Following that, postfix is better than prefix.
- **Be pragmatic.** Of all these goals, perhaps the most important one.
  Rokugo does not strive to be the most powerful language.
  It does not invent new compilation algorithms.
  It does not feature powerful type inference.

Most important of all, Rokugo is not meant to be a systems programming language.
It is built for embedding in other applications for a reason.
We already have programming languages that would fulfill Rokugo's goals in the systems space (have you seen Rust? I heard it's pretty good!)

Rokugo is a language for applications, not systems.

## Source code

Rokugo source code is UTF-8 encoded.
Unicode U+000A `LINE FEED` is used as the line separator.
The characters U+0020 `SPACE`, U+0009 `CHARACTER TABULATION`, U+000D `CARRIAGE RETURN` are recognized as whitespace and ignored.

## File extensions

Rokugo uses the following file extensions:

| Extension | Purpose |
| --- | --- |
| `.rk` | Source code |
| `.rka` | Compiled archive |
| `.rkx` | Compiled archive, with an executable entry point |
| `.rkd` | Debug symbols |

### Comments

Two types of line comments are recognized.
The first one is regular comments, starting with `#`:

```rokugo
# This is a line comment.
```

The second one is documentation comments, starting with `##`.
Note that while regular comments are a purely lexical construct ignored by the parser, documentation comments are recognized as part of the syntax tree, and as such can only appear above items.

```rokugo
## This is a documentation comment.
## An item must follow for it to be valid syntax.
let variable : Int32 = 1
```

## Expressions

### Primitives

Rokugo supports a few types of primitive literals.

First of all, `Integer` and `Decimal`:

```rokugo
123      # Integer
123.456  # Decimal
```

These have infinite storage precision and no runtime representation.
To be used in a runtime context, they must be hinted towards concrete [runtime integer types](#numbers) using the [`of` operator](#-type-ascription-operator).

Then there are strings:

```rokugo
"Hello, world!"
```

Strings are UTF-8 encoded and have the type `String`.

Notably, string literals cannot contain embedded newlines.

```rokugo
"Hello,
world"  # error: string literals may not span multiple lines
```

There is currently no way around this as Rokugo also does not have a multiline string literal.

Strings are made up of characters, which have the type `Char` and use single-quoted literals:

```rokugo
'a'  # U+0061
'ł'  # U+0142
```

The `Char` type represents any valid Unicode scalar value.

The following escape sequences are supported in `String` and `Char` literals:

| Escape | Code point | Description |
| --- | --- | --- |
| `\\` | U+005C | Literal backslash |
| `\'` | U+0027 | Literal apostrophe |
| `\"` | U+0022 | Literal quote |
| `\n` | U+000A | Line feed |
| `\r` | U+000D | Carriage return |
| `\t` | U+0009 | Horizontal tab |
| `\u{x}` | x | Unicode scalar value `x` (one or more hexadecimal digits) |

### Compounds

Rokugo features the following types of compound literals.

#### Records

Records are ordered, heterogenous containers storing key-value pairs.
They are delimited by braces.

```rokugo
{ greeting = "hello", target = "world" }
```

Record fields can be accessed using the dot `.` operator.

```rokugo
{ greeting = "hello", target = "world" }.greeting  # "hello"
```

Order of fields is significant, and influences sort order (`(<)` and other comparison operators.)
For instance, to represent [SemVer](https://semver.org/) with records, you want to order your fields such that `major` is first.

```rokugo
let SemVer = {
    major = Nat32,
    minor = Nat32,
    patch = Nat32,
}
```

Existing fields can be modified, as well as new fields can be appended using the [`with` operator](#with).

Record fields can be initialized in a shorthand way from existing variables by omitting the `= value` part.

```rokugo
let x = 1
let y = 2
let vector = { x, y }
```

Similar syntax is supported in [record patterns](#record-patterns).

#### Tuples

Tuples are ordered, heterogenous containers delimited by parentheses.
Any number of elements is supported.

```rokugo
(1, 2)
(1, 2,)  # Trailing commas are allowed
```

The single-element tuple must be written with a trailing comma:

```rokugo
(1,)
```

The zero-element tuple is also called the _unit_ tuple, and is often used to denote the lack of any meaningful value.

```rokugo
()
```

Tuple fields can be accessed via names `_0`, `_1`, ..., `_n` where `n` is the number of elements.

#### Tags

Tags are values identified by their name. A tag literal is written like so:

```rokugo
:some_tag
```

Note how Rokugo lacks a literal for boolean values.
This is because booleans are expressed using the `:false` and `:true` tags. The type `Bool` is defined as:

```rokugo
let Bool = :false | :true
```

Tags can carry a payload value along with them. To attach a payload, [function application](#function-application) syntax can be used:

```rokugo
:meters 1
```

More than one value can be attached as the payload:

```rokugo
:rgba 5 142 240 255
```

Tags are mostly used with [unions](#union-types), but can be useful on their own to attach unit information to values.

### Operators

#### User-defined operators

All operators except [magic operators](#magic-operators) are ordinary, user-defined [functions](#functions).

An operator is made up of a sequence of one or more of the following characters:

```text
+ - * / %
< = > ! ?
^ & | ~ $
. : @
```

The operators `.`, `:`, `=`, `=>`, `@`, `&`, and `|` have a [special meaning to the compiler](#magic-operators) and are reserved.

#### Magic operators

The following operators have a special meaning to the compiler.

##### `.` field access operator

`a.x` accesses the field `x` of the tuple or record `a`.

##### `:` type ascription operator

`expr : T` coerces the `expr` to be of type `T`.
If the coercion cannot be performed, a compilation error is emitted.

Type ascription is used in many places throughout this documentation to explicitly denote the types of expressions.

##### `=` binding operator

The `=` operator is used for pattern matching irrefutably, as well as a separator in `fun` bindings and in record field bindings.

Most commonly, `=` is used to give a name to some value using `let`:

```rokugo
  let x = 1
#         ^ value: match on 1
# ^^^^^ pattern: bind a new variable `x`
```

##### `|` union operator

`a | b` represents a [type union](#union-types).

##### `&` product operator

`a & b` represents a product type, which is a subtype of both `a` and `b` at the same type.

Note that the union and product operators may also be used as noops in prefix form, to help with aligning item declarations vertically:

```rokugo
let Colors =
#   ↓ used as a cosmetic prefix here
    | :red
    | :yellow
    | :green
    | :blue
```

##### `->` function type operator

`a -> r` is the type of a function which takes an argument of type `a` and returns a value of type `r`.
It is the only operator in Rokugo which is right-associative rather than left-associative.

##### `@` annotation sigil

Not actually an operator, but a token reserved for future use in annotations.

##### `and` and `or`

`and` and `or` are short-circuiting boolean control flow operators.

`and` represents the boolean AND operation, while `or` represents the OR operation.

The operators are short-circuiting because if the result of the operation is known from evaluating the expression on the left-hand side, the right-hand side will not be evaluated.

- If the left-hand side of `and` is `:false`, the result of the whole expression is known to be `:false` and as such the right-hand side will not be evaluated.
- If the left-hand side of `or` is `:true`, the result of the whole expression is known to be `:true` and as such the right-hand side will not be evaluated.

##### `with`

`with` is an operator that allows for updating a record's fields.

For example:

```rokugo
let joe = { name = "Joe", age = 32 : Nat32 }
let jane = joe with { name = "Jane" }
```

`with` returns a new record with the specified fields changed to some other values.
Fields from the old record can still be accessed like usual in the new record's fields.

#### Precedence

A unique thing about Rokugo expressions is how it handles precedence.

TODO: Precedence categories, how operators from different categories do not interact.

### Variables

Variables in Rokugo are declared using the `let` keyword:

```rokugo
let greeting = "Hello!"
```

By default, the type of the variable is inferred from its right-hand side.
However, in some cases an explicit type has to be provided if the inferred type cannot be represented at runtime and there are multiple choices available (as is the case with numbers.)

An explicit type can be given by using [type ascription `:`](#-type-ascription-operator) on the left-hand side of the `=`:

```rokugo
fun main () : () = do {
    # An explicit type must be provided here, because type inference in functions favors
    # runtime-representable types (such as Nat32) over compilation-exclusive types
    # (such as Integer,) and the ambiguity between which exact integer type to choose would cause
    # a compilation error.
    let x : Nat32 = 12345
}
```

Variables can be referenced using their bare name, without any sigils:

```rokugo
let two : Int32 = 2
let four = two + two
```

Some algorithms are easier expressed imperatively rather than functionally.
For that, Rokugo allows you to declare variables as mutable, using `var`:

```rokugo
var x : Int32 = 2
```

A new value can be given to the variable by using `set`.

```rokugo
set x = 3
```

Note however that `var` variables cannot be captured in [lambdas](#function-as-first-class-values).
Instead, the variable has to be rebound as a `let` before it's captured.

```rokugo
var x : Int32 = 2
set x = 3
let x = x  # Shadow the existing `x`, but make it immutable
let multiply_by_x = fun (y : Int32) = x * y
```

This is not the case for [effect handlers](#effect-system), which represent a side effect explicitly and are therefore allowed to capture and modify outer variables.

#### Scope and `do {}` blocks

Declared variables can be referenced until the end of the surrounding block `{}`.
After that point they go out of scope and can no longer be read.

It is possible to create a new scope for variables using a `do {}` block.
They also function as a way to execute a sequence of expressions with side effects, discarding their results.
`do {}` blocks are expressions themselves; the result of a `do {}` block is the last expression in the block.

```rokugo
let some_vector = do {
    let x = 2
    let y = x * 5
    { x = x, y = y }
}
# x and y are no longer reachable here
```

### Functions

The primary way of structuring computation in Rokugo is through _functions._

It is important to know that functions in Rokugo are _first-class_, meaning that they're a value like any other.
This means you can pass them around as parameters to other functions, bind them to variables, etc. which comes in useful when building reusable code.

#### Function declaration

Functions are declared using the `fun` keyword, followed by the function name, followed by the function's parameters, followed by the function definition after `=`.

```rokugo
fun greeting name =
    String.cat "Hello, " name
```

Note that although explicit types are not given in the function's signature, the function is fully statically typed.
The type of `name` is inferred from the function's body, which passes `name` as an argument to `String.cat`, expected to be a `String`.

Signature type inference is allowed within [file-local and `internal`](#item-privacy-public-types-and-internal-types) functions.
Any functions which are visible outside the current package must specify the function signature explicitly.
This can be done by [ascribing a type](#-type-ascription-operator) to each of the function's parameters, as well as the function itself.

```rokugo
fun greeting (name : String) : String =
    String.cat "Hello, " name
```

This aids in documentation, as well as keeping the interface of the function stable across versions.

There is one exception to this rule, and it's when the return type of the function is dependent on its input values.
This is precisely what happens in the case of polymorphic types, or types which can be parametrized based on a set of input parameters.
In that case specifying the return type of the function would result in redundantly having to perform a _type of_ operation by hand, so it needn't be done:

```rokugo
fun Vec2 (t : _) = { x = t, y = t }
#               ^ return type is omitted
```

If a function performs any [side effects](#effect-system), they are also inferred automatically for module-local functions.
Public functions may specify them using the `~` operator after the function's type.

```rokugo
fun greet (name : String) : () ~ Console =
    Console.write_line (greeting name)
```

Custom operators can be defined by parenthesizing the operator name:

```rokugo
## Concatenation of two strings.
fun ($) (a : String) (b : String) : String =
    String.cat a b
```

Custom operator functions can also be referred to (and partially applied) by parenthesizing the operator name.

```rokugo
# Create an alias for the + operator.
let add = (+)

# Partially apply (+).
let add_two = (+) 2
```

Note that Rokugo does not support function overloading.
For a function to be applicable to many different types of parameters, it has to be polymorphic.

The type of a function which accepts an argument of type `a` and returns a value of type `r` is written as `a -> r`.
Unlike most operators, `->` is right-associative, so `a -> b -> c` is `a -> (b -> c)`, not `(a -> b) -> c`.

Note that there are no functions which accept more than one argument.
This helps with programming generically, because there's only one argument passing case you need to worry about - where you pass a single argument to the function.

#### Function application

_Applying_ a function in Rokugo is what you might call _calling_ a function in imperative languages.
The exact details are quite [mathy](https://en.wikipedia.org/wiki/Function_application), but this is because Rokugo, as well as functional programming languages in general, are more about _obtaining results_ rather than _performing actions._

As already shown in examples above, function application does not use any delimiters.
This is for a few reasons:

- Applying functions is very common, thus it helps to keep the operation concise.
- It interacts naturally with currying (see below.)

For example, to apply a function with two arguments, you only specify them one after another:

```rokugo
fun double x = x * 2

let numbers = List.map double (List.of (1, 2, 3))
```

Note how `List.of` has to be parenthesized; this is because of how function application is secretly an infix operation, just like `+` and other operators.
Therefore the following call, without parenthesizing the 2nd argument:

```rokugo
List.map double List.of (1, 2, 3)
```

in C-like terms, would be parsed as:

```rokugo
List.map(double, List.of, (1, 2, 3))
```

which is not what we want.
Naturally, the compiler will tell you about this mistake by emitting a type error.

Additionally, newlines are not allowed between function arguments (but are fine within arguments themselves.)
This is to prevent ambiguities with side-effectful functions in `do {}` blocks:

```rokugo
fun main () : () ~ Console = do {
    Console.write_line "Hello, world!"
    Console.write_line "This is another call."
}
```

If arguments to a single call were allowed to continue over many lines, the above code would be interpreted as a single call:

```rokugo
fun main () : () ~ Console = do {
    Console.write_line "Hello, world!" Console.write_line "This is another call."
}
```

which is not what you want.
If you find yourself writing functions with very many arguments, consider using a record to name the arguments:

```rokugo
fun function_with_very_many_arguments
    (args : {
        first_argument = Int32,
        second_argument = Int32,
        third = Int32,
        fourth = Int32,
        final = Int32,
    })
: () = do {
    # ...
}
```

As an example of this, consider how the pipeline operator is implemented:

```rokugo
fun (|>) (value : let a) (function : a -> let r) : r =
    function value
```

If Rokugo used argument _tuples_ instead, `|>` would need to be implemented as [magic](#magic-operators) syntax sugar, like it is [in Gleam](https://gleam.run/book/tour/functions.html#pipe-operator), or a macro [as in Elixir](https://github.com/elixir-lang/elixir/blob/d716bc2703c0ee3647c9f988d68ddb6100080022/lib/elixir/lib/kernel.ex#L4232).
Thanks to the composability of single-argument functions and currying, it can be implemented in the standard library instead.

Some functional languages like Haskell are lazy, while others, like OCaml and Rokugo, are strict.
This means function arguments are evaluated in a strictly specified order before the function is called, rather than on-demand as the function uses them.

Because of this, as well as functions being [first-class](#function-as-first-class-values), it is impossible to have a function without parameters.
To signal that a function does not accept any meaningful arguments, make it accept a single `()` parameter, as is the case with `main`.

```rokugo
fun main () : () = ()
```

Rokugo's evaluation order for arguments is left-to-right.
This is somewhat counterintuitive on a theoretical level, because after all, Rokugo's functions accept one argument and return one value.
Therefore you'd think the function application operator is a binary operator like any other syntax-wise:

```ebnf
application = expr, expr;
```

While in reality, function application consumes as many arguments as it can.

```ebnf
application = expr, expr, { expr };
```

The reason for this is that it's just more practical that way.
When reading functions, you can easily expect that expressions will be evaluated from left to right rather than the opposite order, which matches the order you read English in.

#### Function as first-class values

With Rokugo sporting first-class functions, naturally there must be a way of creating a function _ad hoc._
For this, a `fun` literal (or lambda) can be used:

```rokugo
let double = fun x = x * 2
```

Syntactically, lambdas are similar to regular function declarations, except they lack the function name.
Type and effect ascription can be used as in named function declarations.
This has to be done when types cannot be inferred in a runtime contexts, because type polymorphism is only supported during compilation.

This means the following example will not compile:

```rokugo
fun main () : () = do {
    let multiply = fun x y = x * y
}
```

Instead, the types of `x` and `y` have to be specified explicitly:

```rokugo
fun main () : () = do {
    let multiply = fun (x : Int32) (y : Int32) = x * y
}
```

Note however that the return type and effects can be omitted, because they can be inferred from the lambda's body.

##### Currying and partial application

As mentioned before, functions in Rokugo always take one argument, and always return one value.
For declarations of functions with multiple parameters, Rokugo automatically performs [currying](https://en.wikipedia.org/wiki/Currying).

Functions declared to accept more than one argument are actually functions which return other functions:

```rokugo
# This function:
fun multiply (x : Int32) (y : Int32) : Int32 = x * y

# is the same as:
let multiply =
    fun (x : Int32) : Int32 -> Int32 =
        fun (y : Int32) : Int32 =
            x * y
```

Applying `multiply` to one argument and leaving the other one out would be an example of _partial application,_ because we apply only part of the function's arguments.
Partially applying `multiply` leaves us with another function we can call to obtain the final result:

```rokugo
let double = multiply 2
let four = double 2
```

#### Function polymorphism

Functions can be polymorphic over their parameter types.
Type parameters can be introduced using `let p` syntax, where `p` is the name of the type parameter.
These are used in place of regular type annotations:

```rokugo
fun map (f : let a -> let b) (list : List a) : List b =
    List.iter list
    |> Iterator.map f
    |> List.from_iterator
```

Polymorphism alone isn't enough to drive all generic code however, as sometimes we'd like to ensure our polymorphic types support certain operations.

For that, it is possible to declare implicit module parameters that will be searched for when the function is instantiated.
Implicit module parameters are declared with _`use` declarations_ before the function's equals `=` sign.
For example, to declare that the function requires an implementation of `Add` that works on the type parameter `a`:

```rokugo
fun Vec2 t = { x = t, y = t }

fun add (a : Vec2 (let t)) (b : Vec2 t) : Vec2 t
use A : Add with { T = t } =
    { x = A.add a.x b.x, y = A.add a.y b.y }
```

Now because a module supporting `Add with { T = t }` is in scope, the `+` operator can be used as well, as it also requires a module which implements `Add`.

```rokugo
# Definition of + from the standard library:
fun (+) (a : let t) (b : t) : t
use A : Add with { T = t } =
    A.add a b

fun add (a : Vec2 (let t)) (b : Vec2 t) : Vec2 t
use A : Add with { T = t } =
    { x = a.x + b.x, y = a.y + b.y }
#             ^              ^
```

In function types, implicit modules are represented with a single record parameter that comes before the function's declared parameters.

```rokugo
add : { A = Add with { T = let t } } -> Vec2 t -> Vec2 t -> Vec2 t
```

Note however that the record type is in no way special.
This is because implicit parameters are only resolved when a function is declared with `fun`.

The reason why the record parameter comes before other parameters is so that specializations of the function may be conveniently created and passed around as parameters, which cannot be polymorphic.

```rokugo
let add_int_vectors = add { A = Int32.Add }
```

In simpler terms, it's so that the _implicit module record is easy to get rid of._

TODO: Establish implicit module search rules. Right now I'm thinking of something along the lines of:

- For an implicit module `M` declared to implement an interface `I`
  - Recursively find all modules within the current scope that implement the interface `I`
    - If there is exactly one, use it.
    - If there is more than one, emit a compilation error.

But I don't know how to make it interact with module _functions_ (and whether to make them interact at all.)

### Conditionals and control flow

#### `if` expression

The simplest way of branching control flow in Rokugo is the `if` expression.

The `if` expression takes the form:

```rokugo
if condition then true_expression
else false_expression
```

`condition` is expected to be a `Bool` (`:true | :false`) telling the `if` whether to take the `true_expression` branch or the `false_expression` branch.

For example, to check a number's parity:

```rokugo
fun parity x =
    if x % 2 == 0 then :even
    else :odd
```

If more than one statement needs to be executed for side effects or extra `let` bindings, `if` can be paired with `do {}`:

```rokugo
let CacheEntry = ...  # example module defined elsewhere

fun evict (entry : CacheEntry) : (:evicted | :retained CacheEntry) = do {
    let entry = entry with { eviction_timer = entry.eviction_timer - 1 }
    if entry.eviction_timer == 0 then do {
        CacheEntry.destroy entry
        :evicted
    } else do {
        :retained entry
    }
}
```

#### `while` expression

Rokugo is a multi-paradigm language.
Although it is functional at its core, some algorithms are better and more efficiently expressed using imperative loops.
This is what `while` is for - it repeatedly executes an expression _while_ a condition is `:true`.

```rokugo
fun factorial (n : Nat32) : Nat32 = do {
    var i : Nat32 = 1
    var x : Nat32 = 1
    while i <= n do {
        set x = x * i
        set i = i + 1
    }
    x
}
```

Note that because `while` pretty much always has side effects, a `do` block is _required_ after the condition.

The result of a `while` expression is `()`.
There is no other meaningful result that could be returned, since the body of a `while` may get executed zero or more times.

#### `match` expression

`match` allows for matching an expression sequentially against a set of [patterns](#patterns).
It returns a function, which is most commonly used together with the `|>` operator:

```rokugo
let Rgb = :rgb Float32 Float32 Float32
let Rgba = :rgba Float32 Float32 Float32 Float32
let Color = :transparent | Rgb | Rgba

fun to_rgba (color : Color) : Rgba =
    color |> match {
        :transparent => :rgba 0.0 0.0 0.0 0.0
        :rgb (let r) (let g) (let b) => :rgba r g b 1.0
        :rgba _ _ _ _ => color
    }
```

The `|>` can be omitted however, yielding a shorthand way to write functions which match on an input and transform it into something else.

```rokugo
let to_rgba : Color -> Rgba = match {
    :transparent => :rgba 0.0 0.0 0.0 0.0
    :rgb (let r) (let g) (let b) => :rgba r g b 1.0
    :rgba _ _ _ _ => color
}
```

Each `pattern => expr` pair is called an _arm_.

The patterns in a `match` expression are irrefutable, and together must cover all the posibilities of the input range (be exhaustive.)

#### `is` expression

`is` allows for checking whether an expression matches a single irrefutable [pattern](#patterns).

```rokugo
let Color = :red | :yellow | :green | :blue

fun yeah (color : Color) : () ~ Console =
    if color is :blue then
        Console.write_line "I'm blue daba dee daba dye"
    else
        ()
```

Inside `if` expressions, `is` can bind matched values to variables, which are then visible in the `:true` branch.

```rokugo
let Shape =
    | :square { x = Float32, y = Float32, side = Float32 }
    | :rect { x = Float32, y = Float32, width = Float32, height = Float32 }
    | :circle { x = Float32, y = Float32, radius = Float32 }

fun diagonal (shape : Shape) : Option Float32 =
    if shape is :square ({ let side } & _) then
        :some (side * (sqrt 2.0))
    else if shape is :rect ({ let width, let height } & _) then
        :some (sqrt (width * width + height * height))
    else
        :none
```

This can be combined with `and`, to chain multiple patterns in series.

```rokugo
fun move_camera (world : World) : World = do {
    var world = world

    if (World.query_one world) is :some ((let player : Player, let transform : Transform))
        and (World.query_one world) is :some ((let entity : Entity, let camera : Camera))
    then
        World.update entity transform (mut world)
    else ()

    world
}
```

#### `break` expression

The `break` expression exits out of the current `while` loop.

Since `break` is an expression, it can be used anywhere, such as in `match` arms and on the right-hand side of `or`.
In those cases, because `break` exits out of the loop immediately, there can be no value produced by the expression, and therefore its type is [`Nothing`](#bottom-type).

```rokugo
let Instruction =
    | :increment
    | :double
    | :halt

fun interpret (code : List Instruction) : Nat32 = do {
    var pc : Offset = 0
    var result : Nat32 = 0
    while :true {
        code |> List.at pc |> match {
            :increment => set result = result + 1
            :double => set result = result * 2
            :halt => break
        }
        set pc = pc + 1
    }
    result
}
```

## Patterns

Throughout all of Rokugo, patterns are the prevalent way of matching values against their shape.
As opposed to [literals](#literals), which _construct_ values, patterns _destruct_ them.

Patterns can be used in [bindings](#-binding-operator), [`match` expressions](#match-expression), and [`is` expressions](#is-expression).

A pattern can be _refutable_, or _irrefutable_.
When a pattern is refutable, it means it may not match against all values.
Irrefutable patterns on the other hand are the opposite - they always match against all values of a given type.

### Wildcard pattern

The wildcard pattern is an irrefutable pattern that matches any value without doing anything to it.
It is written down using the `_` keyword.

There are two common use cases for the wildcard pattern:

- As a catch-all arm in `match` expressions
- To explicitly signal that a value is being discarded

For example:

```rokugo
let NumberWord = :one | :two | :three

let int_to_word : Int32 -> Option NumberWord =
    match {
        1 => :some :one
        2 => :some :two
        3 => :some :three
        _ => :none
    }
```

### Primitive patterns

Primitive patterns match exactly a single literal primitive value (and are therefore refutable).

They can be used to match on numeric types, `String`s, and `Char`s.
Their syntax is exactly the same as those types' literals.

### Variable patterns

Variable patterns are irrefutable patterns which create or set existing variables.

For creating new bindings, Rokugo has `let` and `var`.

`let` creates a new, immutable variable, and is the most common way of bringing new names into scope.

```rokugo
let a = "hello"
(let a, let b, let c) = ("one", "two", "three")
```

`var` is similar to `let` but creates a new mutable variable, which can be then modified using the `set` pattern, although do note the latter can only be used on the left-hand side of the [`=` binding operator](#-binding-operator).
This is because it would be unclear what it's meant to do in the context of [`is`](#is-expression) and [`match`](#match-expression) expressions.

```rokugo
fun main () : () = do {
    var a : Int32 = 1
    var b : Int32 = 2
    set a = a + 2
    # Swap two variables around:
    (set a, set b) = (b, a)
}
```

The right-hand side of a `set` pattern is allowed to be a record field, provided that the record is stored within a mutable variable.

```rokugo
fun main () : () = do {
    var person = {
        name = "John",
        age = 42,
    }
    set person.age = person.age + 1
}
```

Do note that `var` bindings may only appear inside of `do` blocks.
This is because modifying variables inside modules would result in `require` order influencing compiled values, and in Rokugo compilation aims to be fully deterministic to help with build reproducibility.

### Or-patterns `|`

An or-pattern matches on two or more sub-patterns and chooses the first that matches.
Depending on the or-pattern's exhaustiveness, it may be considered refutable or irrefutable.

For example, the following or-pattern is irrefutable:

```rokugo
let Expr =
    | :two Float32
    | :four Float32

fun main () : () ~ Console = do {
    let e : Expr = :four 2.0
    e |> match {
        :two (let x) | :four (let x) => Console.write_line (x |> Float32.to_string)
    }
}
```

While the following or-pattern is refutable, and therefore the `match` requires an additional catch-all arm:

```rokugo
let Expr =
    | :two Float32
    | :four Float32

fun main () : () ~ Console = do {
    let e : Expr = :four 2.0
    e |> match {
        :two (let x) => Console.write_line (x |> Float32.to_string)
        _ => Console.write_line "four? what's that?"
    }
}
```

Similarly to [unions](#union-types), the first operand can be prefixed with an extra pipe `|`.
This extra pipe can be used even when the pattern is not an or-pattern, leading to a style of `match` similar to OCaml.

```rokugo
fun main () : () ~ Console = do {
    let e : Expr = :four 2.0
    e |> match {
        # Note the extra pipe here.
        | :two (let x) => Console.write_line (x |> Float32.to_string)
        | _ => Console.write_line "four? what's that?"
    }
}
```

### And-patterns `&`

And-patterns are used to match on the _rest_ left over by another pattern.
They are irrefutable if both the pattern on the left side of the `&` and the pattern on the right side of the `&` is irrefutable.

They're most useful in conjunction with [tuple](#tuple-patterns) and [record](#record-patterns) patterns, which must exhaustively list all the fields of the matched type.
In particular, an and-pattern allows for discarding the remaining fields of a tuple or record to match on it non-exhaustively.

```rokugo
let Player = {
    x = Float32,
    y = Float32,
    color = :red | :blue,
}

fun and_pattern_example (player : Player) : () = do {
    # We don't care about `color` or anything else, discard it.
    ({ let x, let y } & _) = player
}
```

### Tuple patterns

Tuple patterns are used to match on tuple values.
They are irrefutable if all the fields are matched against patterns that are irrefutable.

```rokugo
let vector = (1 : Int32, 2 : Int32, 3 : Int32)
(let x, let y, let z) = vector
```

Note that tuples must be matched exhaustively.
Fields cannot be left unmatched:

```rokugo
(1 : Int32, 2 : Int32, 3 : Int32) |> match {
    (let x, let y) => ()
    #            ^ error: tuple has 3 fields, but 2 were matched
}
```

To ignore fields, use an [and-pattern](#and-patterns-).

```rokugo
(1 : Int32, 2 : Int32, 3 : Int32) |> match {
    (let x, let y) & _ => ()
}
```

### Record patterns

Record patterns are used to match on record-like values.
Just like [tuple patterns](#tuple-patterns), they are irrefutable if all the fields are matched against patterns that are irrefutable.

Record-like values are values on which the [`.` operator](#-field-access-operator) can be used, and include records, modules, interfaces, and effects.

```rokugo
fun main () : () ~ Console = do {
    let vector = { x = 1 : Int32, y = 2 : Int32, z = 3 : Int32 }
    if vector is { x = 0, y = 0, z = 0 } then
        Console.write_line "zero vector"
    else
        Console.write_line "non-zero vector"
}
```

Record patterns support an additional shorthand for destructuring fields to variables of the same name.
This can be done by using `let x` or `var x` instead of `x = let x` or `x = var x` respectively.
The most useful use case for this is destructuring imported modules into variables within the current scope.

```rokugo
{ let Template } = require "rokugo/string"

# instead of

let String = require "rokugo/string"
let Template = String.Template
```

### Tag patterns

Tag patterns can be used to match on tag (or tagged) values.
Since for a tag type which doesn't have a payload the only possible value is the tag itself, tag patterns for payload-less tags are always irrefutable.
For a tag with a payload attached, a pattern is irrefutable if all the fields are matched against patterns that are irrefutable.

```rokugo
let astronomical_unit = :meters 149597870700
:meters (let astronomical_unit_in_meters) = astronomical_unit
```

### Type patterns

Type patterns can be used to match on the type of a value whose type can be one of many different types.
They are irrefutable if a value can be of only one type (its type is not a [union](#union-types).)

```rokugo
let Value =
    | Int64
    | Nat64
    | Float64
    | String

let ValueType =
    | :int
    | :nat
    | :float
    | :string

# Ascribing a type to `let` also makes use of type patterns.
let type_of : Value -> ValueType =
    match {
        _ : Int64 => :int
        _ : Nat64 => :nat
        _ : Float64 => :float
        _ : String => :string
    }
```

## Module system

Rokugo has a first-class module system.
A module groups items into a single value.
At a fundamental level, modules don't seem like much more than syntax sugar for [records](#records), and that's because they mostly _are_.

The two variables below are equal to each other:

```rokugo
let v1 = {
    x = 1.0 : Float32,
    y = 2.0 : Float32,
}

let v2 = module {
    let x : Float32 = 1.0
    let y : Float32 = 2.0
}

(v1 == v2) : :true
```

### Interfaces

The magic of modules comes in with _interfaces_.
Interfaces allow for specifying what sort of items a module must expose to fulfill an API contract.

An interface value is created with the `interface` keyword:

```rokugo
let Add = interface {
    let T
    fun add (a : T) (b : T) : T
}
```

As illustrated in the example above, interfaces contain an arbitrary listing of items, but they _do not need to have corresponding implementations._
`let` bindings in interfaces may have values (that's how interface instantiation happens after all), but `fun` items must not provide implementations, since that is the job of modules.

Then, a module can declare that it implements an interface, by specifying it after the `module` keyword:

```rokugo
let Int32Add = module : Add with { T = Int32 } {
    fun add (a : T) (b : T) : T =  # compiler builtin
}
```

Multiple interfaces can be implemented by creating an interface _product_ with `&`:

```rokugo
let Int32MathInterface =
    & Add with { T = Int32 }
    & Sub with { T = Int32 }
    & Mul with { T = Int32 }
    & Div with { T = Int32 }

let Int32Math = module : Int32MathInterface {
    fun add (a : T) (b : T) : T =  # compiler builtin
    fun sub (a : T) (b : T) : T =  # compiler builtin
    fun mul (a : T) (b : T) : T =  # compiler builtin
    fun div (a : T) (b : T) : T =  # compiler builtin
}
```

Note however that interfaces in a product must not have any conflicting symbols.
Therefore it is impossible to create a product with two different `T`:

```rokugo
let BadMathInterface =
    & Add with { T = Int32 }
    & Sub with { T = Float32 }  # error!
```

It is also important to note that interfaces, unlike most other types, are **nominal.**
This is because you generally don't want to implement someone else's API by accident, as that API might have a different _informal_ contract than yours.

```rokugo
let Greeting1 = interface {
    fun greeting (name : String) : String
}

let Greeting2 = interface {
    fun greeting (name : String) : String
}

Greeting1 : Greeting2  # type mismatch
```

### Module defaults

Since it would be quite inconvenient to have to write `Int32.Int32` every single time you want to reference the `Int32` type, Rokugo features a way for modules to pose as other values when not the left-hand side of a [`.` dot expression](#-field-access-operator).
This is what _module defaults_ are.

A module default is declared by using a `default` expression.
For example, this is how some of the modules in the prelude declare default items:

```rokugo
let Int32 = module {
    let Int32 = ... # compiler builtin
    default Int32
}

let Option = module {
    fun Option (a : _) =
        | :some a
        | :none
    default Option
}
```

Note that the right-hand side of `default` must be the name of an existing variable within the module.
While an arbitrary expression could have been allowed, it is important that other items within the module are still able to refer to the module default using a sensible name; therefore an existing variable has to be used.

In case a module defines a `default` value the module itself cannot be used as a value directly, because the compiler uses the module's `default` instead.
In that case, when referencing the module itself is needed, `TheModule.module` can be used.

For example, to alias the entire standard library `Int16` module to some other name:

```rokugo
let Short = Int16.module
```

Note that the _module_, not the default value is still used for field resolution in the `.` operator.
This can be somewhat unwanted in case of defining a module with an effect or interface posing as its default, as then the effect functions will be accessible using `SomeEffect.SomeEffect.the_function`, which is needlessly verbose.

In that case, the effect can be destructured into functions within the module itself using a [record pattern](#record-patterns).

```rokugo
let Log = module {
    let Log = effect {
        fun log (channel : String) (message : String) : ()
    }
    { let log } = Log
    default Log
}

# Here we use Log as an effect, by decaying the module into its default:
fun main () : () ~ Log = do {
    # But we can still refer to Log.Log.log via the alias.
    Log.log "main" "Hello!"
}
```

### Prelude

The prelude is a special standard library module from which all items are brought into scope by default.

It contains commonly used symbols like `Iterator`, `List`, etc. so that you don't have to import them manually in every single file.

### `require` and file scope

The `require` expression enables loading modules from the filesystem.

Semantically, Rokugo's `require` is similar to Lua's `require`.

The top-level of a Rokugo file is similar to the inner `{}` scope of a `do {}` block: the last expression in the file is its return value, and that value is then the result of calling `require` with the path to that file.

`require` paths are strings, which are structured like `package/path/to/module`.
In order to refer to modules local to the current package, `./path/to/module` is used.

Example importing a standard library module:

```rokugo
let Iterator = require "rokugo/iterator"
```

Using this system, a module file can be created by putting a `module {}` expression at the end:

**File:** src/my_module.rk

```rokugo
module {
    fun square (x : Int32) : Int32 = x * x
}
```

and then that `module {}` can be imported using `require`:

**File:** src/main.rk

```rokugo
let MyModule = require "./my_module"

fun main () : () ~ Console = do {
    Console.write_line (Int32.to_string (MyModule.square 2))
}
```

Note that this works with any value.
It is just as possible to create interface files, and even files from ordinary values such as records - which can be useful for compile-time configuration.

**File:** src/config.rk

```rokugo
{
    name = "Example",
    version = "1.0",
}
```

**File:** src/main.rk

```rokugo
let config = require "./config"

fun main () : () ~ Console = do {
    Console.write config.name
    Console.write " "
    Console.write_line config.version
}
```

### Item privacy, public types, and internal types

Rokugo has a notion of an item's _public type_ vs _internal type_ for all items in modules and at the file level.

The _public type_ is the type that comes after `:` in public items.
This is the type that is visible to external packages.

The _internal type_ is the _actual_ type of the item, and it's what comes after `=`.
This is the type that modules within the module's owning package see.

```rokugo
let Matrix = require "linalg/matrix"

module {
    # RenderState has a public type of {} - an abstract record with no fields.
    # Its internal type is a record with a field `transform_stack`.
    let RenderState : {} = {
        transform_stack = List Matrix,
    }
}
```

It is also possible to declare `internal` items within modules.
These items are not visible to external packages, but are visible within the current package.

```rokugo
let Matrix = require "linalg/matrix"

module {
    internal let TransformStack = List Matrix

    internal fun current_transform (stack : TransformStack) =
        List.last stack

    internal fun push_transform (stack : TransformStack) =
        List.push (stack |> current_transform)

    let RenderState : {} = {
        transform_stack = TransformStack,
    }

    fun push (state : RenderState) : RenderState =
        state with {
            transform_stack = push_transform state.transform_stack
        }
}
```

It is also possible to declare items which are completely private - this is illustrated with the `require "linalg/matrix"` import in the examples above.
The `Matrix` item is not exposed within the module returned from the file; it is only a variable local to the file itself.

Do note however that due to the way scoping of `module {}` works, it is impossible to reference internal or public items from within the returned module in private items.
A good rule of thumb to follow is that imports should be file-local, while any declared items should be `internal` in the returned module.

## Type system

Rokugo lacks an explicit universe of types.
Instead, ordinary values are used to express the types of other values.

In addition to that, most types in Rokugo are structural (with the exception of interfaces and effects, which are nominal for robustness reasons.)
This means that a value of type `a` can be passed in where a value of type `b` is expected, as long as type `a` has all the publicly visible features of `b`.

### Literals

The type of each literal is the literal itself.
For instance, all of the following expressions hold true:

```rokugo
1             : 1
"abc"         : "abc"
:my_tag       : :my_tag
:meters 123.0 : :meters 123.0
:meters 123.0 : :meters Decimal
```

### Numbers

Rokugo has three classes of numeric types:

- `Nat` - naturals (also known as unsigned integers in other languages)
- `Int` - integers (also known as signed integers in other languages)
- `Float` - IEEE 754 floating point numbers

Each numeric type has an explicit storage size.
For `Nat` and `Int`, storage sizes of 8, 16, 32, and 64 bits are available under the types `Nat8`, `Nat16`, `Nat32`, `Nat64`, `Int8`, `Int16`, `Int32`, and `Int64`.
For `Float`, storage sizes of 32 and 64 bits are available under the types `Float32` and `Float64`, representing IEEE 754 binary32 and binary64 floats.

In addition to specifically-sized `Nat`s and `Int`s, the types `Size` and `Offset` are available.
Both represent a host-specific `Nat` and `Int` respectively.
The storage size of both is the same, and the size of `Size` must be large enough to represent the length of any `Slice Nat8`.

### Union types

Union types express an _either-or_ relationship between two types.
The type `a | b` describes a value whose type can be _either `a` or `b`_, but not both at the same time.

Union types in Rokugo are structural just like most other types.

```rokugo
let AnyNumber1 = Float32 | Int32
let AnyNumber2 = Float32 | Int32

AnyNumber1 : AnyNumber2
```

The order of variants within the union is important, and defines the sorting order (`(<)` operator.)
The two unions below are not equivalent and will sort differently, but they are subtypes of each other and as such implicit conversion happens between them:

```rokugo
let AnyNumber1 = Float32 | Int32
let AnyNumber2 = Int32 | Float32

AnyNumber1 : AnyNumber2
```

Unions cannot contain duplicate variants. The following produces a compilation error:

```rokugo
let UnitOrUnit = () | ()  # error: union contains duplicate variants
```

This also means that it is not possible to directly use function parameters in union variants, as that would risk colliding with an existing variant.

```rokugo
fun Option t =
    # This is invalid, because `t` can be `:none` itself.
    t | :none
```

Instead, the parameter has to live inside a tag.

```rokugo
fun Option t =
    :some t | :none
```

It is of course possible to create unions with more than two variants.
In that case it can be useful to wrap the variants to separate lines.
The first variant can be prefixed with an additional `|` for vertical alignment.

```rokugo
let Color =
    | :rgb Float32 Float32 Float32
    | :rgba Float32 Float32 Float32 Float32
    | :lab Float32 Float32 Float32
```

Since a union can represent one of many different types, it has to be [pattern matched](#match-expression) to extract values out of it.

### Product types

Product types express an _and_ relationship between two types.
The type `a & b` describes a value whose type is a subtype of _both `a` and `b`_.
In other words, it creates a new type whose fields include those of two other types.

This is most useful when combining [records](#records), [interfaces](#interfaces), and [effects](#effect-system).

For records in particular, `a & b` creates a new record which includes the fields of `a`, and then the fields of `b`:

```rokugo
let Vec2 = { x = Float32, y = Float32 }
let Vec3 = Vec2 & { z = Float32 }
let Vec4 = Vec3 & { w = Float32 }

Vec4 : { x = Float32, y = Float32, z = Float32, w = Float32 }
```

Note that this is not the same as [product types in OCaml](https://ocaml.org/docs/basic-data-types#tuples).
In OCaml, product types are a fancy name for tuples, while Rokugo product types are abstract entities for combining type constraints.
Therefore, it's impossible to have a value whose type satisfies `Float32 & Int32`, and as such only interfaces and effects are allowed as members of product types.
It is also required that a product type contains _only_ interfaces, xor _only_ effects, because it is impossible for a value to be both an interface and an effect at the same time.

### Abstract types

A type is _abstract_ when its exact implementation is not known at usage site.
This is the case with types that come from interfaces, because it is impossible to know the exact definition.

For example:

```rokugo
let I = interface {
    let T
    let value : T
}

let M = module : I {
    let T = { i = Int32 }
    let value = { i = 1 }
}

fun example (mod : I) : () ~ Console =
    Console.write_line (Int32.to_string mod.value.i)
#                                       ^^^^^^^^^^^ compile error
```

The above example does not compile, because the type of `mod.value.i` is abstract and opaque: it does not have an `i` field, let alone one that would be an `Int32`.

### Wildcards

The identifier `_` represents a _wildcard_ type.
This is a type that accepts any value.
It is most useful when building polymorphic types for which type parameters do not have to fulfill any sort of contract.

One example of this is `Option t`:

```rokugo
fun Option (t : _) =
    | :some t
    | :none
```

Note that the `: _` is required because [all public module functions must specify their signatures](#function-declaration).

### Bottom type

The bottom type `Nothing` is a type with no values.
It coerces to any other type that is more appropriate in a given context.

`Nothing` is returned by all expressions that do not return back to the enclosing expression, such as `break` or `return`.

### Type hierarchy

As a dependently typed language, Rokugo doesn't distinguish between _values_ and _types_ per se; but then the question arises, _what does it mean to even check types in that context?_

TODO

## Ownership and linear typing

Rokugo has two types of values: _data_ and _resources_.

Data are ordinary values which can be freely copied around functions.
That's because they're really just that - _plain old data_.

Note that Rokugo has no notion of a reference.
All values are copied around when assigned to variables, passed as arguments, etc.
The runtime may freely optimize these copies to share memory when that's more efficient, so you shouldn't worry about copies becoming too expensive.

Almost all values in Rokugo are data.
The only exception to this rule is any type which is a [host resource](#host-resources), such as files, sockets, and other types specific to the host.
The types of these values are _linear_, which means that once a value of that type is created, it cannot be copied, and must be used _exactly_ once.
This is opposed to affine typing (as used by [Rust](https://rust-lang.org)), which allows you to use a value one _or_ zero times.

In short, the rules are:

- When a linear value is returned from a host function, it must also be consumed back by another host function.
- Assigning a linear value to a variable or passing it to a parameter _moves_ it.
- When a linear value is moved out of a variable, it may no longer be referenced through that variable.

Data types turn into resource types if they contain any resources.
For example, a `List File` is a resource type, not a data type like `List Nat8`.
The same thing happens with records, tuples, tags, unions, and other containers.

A simple resource that can be used to demonstrate the linear typing system is `Linear`, which serves as a marker to turn a data type into a resource type.
(In the example below it is used alone to make the example more minimal.)

```rokugo
let Linear = require "rokugo/linear"

fun main () : () = do {
    # Create a new Linear value and store it in `a`.
    let a = Linear.new ()

    # Now move the Linear out of `a` into `b`.
    let b = a
    # The Linear cannot be reached through `a` anymore.

    # After we no longer need the Linear, we must dispose of it.
    # If there are any code paths where the Linear is not disposed, the compiler will tell us.
    Linear.drop b
}
```

Linear typing interacts with the effect system, too.
This is reflected in the [continuation types](#continuations), which all have their own rules for calling them.
Refer to their individual documentation for more info.

### Mutation of resources through function arguments

Since passing a resource to a function argument consumes it, to represent mutation Rokugo requires the function to return the resource back to the caller.

```rokugo
fun main () : () ~ IO & Throw = do {
    var file = File.open "hello.txt"
    (let line, set file) = File.read_line file
    File.close file
}
```

Notice how `File.read_line` returns back the file the line was read from.
This is because more data can be read from the stream after reading just a single line, so consuming the file wouldn't make much sense.

This can become verbose pretty quickly, so Rokugo has a shorthand syntax for assigning returned arguments back to variables:

```rokugo
fun main () : () ~ IO & Throw = do {
    var file = File.open "hello.txt"
    let line = File.read_line (mut file)
    File.close file
}
```

Any number `n` of `mut` expressions can be used in function arguments to chop the last `n` values off the end of the returned tuple, and assign them back to variables whose values are passed to the function as arguments.
More generally, the transformation is:

```rokugo
fun function (x : Int32) (a : T1) (b : T2) (c : T3) : (Int32, T1, T2, T3) =
    (x + 1, a, b, c)

fun call (a : T1) (b : T2) (c : T3) : () = do {
    (var a, var b, var c) = (a, b, c)
    let i = function 10 (mut a) (mut b) (mut c)
}
```

When a single-value tuple is returned, the value inside is returned.
This avoids the need for ugly `(let result,) = expr` unpacking when calling the function.

Just like `set`, `mut` can also be used on fields of records stored in `var`s.

## Effect system

Rokugo features an algebraic effect system for annotating functions which can cause side effects.

In pure functional programming, the result of a function is only dependent on its input parameters.
This often turns out to be too inflexible in large codebases, since now every single bit of context has to be passed into functions as a parameter, and every function which wants to modify that bit of context has to return a modified value back.
In turn this makes code much more brittle when it comes to refactoring; when a new piece of context is needed, it can be hard to add it in without breaking half of the existing codebase.

This also causes problems when trying to perform I/O or other side effects.
These have to be represented via monads to make the order of computations and side effects explicit, and monads are generally not the most intuitive concept to grasp.

And that's why the effect system exists.

### Effect declarations

Effects are declared using the `effect` keyword, followed by a list of _operation_ functions that form the effect's interface.

Operation functions do not have return types, and instead their last parameter is of a [continuation type](#continuations).

```rokugo
let Log = effect {
    fun log (message : String) (cont : Return ())
}
```

From a usage perspective, an `effect` declaration roughly desugars to a module with lots of compiler magic:

```rokugo
let Log = module {
    let Log = ...  # effect declaration magic
    default Log

    # Note the extra Log effect on the function.
    fun log (message : String) (cont : Return ()) : () ~ Log =
        ...  # call the current handler for the effect
}
```

The `Log.log` acts like a regular function and can be called, stored in variables, passed around in arguments, and so on.
What's magic about it is the continuation parameter `cont : Return ()`, which influences how calling the function works.

More about continuations can be learned in [their own section](#continuations).

### Effect handlers

Calling an operation causes the function to inherit that operation's effect.
For example, calling `Log.log` causes a function to get the `Log` effect.

However, for `Log.log` to do anything useful, an implementation of the operation has to be provided somewhere down the call stack.
This is where `handle` expressions come in, which attach an _effect handler_ to a given expression.

```rokugo
fun do_logging_stuff () : () ~ Log = do {
    Log.log "Hello, world!"
    Log.log "This is handled by an effect."
}

fun main () : () ~ Console = do {
    # The expression to the left of `handle` is performed with a handler of the effect provided on
    # the right (in this case `Log`).
    do_logging_stuff () handle Log {
        # This is the effect handler, where we declare implementations of the effect's operations.
        # Type inference can be used to shorten the declarations.
        fun log message cont = do {
            Console.write_line message
            cont ()
        }
    }
}
```

An effect handler always handles exactly one effect.
Since `handle` is an infix expression, it can be chained to attach multiple effect handlers:

```rokugo
fun main () : () = do {
    some_function () handle A {
        # ...
    } handle B {
        # ...
    } handle C {
        # ...
    }
}
```

An idiom that comes out of this naturally is attaching an effect handler to a `do {}` block:

```rokugo
fun main () : () = do {
    let v = some_function ()
    consume_somehow v
} handle A {
    # ...
}
```

### Continuations

With the basics out of the way, now it's time to talk about that `cont` parameter of operations.

This parameter defines how the operation influences control flow in the calling function.
The name _continuation_ comes from the fact that it's a function that _continues_ executing the caller's code.

Rokugo defines several continuation types, each with its own restrictions.

#### `Abort`

The `Abort` continuation type does not actually resume execution of the calling function.

Instead, when called, `Abort` exits the program with an exit code.

```rokugo
let Panic = effect {
    fun panic (message : String) (abort : Abort)
}

fun main () : () ~ Console = do {
    # ...
} handle Panic {
    fun panic message abort = do {
        Console.write "panic: "
        Console.write_line message
        Console.write_line "aborting"
        abort Abort.ExitCode.failure
    }
}
```

In an aborting operation, `Abort` must be the last function called in all control flow paths.

#### `Return`

The `Return` continuation type resumes execution of the calling function.

It must always be called within the effect handler itself, and may not be taken out of scope.
Additionally, a `Return` continuation must be the last function called in all control flow paths (a _tail call_).
This means it is impossible to continue executing more code after a `Return` continuation is called.

Control flow-wise, `Return`ing operations act exactly like regular functions.
After a `Return`ing operation is called, control flow is guaranteed to continue back in the calling function immediately after the handler is done working.
This makes them as efficient as calling a function.

`Return` takes a single argument, which is the return value of the operation.

```rokugo
let Log = effect {
    fun log (message : String) (cont : Return ())
}

fun main () : () ~ Console = do {
    Log.log "hello!"
    Log.log "another log here"
} handle Log {
    fun log message cont =
        cont do {
            Console.write "log: "
            Console.write_line message
        }
}
```

Another example, returning a non-`()` value:

```rokugo
let AutoIncrement = effect {
    fun next (cont : Return Nat32)
}

fun with_counter (f : () -> let r ~ AutoIncrement & let e) : r ~ e = do {
    var counter : Nat32 = 0
    f () handle AutoIncrement {
        fun next cont = do {
            let value = counter
            set counter = counter + 1
            cont value
        }
    }
}

fun main () : () ~ Console = with_counter fun () = do {
    Console.write_line (Nat32.to_string AutoIncrement.next)  # 0
    Console.write_line (Nat32.to_string AutoIncrement.next)  # 1
    Console.write_line (Nat32.to_string AutoIncrement.next)  # 2
}
```

#### `Continue`

The `Continue` continuation type is used for continuations that may execute later, but are guaranteed to execute before `main` is done running.

As opposed to `Return`, `Continue` continuations may be taken out of the effect handler function's scope.
They also don't have to be called via a tail call, as additional code may execute after a `Continue` continuation is called.
However, they may only be called once.

`Continue`s are restrictive as they are to uphold the guarantees of [linear types](#ownership-and-linear-typing).
To recap, a linear value must be used exactly once.
This means that if it's captured as part of any continuation, that continuation must also be executed exactly once.

`Continue`s may also declare an effect type, like `Continue r ~ e`.
This is necessary because the continuation may be executed at any later point in time, and there must be an handler installed for that effect at that point.
Note that the continuation itself may call back to the operation's effect, and this is denoted in its signature.

Example:

```rokugo
let LaterCont = Continue () ~ Console & Later

let Later = effect {
    fun pause (cont : LaterCont)
}

fun main () : () ~ Console = do {
    var todo : List LaterCont = List.of ()
    do {
        Console.write_line "Creating file"
        let file = File.create "my_file.txt"

        Later.pause

        Console.write_line "Writing line"
        File.write_line file "hello!!"

        Later.pause

        Console.write_line "Closing file"
        File.close file
    } handle Later {
        fun pause cont =
            set todo = List.push cont todo
    }

    while not (List.is_empty todo) do {
        Console.write_line "--- resuming ---"
        let cont = List.pop (mut todo)
        cont () handle Later {
            fun pause cont =
                set todo = List.push cont todo
        }
    }
}
```

```text
Creating file
--- resuming ---
Writing line
--- resuming
Closing file
```

Note that there is no risk of these continuations running out of order.
Executing the "Closing file" continuation is only possible after the previous continuation finishes running and calls back to `Later.pause`, giving us another continuation, consuming the previous one in the process.

#### `MaybeContinue`

`MaybeContinue` are continuations which give you full freedom as to how and when to execute them.
From a usage perspective, `MaybeContinue` continuations are just regular functions.
They can be called whenever you'd like, and however many times you'd like.

This comes with the limitation that [linear values](#ownership-and-linear-typing) **must not** be carried across calls to `MaybeContinue` operations.
This is because `MaybeContinue` operations may be executed zero or more times, which would break the guarantees of the linear typing system (as all linear values have to be used _exactly_ once.)

Other than that, `MaybeContinue` does not differ in usage to `Continue`.
Effects performed by `MaybeContinue` continuations still need to be annotated explicitly using the same syntax `MaybeContinue r ~ e`.

## Multithreading

## Compilation and runtime

## Embedding

Rokugo is designed to be used as a plugin language for applications.
Although a rich, standalone platform for interfacing with OS facilities is provided, the Rokugo runtime itself is embeddable within larger programs.

To facilitate this, there are a bunch of mechanisms for interfacing with the _host application_.

### Host protocols

Host protocols are interfaces that contain functions which, when called, are executed by the host application instead of the language runtime.

Host protocols can be created using a regular `interface`, except all functions which are to be executed by the host application should be [annotated](#-annotation-sigil) with the `Host.function` annotation.
Then, the `Host.protocol` function can be used to retrieve a module implementing the interface.

`Host.protocol` accepts two arguments: the protocol interface, as well as a UUID uniquely identifying that protocol.
Interface names and identity is fully anonymous, so the UUID is used to sync the compiler, runtime, and host together into agreement as to which protocol is being talked about.

```rokugo
let Host = require "rokugo/runtime/host"

let IntAdderProtocol = interface {
    fun add (a : Int32) (b : Int32) : Int32
}

let IntAdder = Host.protocol IntAdderProtocol "4aa52322-da76-45df-9f78-3361bd70e758"
```

Host functions have limited capabilities compared to functions implemented in the language runtime.
Most importantly, they cannot be paused, and therefore cannot call non-`Return` effects.

Since host protocol functions can interact with the outside world in an impure way, it's advised to always wrap them in effect handlers, unless you are absolutely sure a function is pure.
The example below showcases how this can be done for a simple (albeit incomplete) I/O interface:

```rokugo
module {
    let File = Host.resource "d4b4b87b-029c-4026-be66-7ed3271316e9"

    # Naive I/O interface ignoring errors.
    let IO = effect {
        fun write (file : File) (bytes : Slice Nat8) (cont : Return File)
    }

    internal let HostIOProtocol = interface {
        fun write (file : File) (bytes : Slice Nat8) : File
    }

    internal let HostIO = Host.protocol HostIOProtocol "32a8bd1e-d82b-40b0-8cab-2d789e993fdc"

    fun with_host_io (f : () -> let r ~ IO & let e) : r ~ e =
        f () handle IO {
            fun write file bytes = HostIO.write file bytes
        }
}
```

### Host resources

Host resources are opaque, [linear](#ownership-and-linear-typing) values allocated and managed by the host.

A host resource type can be retrieved by using the `Host.Resource` function.

```rokugo
let Texture = Host.resource "deecd85a-564a-421f-b164-905aeb7f3694"
```

Creation and destruction of a host resource value is performed using a [host protocol](#host-protocols).
Because host resources are linear, there is no risk of running into a [double free](https://en.wikipedia.org/wiki/C_dynamic_memory_allocation#Use_after_free).

## Style and naming conventions

- Indent with 4 spaces.
- No newline before the opening brace `{`.
- Name values with `snake_case`.
- Name types and modules with `PascalCase`.
  - Everything that is intended to be used as the right-hand side of type ascription `:` is considered a type.
- Functions which construct values should be called `new`, and functions which destroy values should be called `drop`.
  - In case there is more than one constructor, `new` may be swapped out for something else (as is the case with `File.new` and `File.open`.)
