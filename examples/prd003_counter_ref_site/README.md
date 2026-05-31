# PRD-003 counter/reference discovery fixture

This fixture backs the PRD-003 discovery report for cross-source counters and
cross-source HTML references.

Key checks:

```sh
typst compile --features html --format html examples/prd003_counter_ref_site/thesis-combined.typ /private/tmp/thesis-combined.html
typst compile --features html --format html examples/prd003_counter_ref_site/thesis/ch-02.typ /private/tmp/ch-02-unseeded.html
typst compile --features html --format html examples/prd003_counter_ref_site/thesis-ch-02-seeded.typ /private/tmp/ch-02-seeded.html
typst compile --features html --format html examples/prd003_counter_ref_site/combined-ref.typ /private/tmp/combined-ref.html
typst compile examples/prd003_counter_ref_site/blog-standalone.typ /private/tmp/blog-standalone.pdf
typst compile --features html --format html examples/prd003_counter_ref_site/blog-adapted-ref.typ /private/tmp/blog-adapted-ref.html
```
