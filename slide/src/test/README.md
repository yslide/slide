# slide system tests

This directory provides a runner for slide's system tests. System tests should be added to
subdirectories of this directory.

A system test checks the standard output and error for a run of a program through the slide CLI. If
needed, a system test can specify CLI options to be used in the test.

Slide system tests have the form

```slide-test
@<annotation>*

!!!args
<CLI args>
!!!args

===in
<program input>
===in

~~~stdout
<standard output>
~~~stdout

~~~stderr
<standard output>
~~~stderr

~~~exitcode
<exit code>
~~~exitcode
```

We highly suggest running system tests via slide's [ladder](../../../ladder) build manager.

```bash
ladder test --sys         # run all system tests
ladder test --sys --bless # accept system test outputs as baselines
```

### Optional clauses

- Annotations (see the [annotations](#Annotations) section below).
- The `!!!args` clause; it does not need to be included if your test does not require non-default
  CLI arguments.

## Example workflow

Let's say we want to add a test to check that `x + 1 + 2 -> x + 3`. To start, create a `.slide` test
file with the input:

```slide-test
===in
x + 1 + 2
===in
```

We're going to ask the test runner to generate the output of the program for us, and then we can do
a manual check to make sure the results are correct. To bless the system tests, run

```bash
ladder test --sys --bless
```

The contents of the test file should now be

```slide-test
===in
x + 1 + 2
===in

~~~stdout
x + 3
~~~stdout

~~~stderr
~~~stderr

~~~exitcode
0
~~~exitcode
```

Awesome! Exactly what we expected.

Now, let's say we want to check that the s-expression form of evaluation is correct. We need to add
an explicit args clause, because s-expression output is not a default output of slide.

```slide-test
!!!args
-o s-expression
!!!args

===in
x + 1 + 2
===in

~~~stdout
x + 3
~~~stdout

~~~stderr
~~~stderr

~~~exitcode
0
~~~exitcode
```

Let's see what happens when we run the test:

```slide-test
ladder test --sys

running 1 test
test [system] ui/add_x_1_2.slide ... FAILED

failures:

---- ui/add_x_1_2.slide ----
Mismatch in stdout:
-x + 3

+(+ x 3)
```

We forgot to update the expected output! Let's do that now (either manually or with `--bless`):

```slide-test
!!!args
-o s-expression
!!!args

===in
x + 1 + 2
===in

~~~stdout
(+ x 3)
~~~stdout

~~~stderr
~~~stderr

~~~exitcode
0
~~~exitcode
```

Running the tests again, we now get a success.

```
ladder test --sys

running 1 test
test [system] ui/add_x_1_2.slide ... ok
```

## Annotations

Annotations instruct the test to treat a system test in a certain way. The following annotations are
supported:

- `@TODO`: if specified, `ladder test --fail-todo` will fail on tests with this annotation. The
  annotation may be followed by a colon-separated message: `@TODO: make this better`.

Annotations should appear at the top of a slide test file, with exactly one annotation per line.

```slide-test
@TODO: why doesn't this add?

===in
1 + 2
===in

~~~stdout
1 + 2
~~~stdout

~~~stderr
~~~stderr

~~~exitcode
0
~~~exitcode
```
