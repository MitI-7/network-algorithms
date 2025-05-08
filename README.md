# network-algorithms

## Shortest Path
- dijkstra
  - O((n + m) log(n))
- bellman ford
  - O(nm)

## Maximum Flow

- ford fulkerson
    - O(nmU)
- edmonds karp
    - O(n^2 m)
- dinic
    - O(n^2 m)
- capacity scaling(dinic)
    - O(nm log U)
- shortest augmenting path
- push relabel(fifo)
- push relabel(highest label)

## Minimum Cost Flow

- cost scaling push relabel
- cycle canceling
    - O(nm^2 CU)
- out of kilter
    - O(nU \* (m + n) log n)
- primal dual
    - O(min{nU, nC} n^2 m)
- successive shortest path
    - O(nU \* (m + n) log n)
- primal network simplex
- dual network simplex
- parametric network simplex

## Generalized Maximum Flow
- highest gain path
- primal dual(dinic)
- primal dual(push relabel)

## maximum matching
- blossom
  - O(n^3)
## maximum bipartite matching
- hopcroft karp
  - O(m sqrt(n))