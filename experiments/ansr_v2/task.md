you need improve tune result
you need increase finite per popsize with wide range params
you need decrease best, mean and mad per popsize
you can run `cargo run --bin tune -r`
you can't change popsize params
commit every change
write progress in progress.md
don't use subagents
one run in one time
param math bounds: popsize ∈ [d/2, 2d] (4 options), restart_tolerance ∈ [1e-8,1], sigma ∈ (0,1], self_instead_neighbour ∈ [0,1]
all params (exclude popsize) should cover full math bounds
you can change algorithm, you can add new params but better fewer params, new params only in math bounds
continue experiments until i say stop
update progress.md after any run
don't change random init
always prefer tune algo, not tune hyper params
speed up test, don't make it slower (don't add more grid values)
if finite improves and best doesn't get worse, that's ok (mean can increase when more configs converge)