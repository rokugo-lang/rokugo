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

They are made up of characters, which have the type `Char` and use single-quoted literals:

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

Records are unordered, heterogenous containers storing key-value pairs.
They are delimited by braces.

```rokugo
{ greeting = "hello", target = "world" }
```

Record fields can be accessed using the dot `.` operator.

```rokugo
{ greeting = "hello", target = "world" }.greeting  # "hello"
```

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

Tags are mostly used with [unions](#unions), but can be useful on their own to attach unit information to values.

### Operators

#### User-defined operators

All operators except [magic operators](#magic-operators) are ordinary, user-defined [functions](#functions).

An operator is made up of a sequence of one or more of the following characters:

```text
+ - * / %
< = > ! ?
^ & | ~ $
. : @ \
```

The operators `.`, `:`, `=`, `@`, `&`, and `|` have a [special meaning to the compiler](#magic-operators) and are reserved.

#### Magic operators

The following operators have a special meaning to the compiler.

##### `.` field access operator

`a.x` accesses the field `x` of the tuple or record `a`.

##### `:` type ascription operator

`expr : T` enforces the `expr` to be of type `T`. If the coercion cannot be performed, a compilation error is emitted.

##### `=` binding operator

The `=` operator is used for pattern matching irrefutably, as well as a separator in `fun` bindings, and in record field bindings.

##### `|` union operator

`a | b` represents a [type union](#unions).

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

##### `@` annotation sigil

Not actually an operator, but a reserved token used for annotations.

##### `and` and `or`

`and` and `or` are short-circuiting boolean control flow operators.

`and` represents the boolean AND operation, while `or` represents the OR operation.

The operators are short-circuiting because if the result of the operation is known from evaluating the expression on the left-hand side, the right-hand side will not be evaluated.

- If the left-hand side of `and` is `:false`, the result of the whole expression is known to be `:false` and as such the right-hand side will not be evaluated.
- If the left-hand side of `or` is `:true`, the result of the whole expression is known to be `:true` and as such the right-hand side will not be evaluated.

#### Precedence

Precedence of operators in Rokugo is statically determined based on the characters the operator is made out of.

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
    { x: x, y: y }
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

Signature type inference is supported within module-local functions.
Any functions which are visible outside the current module must specify the function signature explicitly.
This can be done by [ascribing a type](#-type-ascription-operator) to each of the function's parameters, as well as the function itself.

```rokugo
fun greeting (name : String) : String =
    String.cat "Hello, " name
```

This aids in documentation, as well as keeping the interface of the function stable across versions.

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

Since it would be quite inconvenient to have to write `Int32.Int32` every single time you want to reference the `Int32` type, Rokugo features a way for modules to pose as other values in specific contexts.
This is what _module defaults_ are.

A module default is declared with the `default` keyword in a module, followed by an item.
This declaration can be one of the following:

- `default let = expr` - declares the value the module should decay to if not used in a [`.` dot expression](#-field-access-operator)
- `default fun x = x` - declares the function to use if the module is applied like a function
- `default effect {}` - declares the effect to use if the module is used as a function's effect

For example, this is how some of the modules in the prelude declare default items:

```rokugo
let Int32 = module {
    default let = ... # compiler builtin
}

let Option = module {
    default fun a =
        | :some a
        | :none
}

let Console = module {
    default effect {
        fun read () : ()
        fun write (data : Slice Nat8) : ()
    }
}
```

If any of these symbols need to be referred to by name, it is possible to do so by using `Module`, `Module.fun`, or `Module.effect` respectively.

In case a module defines a `default let`, the module itself cannot be used as a value directly, because the compiler uses the module's `default let` instead.
In that case, when it's needed to reference the module itself, `Module.module` can be used.

For example, to alias the entire standard library `Int16` module to some other name:

```rokugo
let Short = Int16.module
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

Privacy can be achieved by ascribing the module with an explicit interface type.
Note that although interfaces are nominal, it doesn't matter here since the interface is only used to coerce the module into a narrower type.

```rokugo
module {

    let RenderState = {
        transform_stack = List Matrix,
    }

    fun current_transform state =
        List.last state.transform_stack

    fun push_transform state =
        state with {
            transform_stack = state.transform_stack |> List.push (state |> current_transform)
        }

} : interface {

    let RenderState

    fun push_transform (s : RenderState) : RenderState

}
```

Modules ascribed with an interface this way have one extra advantage, and it has to do with package-local `require`s.
Namely, `require` recognizes this idiom and exposes the entire interface to the package, despite the publicly declared interface being narrower.

## Type system

Rokugo lacks an explicit universe of types.
Instead, ordinary values are used to express the types of other values.

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

### Slices

### Type inference

## Effect system

## Compilation and runtime
