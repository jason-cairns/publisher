# ADR 0004 — Publications Compile Independently

## Status

Accepted

## Context

The site contains multiple publication types:
- standard pages
- thesis chapters
- writing entries
- recommendation sections

Possible approaches:
- recursive publication graphs
- monolithic site compilation
- standalone publication compilation

## Decision

Every publication compiles independently.

Each publication:
- produces standalone HTML
- contributes to the unified PDF
- owns exactly one route

Recursive ownership and inherited publication-index semantics are expressed
through Typst ownership edges, not through nested compilation.

Cross-publication reference relationships are expressed through links only.

Publication ownership is expressed separately through explicit Typst ownership edges defined in ADR 0005.

The unified PDF is assembled from the ownership-derived rendered set in
ownership preorder. The assembly file is generated during the build, so
there is no manually maintained list of PDF includes.

## Consequences

Positive:
- strong isolation
- easy testing
- predictable builds
- simple routing model
- reduced hidden coupling

Negative:
- shared layout behavior must be implemented separately
- unified PDF assembly requires generated build orchestration
