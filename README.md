# zebra
Program Synthesis is Possible: https://www.cs.cornell.edu/~asampson/blog/minisynth.html#:~:text=Program%20synthesis%20is%20not%20only,computer%2Dscience%20life%20can%20use.
```
x * 10
x << ?a + x << ?b
```
```
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/zebra`
(forall ((|x | (_ BitVec 8)))
  (= (bvmul |x | #x0a) (bvadd (bvshl |x | |?a |) (bvshl |x | ?b))))
?b -> #x01
?a  -> #x03
```
