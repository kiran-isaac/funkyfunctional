# SFL Explorer
This was my dissertation project. SFL explorer (https://functional.kiransturt.co.uk/) is a website that allows you to see the step-by-step evaluation of a made up functional language: SFL (Simple Functional Language). SFL is a type-checked haskell-like function language.

My Dissertaion can be found at the repo root: [Dissertation.pdf](Dissertation.pdf)

## Example SFL Code and Evaluation
pairs
```
pair :: a -> b -> (a, b)
pair x y = (x, y)

first :: (a, b) -> a
first (x, _) = x

second :: (a, b) -> b
second (_, x) = x

main :: Bool
main = second $ (pair 1 true)
```
Evaluation (read bottom to top)
<img width="2556" height="1369" alt="image" src="https://github.com/user-attachments/assets/3df32e86-66b7-4a1e-beb6-d83de12f2233" />

The prelude can be found in [prelude.sfl](prelude.sfl)

## Usage
On the website you may enter a program. By default the program includes the Prelude, but this can be turned off in settings. Many default programs are included for your conveniance. 

You may choose between lazy and free choice evaluation. Free choice allows you to pick from any of the valid next steps (i.e. expressions to reduce).

## Typechecking

Based on "Complete and Easy Bidirectional Typechecking for Higher-Rank Polymorphism" [Jana Dunfield, Neelakantan R. Krishnaswami]. See section 4.3.5 of my dissertation.

## The Website
The website was developed by compiling the rust-based interpreter to wasm, and then fetching this from a React fromtend, along with a wrapper package (see `wasm_lib` and `frontend` at repo root).
