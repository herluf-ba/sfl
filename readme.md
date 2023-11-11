# `S`mall `F`unctional `L`anguage
This is a hobby project aiming to implement a small functional language.
**Goals**:
- Implement a functional language
- Typecheck using Hindley-Milner
- Have fun


## IDEAS
- Make the type checker able to annotate an AST with types / make a new AST with type info. This would be nice for IDE stuff

## TODO:
- figure out a way to test for regressions... Have some sort of folder of known to work files?  


## Syntax
```
  def foo(a: number, b: number): number { 
    a + b
  }

  def bar(a: number): number {
      if a > 2 { a - 2 } else { a }
  }
  
  def main(): number {
    let fooo = foo;
    let a = foo(fooo(1, 2), 3);
    let b = bar(1);
    a / b
  }
```